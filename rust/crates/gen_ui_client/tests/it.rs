// TJ-ARCH-MOB-001 compliant
//! Boundary tests for the flint client. No network, no mocks — the observable
//! behavior is: (1) a gate JWT decodes to the right role/tenant and expiry drives the
//! AuthState machine; (2) forge AG-UI SSE frames fold into the ContentBlock contract.
#![cfg(not(target_arch = "wasm32"))]

use gen_ui_client::flint::forge::{agui_to_a2ui, parse_agui_frame, AgUiEvent};
use gen_ui_client::flint::token::{AuthState, Role, Token};
use gen_ui_types::content_block::ContentBlock;
use gen_ui_types::events::A2uiEvent;

/// Mint an unsigned-payload HS256 JWT for the given claims (test-only; the client
/// decodes WITHOUT verification, mirroring the real "gate/forge own verification").
fn mint(claims: serde_json::Value) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"test"),
    )
    .expect("encode")
}

#[test]
fn token_decodes_role_and_tenant_from_gate_claims() {
    // Mirrors forge-identity::Claims: typed sub/role/tenant_id + untyped extra.
    let jwt = mint(serde_json::json!({
        "sub": "user-42", "role": "agent", "tenant_id": "acme",
        "exp": 9_999_999_999i64, "agent_id": "planner", "act": "user-42",
    }));
    let token = Token::parse(&jwt).expect("parse");
    assert_eq!(token.role(), Role::Agent);
    assert_eq!(token.claims.tenant_id.as_deref(), Some("acme"));
    // `act`/`agent_id` are untyped per the platform contract — read from `extra`.
    assert_eq!(token.claims.extra_str("agent_id"), Some("planner"));
    assert_eq!(token.claims.extra_str("act"), Some("user-42"));
}

#[test]
fn absent_role_coerces_to_anon() {
    let jwt = mint(serde_json::json!({ "sub": "x", "exp": 9_999_999_999i64 }));
    let token = Token::parse(&jwt).expect("parse");
    assert_eq!(token.role(), Role::Anon);
}

#[test]
fn auth_state_machine_tracks_bearer_and_refresh() {
    let fresh =
        mint(serde_json::json!({ "sub": "a", "role": "authenticated", "exp": 9_999_999_999i64 }));
    let state = AuthState::Authenticated {
        token: Token::parse(&fresh).unwrap(),
    };
    assert_eq!(state.role(), Role::Authenticated);
    assert!(state.bearer().is_some());
    assert!(!state.needs_refresh(1_000)); // far from exp
    assert!(state.needs_refresh(9_999_999_999)); // at/after exp (with skew)

    // Unauthenticated carries no bearer and is never "refreshable".
    let empty = AuthState::Unauthenticated;
    assert!(empty.bearer().is_none());
    assert!(!empty.needs_refresh(9_999_999_999));
}

#[test]
fn agui_text_delta_folds_to_content_block_text() {
    let frame = r#"{"type":"TextMessageContent","delta":"hello"}"#;
    let events = parse_agui_frame(frame);
    assert_eq!(events.len(), 1);
    match &events[0] {
        A2uiEvent::Block {
            block: ContentBlock::Text { text },
        } => assert_eq!(text, "hello"),
        other => panic!("expected Text block, got {other:?}"),
    }
}

#[test]
fn agui_run_lifecycle_and_toolcall_map_to_a2ui() {
    // RunStarted → A2uiEvent::RunStarted.
    let started = agui_to_a2ui(&AgUiEvent::RunStarted {
        run_id: "r1".into(),
    });
    assert!(matches!(started.as_slice(), [A2uiEvent::RunStarted { run_id }] if run_id == "r1"));

    // ToolCallStart → a ToolUse ContentBlock (name + id preserved).
    let tool =
        parse_agui_frame(r#"{"type":"ToolCallStart","tool_call_id":"t9","tool_name":"search"}"#);
    match tool.as_slice() {
        [A2uiEvent::Block {
            block: ContentBlock::ToolUse { id, name, .. },
        }] => {
            assert_eq!(id, "t9");
            assert_eq!(name, "search");
        }
        other => panic!("expected ToolUse block, got {other:?}"),
    }

    // Unknown/keepalive frames yield nothing rather than erroring the stream.
    assert!(parse_agui_frame(":keep-alive").is_empty());
    assert!(parse_agui_frame(r#"{"type":"StateDelta","delta":[]}"#).is_empty());
}
