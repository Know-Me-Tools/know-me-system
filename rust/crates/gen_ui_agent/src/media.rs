// TJ-ARCH-MOB-001 compliant
//! Safe Rust-owned remote media and OpenGraph resolver. Flutter never performs
//! HTTP itself; every redirect and DNS result is checked against SSRF targets.
use std::net::{IpAddr, SocketAddr};

use base64::Engine as _;
use futures::StreamExt;
use scraper::{Html, Selector};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::AgentError;

const MAX_REDIRECTS: usize = 3;
const MAX_IMAGE_BYTES: usize = 12 * 1024 * 1024;
const MAX_HTML_BYTES: usize = 2 * 1024 * 1024;
const MAX_VIDEO_BYTES: usize = 100 * 1024 * 1024;
const MAX_AUDIO_BYTES: usize = 30 * 1024 * 1024;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedRemoteContent {
    pub url: String,
    pub mime: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

pub async fn resolve(url: &str, kind: &str) -> Result<ResolvedRemoteContent, AgentError> {
    let limit = match kind {
        "image" => MAX_IMAGE_BYTES,
        "video" => MAX_VIDEO_BYTES,
        "audio" => MAX_AUDIO_BYTES,
        "opengraph" => MAX_HTML_BYTES,
        _ => return Err(AgentError::Media("unknown content kind".into())),
    };
    let (final_url, mime, bytes) = fetch_bounded(url, limit).await?;
    match kind {
        "image" => {
            if !mime.starts_with("image/") {
                return Err(AgentError::Media(format!(
                    "expected an image response, received {mime}"
                )));
            }
            Ok(ResolvedRemoteContent {
                url: final_url.to_string(),
                mime,
                data_base64: Some(base64::engine::general_purpose::STANDARD.encode(bytes)),
                local_path: None,
                title: None,
                description: None,
                site_name: None,
                image_url: None,
            })
        }
        "video" | "audio" => {
            let expected_prefix = format!("{kind}/");
            if !mime.starts_with(&expected_prefix) {
                return Err(AgentError::Media(format!(
                    "expected an {kind} response, received {mime}"
                )));
            }
            let digest = format!("{:x}", Sha256::digest(final_url.as_str().as_bytes()));
            let extension = match mime.as_str() {
                value if value.contains("webm") => "webm",
                value if value.contains("ogg") => "ogg",
                value if value.contains("wav") => "wav",
                value if value.contains("mpeg") => "mp3",
                value if value.contains("mp4") && kind == "audio" => "m4a",
                _ => "mp4",
            };
            let directory = std::env::temp_dir().join("knowme-media-cache");
            tokio::fs::create_dir_all(&directory)
                .await
                .map_err(|error| AgentError::Media(error.to_string()))?;
            let path = directory.join(format!("{digest}.{extension}"));
            tokio::fs::write(&path, bytes)
                .await
                .map_err(|error| AgentError::Media(error.to_string()))?;
            Ok(ResolvedRemoteContent {
                url: final_url.to_string(),
                mime,
                data_base64: None,
                local_path: Some(path.to_string_lossy().into_owned()),
                title: None,
                description: None,
                site_name: None,
                image_url: None,
            })
        }
        "opengraph" => {
            if !mime.starts_with("text/html") && !mime.contains("xhtml") {
                return Err(AgentError::Media(format!(
                    "expected an HTML response, received {mime}"
                )));
            }
            let html = String::from_utf8_lossy(&bytes);
            let document = Html::parse_document(&html);
            let meta = Selector::parse("meta").expect("static selector");
            let title_selector = Selector::parse("title").expect("static selector");
            let property = |wanted: &str| {
                document.select(&meta).find_map(|element| {
                    let value = element.value();
                    let key = value.attr("property").or_else(|| value.attr("name"))?;
                    key.eq_ignore_ascii_case(wanted)
                        .then(|| value.attr("content").map(str::trim).map(str::to_owned))
                        .flatten()
                })
            };
            let title = property("og:title").or_else(|| {
                document
                    .select(&title_selector)
                    .next()
                    .map(|element| element.text().collect::<String>().trim().to_owned())
                    .filter(|value| !value.is_empty())
            });
            let image_url = property("og:image").and_then(|value| {
                final_url
                    .join(&value)
                    .ok()
                    .map(|resolved| resolved.to_string())
            });
            Ok(ResolvedRemoteContent {
                url: final_url.to_string(),
                mime,
                data_base64: None,
                local_path: None,
                title,
                description: property("og:description").or_else(|| property("description")),
                site_name: property("og:site_name"),
                image_url,
            })
        }
        _ => unreachable!(),
    }
}

pub(crate) async fn fetch_bounded(
    source: &str,
    max_bytes: usize,
) -> Result<(reqwest::Url, String, Vec<u8>), AgentError> {
    let mut url = reqwest::Url::parse(source)
        .map_err(|error| AgentError::Media(format!("invalid URL: {error}")))?;
    for redirects in 0..=MAX_REDIRECTS {
        let (client, validated_url) = client_for_url(url).await?;
        let response = client
            .get(validated_url.clone())
            .header(reqwest::header::ACCEPT, "*/*")
            .send()
            .await
            .map_err(|error| AgentError::Media(error.to_string()))?;
        if response.status().is_redirection() {
            if redirects == MAX_REDIRECTS {
                return Err(AgentError::Media("too many redirects".into()));
            }
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| AgentError::Media("redirect missing Location".into()))?;
            url = validated_url
                .join(location)
                .map_err(|error| AgentError::Media(format!("invalid redirect: {error}")))?;
            continue;
        }
        if !response.status().is_success() {
            return Err(AgentError::Media(format!(
                "remote server returned {}",
                response.status()
            )));
        }
        if response
            .content_length()
            .is_some_and(|size| size > max_bytes as u64)
        {
            return Err(AgentError::Media("remote content is too large".into()));
        }
        let mime = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("application/octet-stream")
            .split(';')
            .next()
            .unwrap_or("application/octet-stream")
            .trim()
            .to_ascii_lowercase();
        let mut bytes = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|error| AgentError::Media(error.to_string()))?;
            if bytes.len() + chunk.len() > max_bytes {
                return Err(AgentError::Media("remote content is too large".into()));
            }
            bytes.extend_from_slice(&chunk);
        }
        return Ok((validated_url, mime, bytes));
    }
    unreachable!()
}

