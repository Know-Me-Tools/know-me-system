// TJ-ARCH-MOB-001 compliant
//! In-process `knowme_fetch` MCP server for the LOCAL inference lane: one
//! `fetch` tool mirroring the MCP reference fetch server's contract
//! (`url` / `max_length` / `start_index` / `raw`) so an on-device model can
//! read web content without a sidecar process (iOS forbids spawning one).
//!
//! Network access reuses [`crate::media`]'s hardened client — DNS
//! pre-resolution with private-IP blocking, pinned resolution, manual capped
//! redirects, and a byte cap — rather than re-deriving SSRF defenses here.
//!
//! The tool is deliberately annotated `openWorldHint: true`: the model chooses
//! the URL and the fetched page is untrusted input, so the local tool policy
//! always routes the call through user approval. Never mark this closed-world
//! to win auto-execution.
use async_trait::async_trait;
use ego_tree::iter::Edge;
use ego_tree::NodeId;
use gen_ui_mcp::{McpServerHandle, McpTransport};
use gen_ui_types::{CoreError, CoreResult};
use scraper::{Html, Node, Selector};

use crate::error::AgentError;
use crate::media::fetch_bounded;

/// Hard cap on bytes pulled off the network, before pagination.
const FETCH_MAX_BYTES: usize = 2 * 1024 * 1024;
/// Reference-server default page size, in characters.
const DEFAULT_MAX_LENGTH: usize = 5_000;
/// Upper bound on one page, in characters.
const MAX_MAX_LENGTH: usize = 1_000_000;

/// Subtrees that carry no readable content; their text is dropped entirely.
const EXCLUDED_ELEMENTS: &[&str] = &[
    "head", "script", "style", "noscript", "template", "svg", "iframe", "nav", "header", "footer",
    "aside", "form", "button",
];
/// Elements whose closing edge ends a line of extracted text.
const BLOCK_ELEMENTS: &[&str] = &[
    "p",
    "div",
    "section",
    "article",
    "main",
    "li",
    "ul",
    "ol",
    "table",
    "tr",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "br",
    "pre",
    "blockquote",
    "dt",
    "dd",
    "figcaption",
];

#[derive(Debug, serde::Deserialize)]
struct FetchArgs {
    url: String,
    #[serde(default = "default_max_length")]
    max_length: usize,
    #[serde(default)]
    start_index: usize,
    #[serde(default)]
    raw: bool,
}

fn default_max_length() -> usize {
    DEFAULT_MAX_LENGTH
}

async fn fetch_url(args: &FetchArgs) -> Result<String, AgentError> {
    let (final_url, mime, bytes) = fetch_bounded(&args.url, FETCH_MAX_BYTES).await?;
    let body = if args.raw {
        String::from_utf8_lossy(&bytes).into_owned()
    } else if matches!(mime.as_str(), "text/html" | "application/xhtml+xml") {
        extract_html_text(&String::from_utf8_lossy(&bytes))
    } else if mime.starts_with("text/")
        || mime == "application/json"
        || mime == "application/xml"
        || mime.ends_with("+json")
        || mime.ends_with("+xml")
    {
        String::from_utf8_lossy(&bytes).into_owned()
    } else {
        return Err(AgentError::Media(format!(
            "content type {mime} cannot be simplified to text; retry with raw=true"
        )));
    };
    let max_length = args.max_length.clamp(1, MAX_MAX_LENGTH);
    Ok(format!(
        "Contents of {final_url}:\n{}",
        paginate(&body, args.start_index, max_length)
    ))
}

/// Slice `content` on CHARACTER boundaries — a byte slice can split a
/// multi-byte character and panic. Mirrors the reference server's
/// truncation/continuation contract so models prompted on it behave the same.
fn paginate(content: &str, start_index: usize, max_length: usize) -> String {
    let total_chars = content.chars().count();
    if start_index >= total_chars {
        return "<error>No more content available.</error>".into();
    }
    let page: String = content.chars().skip(start_index).take(max_length).collect();
    let next_index = start_index + page.chars().count();
    if next_index < total_chars {
        format!(
            "{page}\n\n<error>Content truncated. Call the fetch tool with a start_index of {next_index} to get more content.</error>"
        )
    } else {
        page
    }
}

