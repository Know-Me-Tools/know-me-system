// TJ-ARCH-MOB-001 compliant
use std::net::SocketAddr;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use axum::body::Body;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::http::{Response, StatusCode, Uri};
use gen_ui_host::{AppServices, HostConfig};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "$KNOWME_EMBEDDED_WEB_DIR"]
struct EmbeddedWeb;

#[derive(Clone)]
enum WebAssets {
    Embedded,
    External(Arc<PathBuf>),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "knowme_web_server=info,gen_ui=info,tower_http=info".into()),
        )
        .init();
    gen_ui_runtime::init(None);
    gen_ui_runtime::handle().block_on(run())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = std::env::var_os("KNOWME_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir()?.join(".knowme-data"));
    // Browser-local chat uses WebLLM. The server host intentionally has no native
    // engine unless a deployment adds one; hosted providers are configured via BYOK.
    let services = AppServices::bootstrap(HostConfig::knowme(data_dir), None).await?;
    let assets = web_assets()?;
    let fallback_assets = assets.clone();
    let app = gen_ui_server_axum::router(services).fallback(move |uri: Uri| {
        let assets = fallback_assets.clone();
        async move { serve_web(uri, assets).await }
    });

    let address: SocketAddr = std::env::var("KNOWME_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string())
        .parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "KnowMe web server ready");
    axum::serve(listener, app).await?;
    Ok(())
}

fn web_assets() -> Result<WebAssets, Box<dyn std::error::Error>> {
    let Some(root) = std::env::var_os("KNOWME_WEB_ROOT") else {
        return Ok(WebAssets::Embedded);
    };
    let root = PathBuf::from(root).canonicalize()?;
    if !root.join("index.html").is_file() {
        return Err(format!(
            "KNOWME_WEB_ROOT must be a compiled site containing index.html: {}",
            root.display()
        )
        .into());
    }
    Ok(WebAssets::External(Arc::new(root)))
}

async fn serve_web(uri: Uri, assets: WebAssets) -> Response<Body> {
    if uri.path().starts_with("/api/") {
        return response(
            StatusCode::NOT_FOUND,
            "text/plain",
            b"API route not found".to_vec(),
            false,
        );
    }
    let requested = safe_asset_path(uri.path()).unwrap_or_else(|| "index.html".to_string());
    let asset = match &assets {
        WebAssets::Embedded => EmbeddedWeb::get(&requested)
            .or_else(|| {
                (!requested.contains('.'))
                    .then(|| EmbeddedWeb::get("index.html"))
                    .flatten()
            })
            .map(|file| file.data.into_owned()),
        WebAssets::External(root) => match read_external(root, &requested).await {
            some @ Some(_) => some,
            None if !requested.contains('.') => read_external(root, "index.html").await,
            None => None,
        },
    };
    let Some(bytes) = asset else {
        return response(
            StatusCode::NOT_FOUND,
            "text/plain",
            b"Not found".to_vec(),
            false,
        );
    };
    let served_path = if requested.contains('.') {
        requested.as_str()
    } else {
        "index.html"
    };
    let content_type = mime_guess::from_path(served_path)
        .first_or_octet_stream()
        .to_string();
    let immutable = served_path != "index.html" && served_path.starts_with("assets/");
    response(StatusCode::OK, &content_type, bytes, immutable)
}

async fn read_external(root: &Path, requested: &str) -> Option<Vec<u8>> {
    tokio::fs::read(root.join(requested)).await.ok()
}

fn safe_asset_path(path: &str) -> Option<String> {
    let trimmed = path.trim_start_matches('/');
    if trimmed.is_empty() {
        return Some("index.html".to_string());
    }
    let candidate = Path::new(trimmed);
    if candidate
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
    {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn response(
    status: StatusCode,
    content_type: &str,
    body: Vec<u8>,
    immutable: bool,
) -> Response<Body> {
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, content_type)
        .header(
            CACHE_CONTROL,
            if immutable {
                "public, max-age=31536000, immutable"
            } else {
                "no-cache"
            },
        )
        .body(Body::from(body))
        .expect("valid static response")
}
