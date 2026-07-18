// TJ-ARCH-MOB-001 compliant
//! Versioned seed/lookup bundles. Network retrieval stays in the shared Rust core.

use super::{RelationalError, RelationalResult};

#[derive(Debug, Clone)]
pub enum SeedSource {
    Bundled(&'static str),
    Http { url: String },
    Ipfs { cid: String, gateway: String },
}

#[derive(Debug, Clone)]
pub struct SeedBundle {
    pub name: String,
    pub version: u32,
    pub source: SeedSource,
}

impl SeedBundle {
    pub async fn sql(&self, client: &reqwest::Client) -> RelationalResult<String> {
        match &self.source {
            SeedSource::Bundled(sql) => Ok((*sql).to_owned()),
            SeedSource::Http { url } => self.fetch(client, url).await,
            SeedSource::Ipfs { cid, gateway } => {
                if cid.trim().is_empty() {
                    return Err(RelationalError::EmptyCid {
                        name: self.name.clone(),
                    });
                }
                let url = format!("{}/{cid}", gateway.trim_end_matches('/'));
                self.fetch(client, &url).await
            }
        }
    }

    async fn fetch(&self, client: &reqwest::Client, url: &str) -> RelationalResult<String> {
        let bytes = client
            .get(url)
            .header(
                reqwest::header::IF_NONE_MATCH,
                format!("\"{}-{}\"", self.name, self.version),
            )
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|source| RelationalError::SeedFetch {
                name: self.name.clone(),
                source,
            })?
            .bytes()
            .await
            .map_err(|source| RelationalError::SeedFetch {
                name: self.name.clone(),
                source,
            })?;
        std::str::from_utf8(&bytes)
            .map(str::to_owned)
            .map_err(|source| RelationalError::SeedEncoding {
                name: self.name.clone(),
                source,
            })
    }
}