/// Reduce an HTML document to readable text: `# title`, then one line per
/// block element, with non-content subtrees (scripts, styles, chrome) dropped.
/// Intentionally simpler than a markdown converter — same spirit as the
/// reference server's "simplify" step.
fn extract_html_text(html: &str) -> String {
    let document = Html::parse_document(html);
    let title_selector = Selector::parse("title").expect("static selector parses");
    let title = document
        .select(&title_selector)
        .next()
        .map(|element| element.text().collect::<String>().trim().to_owned())
        .filter(|value| !value.is_empty());

    let mut text = String::new();
    // While set, every edge inside the excluded subtree is ignored until the
    // matching close edge is seen.
    let mut skipping: Option<NodeId> = None;
    for edge in document.root_element().traverse() {
        match edge {
            Edge::Open(node) => {
                if skipping.is_some() {
                    continue;
                }
                match node.value() {
                    Node::Element(element) if EXCLUDED_ELEMENTS.contains(&element.name()) => {
                        skipping = Some(node.id());
                    }
                    Node::Text(chunk) => {
                        let trimmed = chunk.trim();
                        if !trimmed.is_empty() {
                            if !text.is_empty() && !text.ends_with('\n') {
                                text.push(' ');
                            }
                            text.push_str(trimmed);
                        }
                    }
                    _ => {}
                }
            }
            Edge::Close(node) => {
                if skipping == Some(node.id()) {
                    skipping = None;
                    continue;
                }
                if skipping.is_some() {
                    continue;
                }
                if let Node::Element(element) = node.value() {
                    if BLOCK_ELEMENTS.contains(&element.name()) && !text.ends_with('\n') {
                        text.push('\n');
                    }
                }
            }
        }
    }

    let body = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    match (title, body.is_empty()) {
        (Some(title), true) => format!("# {title}"),
        (Some(title), false) => format!("# {title}\n\n{body}"),
        (None, _) => body,
    }
}

fn fetch_tools_list() -> serde_json::Value {
    serde_json::json!({
        "tools": [{
            "name": "fetch",
            "title": "Fetch URL",
            "description": "Fetch a URL from the internet and extract its contents as plain text (or the raw body when raw=true). Output is paginated: when a response ends with a truncation notice, call fetch again with the suggested start_index to continue reading.",
            "inputSchema": {
                "type": "object",
                "required": ["url"],
                "additionalProperties": false,
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The HTTP(S) URL to fetch."
                    },
                    "max_length": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": MAX_MAX_LENGTH,
                        "default": DEFAULT_MAX_LENGTH,
                        "description": "Maximum number of characters to return."
                    },
                    "start_index": {
                        "type": "integer",
                        "minimum": 0,
                        "default": 0,
                        "description": "Return output starting at this character index; use the value a previous truncated response suggested."
                    },
                    "raw": {
                        "type": "boolean",
                        "default": false,
                        "description": "Return the raw content without HTML simplification."
                    }
                }
            },
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": true
            }
        }]
    })
}

fn tool_error(message: String) -> serde_json::Value {
    serde_json::json!({
        "content": [{"type": "text", "text": message}],
        "isError": true
    })
}

struct KnowmeFetchTransport;

#[async_trait]
impl McpTransport for KnowmeFetchTransport {
    async fn request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> CoreResult<serde_json::Value> {
        match method {
            "tools/list" => Ok(fetch_tools_list()),
            "tools/call" => {
                let params = params.unwrap_or_default();
                let name = params
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                if name != "fetch" {
                    return Err(CoreError::Terminal(format!("unknown fetch tool '{name}'")));
                }
                let arguments = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));
                let args: FetchArgs = match serde_json::from_value(arguments) {
                    Ok(args) => args,
                    Err(error) => {
                        return Ok(tool_error(format!("invalid fetch arguments: {error}")))
                    }
                };
                match fetch_url(&args).await {
                    Ok(text) => Ok(serde_json::json!({
                        "content": [{"type": "text", "text": text}],
                        "isError": false
                    })),
                    // Fetch failures go back as tool content, not transport
                    // errors, so the model sees the reason (e.g. "retry with
                    // raw=true") instead of an opaque run failure.
                    Err(error) => Ok(tool_error(error.to_string())),
                }
            }
            other => Err(CoreError::Terminal(format!(
                "unsupported fetch MCP method '{other}'"
            ))),
        }
    }
}