async fn client_for_url(url: reqwest::Url) -> Result<(reqwest::Client, reqwest::Url), AgentError> {
    if !matches!(url.scheme(), "http" | "https") || url.username() != "" || url.password().is_some()
    {
        return Err(AgentError::Media(
            "only credential-free HTTP(S) URLs are allowed".into(),
        ));
    }
    let host = url
        .host_str()
        .ok_or_else(|| AgentError::Media("URL has no host".into()))?;
    let port = url.port_or_known_default().unwrap_or(443);
    let addresses: Vec<SocketAddr> = tokio::net::lookup_host((host, port))
        .await
        .map_err(|error| AgentError::Media(format!("DNS lookup failed: {error}")))?
        .collect();
    if addresses.is_empty() || addresses.iter().any(|address| blocked_ip(address.ip())) {
        return Err(AgentError::Media(
            "private, local, or non-routable hosts are not allowed".into(),
        ));
    }
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(20))
        .resolve(host, addresses[0])
        .build()
        .map_err(|error| AgentError::Media(error.to_string()))?;
    Ok((client, url))
}

fn blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let octets = ip.octets();
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_broadcast()
                || ip.is_documentation()
                || ip.is_unspecified()
                || ip.is_multicast()
                || octets[0] == 0
                || (octets[0] == 100 && (64..=127).contains(&octets[1]))
                || (octets[0] == 198 && (octets[1] == 18 || octets[1] == 19))
                || octets[0] >= 240
        }
        IpAddr::V6(ip) => {
            let segments = ip.segments();
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_multicast()
                || ip.is_unique_local()
                || ip.is_unicast_link_local()
                || (segments[0] == 0x2001 && segments[1] == 0x0db8)
                || ip
                    .to_ipv4_mapped()
                    .is_some_and(|mapped| blocked_ip(mapped.into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssrf_guard_blocks_local_and_private_networks() {
        for address in [
            "127.0.0.1",
            "10.0.0.1",
            "172.16.0.1",
            "192.168.1.1",
            "169.254.169.254",
            "::1",
            "fc00::1",
            "fe80::1",
        ] {
            assert!(blocked_ip(address.parse().unwrap()), "{address}");
        }
        assert!(!blocked_ip("1.1.1.1".parse().unwrap()));
        assert!(!blocked_ip("2606:4700:4700::1111".parse().unwrap()));
    }

    #[tokio::test]
    async fn resolver_rejects_non_http_and_credentialed_urls() {
        for url in [
            "file:///etc/passwd",
            "data:text/plain,secret",
            "https://user:pass@example.com/image.png",
        ] {
            assert!(client_for_url(reqwest::Url::parse(url).unwrap())
                .await
                .is_err());
        }
    }
}