/// Register the `knowme_fetch` MCP server on the process-lifetime registry.
/// Called from BOTH boot paths — mobile FFI boot and the desktop host — since
/// each platform boots the registry independently. Re-registration under the
/// same name replaces the previous handle, so this is idempotent per process.
pub async fn register_fetch_mcp() -> Result<(), AgentError> {
    let registry = crate::state::mcp()?;
    let handle = registry.register(McpServerHandle::new_trusted_local(
        "knowme_fetch",
        Box::new(KnowmeFetchTransport),
    ));
    handle.refresh_tools().await?;
    tracing::info!(server = %handle.name, "knowme fetch MCP tool registered");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gen_ui_mcp::McpTool;

    #[test]
    fn paginate_returns_full_content_when_it_fits() {
        assert_eq!(paginate("short body", 0, 100), "short body");
    }

    #[test]
    fn paginate_truncates_on_char_boundaries_and_points_at_next_index() {
        // Multi-byte characters: a byte-based slice would panic or corrupt.
        let content = "ααββγγ";
        let page = paginate(content, 2, 2);
        assert!(page.starts_with("ββ"), "page slices by character: {page}");
        assert!(
            page.contains("start_index of 4"),
            "continuation points at the next character index: {page}"
        );
    }

    #[test]
    fn paginate_past_end_reports_no_more_content() {
        assert_eq!(
            paginate("abc", 3, 10),
            "<error>No more content available.</error>"
        );
        assert_eq!(
            paginate("", 0, 10),
            "<error>No more content available.</error>"
        );
    }

    #[test]
    fn fetch_args_apply_reference_defaults() {
        let args: FetchArgs =
            serde_json::from_value(serde_json::json!({"url": "https://example.com"}))
                .expect("minimal args parse");
        assert_eq!(args.max_length, DEFAULT_MAX_LENGTH);
        assert_eq!(args.start_index, 0);
        assert!(!args.raw);
    }

    #[test]
    fn extract_html_text_keeps_title_and_content_but_strips_chrome_and_scripts() {
        let html = concat!(
            "<html><head><title>Doc Title</title><script>var x = 1;</script></head>",
            "<body><nav>Menu</nav><p>First para.</p>",
            "<p>Second <b>bold</b> para.</p><script>evil()</script>",
            "<footer>Copyright</footer></body></html>"
        );
        let text = extract_html_text(html);
        assert!(text.starts_with("# Doc Title"), "title leads: {text}");
        assert!(text.contains("First para."));
        assert!(text.contains("Second bold para."));
        assert!(
            !text.contains("var x"),
            "script text must be dropped: {text}"
        );
        assert!(!text.contains("evil"));
        assert!(!text.contains("Menu"), "nav chrome must be dropped: {text}");
        assert!(!text.contains("Copyright"));
    }

    #[tokio::test]
    async fn tools_list_declares_a_single_open_world_fetch_tool() {
        let value = KnowmeFetchTransport
            .request("tools/list", None)
            .await
            .expect("tools/list succeeds");
        let tools: Vec<McpTool> =
            serde_json::from_value(value["tools"].clone()).expect("tools parse as MCP tools");
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "fetch");
        let annotations = tools[0].annotations.clone().expect("annotations present");
        assert_eq!(annotations.read_only_hint, Some(true));
        assert_eq!(annotations.destructive_hint, Some(false));
        // Open-world keeps the tool behind user approval — load-bearing.
        assert_eq!(annotations.open_world_hint, Some(true));
    }

    #[tokio::test]
    async fn tools_call_rejects_unknown_tool_names() {
        let result = KnowmeFetchTransport
            .request(
                "tools/call",
                Some(serde_json::json!({"name": "nope", "arguments": {}})),
            )
            .await;
        assert!(result.is_err(), "unknown tools are a transport error");
    }

    #[tokio::test]
    async fn tools_call_returns_tool_error_content_for_invalid_arguments() {
        let value = KnowmeFetchTransport
            .request(
                "tools/call",
                Some(serde_json::json!({"name": "fetch", "arguments": {"url": 42}})),
            )
            .await
            .expect("argument errors surface as tool content");
        assert_eq!(value["isError"], true);
    }
}
