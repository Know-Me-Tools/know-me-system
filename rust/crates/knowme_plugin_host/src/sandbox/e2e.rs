//! End-to-end sandbox tests (tasks 6.1–6.3, 7.1–7.3) against real fixture
//! components built to wasm32-wasip2. Each test notes the spec scenario it
//! pins (see `openspec/changes/plugin-sandbox-extraction/specs/`).

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use crate::dispatch::InMemoryDispatch;
use crate::host::{InstanceConfig, PluginHost, WorldKind};
use crate::sandbox::cache::{cache_dir, ComponentCache};
use crate::sandbox::error::SandboxError;
use crate::sandbox::fixtures::fixture_bytes;
use crate::sandbox::state::HostCapability;
use crate::sandbox::WasmtimePluginHost;
use crate::types::{
    AgUiEvent, ContributionItem, ContributionInput, PipelineStage, RunStarted,
};
use crate::SandboxLimits;

fn caps(set: &[HostCapability]) -> HashSet<HostCapability> {
    set.iter().copied().collect()
}

fn config(
    plugin_id: &str,
    world: WorldKind,
    capabilities: HashSet<HostCapability>,
    dispatch: Arc<InMemoryDispatch>,
    limits: SandboxLimits,
) -> InstanceConfig {
    InstanceConfig {
        plugin_id: plugin_id.to_string(),
        world,
        capabilities,
        dispatch,
        limits,
    }
}

fn test_event() -> AgUiEvent {
    AgUiEvent::RunStarted(RunStarted {
        thread_id: "t1".into(),
        run_id: "r1".into(),
    })
}

// ---------------------------------------------------------------------------
// Task 6.1 / spec "Precompiled component cache": second load hits cache;
// stale entry is invalidated and recompiled.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cache_second_load_hits_and_stale_entry_is_invalidated() {
    let bytes = fixture_bytes("knowme_fixture_hook");
    let tmp = tempfile::tempdir().expect("tempdir");
    let cache = ComponentCache::new(tmp.path().to_path_buf());
    let host = WasmtimePluginHost::new(None).expect("host");

    // First load: compile from source, populate the cache.
    let engine = wasmtime::Engine::new(&{
        let mut c = wasmtime::Config::new();
        c.consume_fuel(true);
        c.epoch_interruption(true);
        c
    })
    .expect("engine");
    let (_component, from_cache) = cache
        .load_or_compile(&engine, &bytes)
        .expect("first load");
    assert!(!from_cache, "first load must compile from source");
    assert!(cache.contains(&bytes), "cache populated");

    // Second load: cache hit (no recompilation).
    let (_component, from_cache) = cache
        .load_or_compile(&engine, &bytes)
        .expect("second load");
    assert!(from_cache, "second load must come from cache");

    // Stale/corrupt entry (simulating an engine-version mismatch, which
    // `Component::deserialize` rejects the same way): invalidated and
    // recompiled from source.
    let entry = cache.path_of(&bytes);
    std::fs::write(&entry, b"not a cwasm artifact").expect("corrupt");
    let (_component, from_cache) = cache
        .load_or_compile(&engine, &bytes)
        .expect("load after corruption");
    assert!(!from_cache, "stale entry must be recompiled from source");
    // ... and the entry was replaced with a valid artifact.
    let (_component, from_cache) = cache
        .load_or_compile(&engine, &bytes)
        .expect("third load");
    assert!(from_cache, "recompiled artifact cached again");

    drop(host);
    let _ = cache_dir(&cache); // exercised for completeness
}

// ---------------------------------------------------------------------------
// Task 7.1 / spec "Hook world end-to-end": log line captured, KV entry
// written, unload works.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn hook_world_end_to_end() {
    let bytes = fixture_bytes("knowme_fixture_hook");
    let host = WasmtimePluginHost::new(None).expect("host");
    let component = host.load(&bytes).await.expect("load");

    let dispatch = Arc::new(InMemoryDispatch::new());
    let id = host
        .instantiate(
            &component,
            config(
                "knowme.fixture.hook",
                WorldKind::Hook,
                caps(&[HostCapability::Storage, HostCapability::Events]),
                dispatch.clone(),
                SandboxLimits::default(),
            ),
        )
        .await
        .expect("instantiate");

    host.invoke_hook(id, "ok", test_event())
        .await
        .expect("invoke");

    let logs = dispatch.captured_logs();
    assert!(
        logs.iter().any(|(_, _, msg)| msg == "hook fixture received event"),
        "log sink must capture the guest line, got {logs:?}"
    );
    let kv = dispatch.kv_snapshot();
    assert_eq!(
        kv.get(&(
            "knowme.fixture.hook".to_string(),
            "knowme.fixture.hook".to_string(),
            "last-topic".to_string()
        )),
        Some(&b"ok".to_vec()),
        "KV backend must contain the guest entry"
    );

    host.unload(id).await.expect("unload");
    let err = host
        .invoke_hook(id, "ok", test_event())
        .await
        .expect_err("invoke after unload must fail");
    assert!(matches!(err, SandboxError::Execution(_)), "got: {err:?}");
}

// ---------------------------------------------------------------------------
// Task 6.2 / spec "Capability-gated instantiation": undeclared import fails
// at instantiation naming the interface; declared import succeeds and calls
// reach the backend.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn undeclared_host_net_import_fails_at_instantiation_naming_interface() {
    let bytes = fixture_bytes("knowme_fixture_net_importer");
    let host = WasmtimePluginHost::new(None).expect("host");
    let component = host.load(&bytes).await.expect("load");

    let dispatch = Arc::new(InMemoryDispatch::new());
    let err = host
        .instantiate(
            &component,
            config(
                "knowme.fixture.net",
                WorldKind::Hook,
                caps(&[HostCapability::Storage, HostCapability::Events]),
                dispatch,
                SandboxLimits::default(),
            ),
        )
        .await
        .expect_err("must fail without the net capability");
    assert_eq!(
        err,
        SandboxError::MissingCapability("host-net".to_string()),
        "the error names the interface"
    );
}

#[tokio::test]
async fn declared_host_net_import_succeeds_and_reaches_backend() {
    let bytes = fixture_bytes("knowme_fixture_net_importer");
    let host = WasmtimePluginHost::new(None).expect("host");
    let component = host.load(&bytes).await.expect("load");

    let dispatch = Arc::new(InMemoryDispatch::new());
    let id = host
        .instantiate(
            &component,
            config(
                "knowme.fixture.net",
                WorldKind::Hook,
                caps(&[
                    HostCapability::Storage,
                    HostCapability::Events,
                    HostCapability::Net,
                ]),
                dispatch.clone(),
                SandboxLimits::default(),
            ),
        )
        .await
        .expect("instantiate with net granted");

    // Public URL: passes the SSRF predicate, denied by the not-installed
    // backend — the guest observes the typed host-error and logs it.
    host.invoke_hook(id, "fetch:example", test_event())
        .await
        .expect("invoke fetch:example");
    // Cloud metadata URL: ssrf-denied at the boundary's backend.
    host.invoke_hook(id, "fetch:metadata", test_event())
        .await
        .expect("invoke fetch:metadata");

    let logs = dispatch.captured_logs();
    assert!(
        logs.iter()
            .any(|(_, _, msg)| msg.contains("backend not installed")),
        "public URL must reach the not-installed backend denial, got {logs:?}"
    );
    assert!(
        logs.iter()
            .any(|(_, _, msg)| msg.contains("SsrfDenied") && msg.contains("169.254.169.254")),
        "metadata URL must be ssrf-denied, got {logs:?}"
    );
}

// ---------------------------------------------------------------------------
// Task 6.3 / spec "Resource limits": fuel exhaustion, memory ceiling,
// wall-clock timeout — with the pinned error taxonomy.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn fuel_exhaustion_maps_to_budget_exceeded_and_host_survives() {
    let bytes = fixture_bytes("knowme_fixture_hook");
    let host = WasmtimePluginHost::new(None).expect("host");
    let component = host.load(&bytes).await.expect("load");

    let dispatch = Arc::new(InMemoryDispatch::new());
    let mut limits = SandboxLimits::default();
    limits.fuel_limit = 1_000_000;
    let id = host
        .instantiate(
            &component,
            config(
                "knowme.fixture.hook",
                WorldKind::Hook,
                caps(&[HostCapability::Storage, HostCapability::Events]),
                dispatch,
                limits,
            ),
        )
        .await
        .expect("instantiate");

    let err = host
        .invoke_hook(id, "spin", test_event())
        .await
        .expect_err("infinite loop must exhaust fuel");
    assert_eq!(err, SandboxError::OutOfFuel);
    // Spec-pinned lift into host-error.
    assert!(matches!(
        err.to_host_error(),
        crate::sandbox::error::HostError::BudgetExceeded(_)
    ));

    // The host remains able to instantiate and invoke other components.
    let contributor = fixture_bytes("knowme_fixture_contributor");
    let component = host.load(&contributor).await.expect("load contributor");
    let dispatch = Arc::new(InMemoryDispatch::new());
    let id = host
        .instantiate(
            &component,
            config(
                "knowme.fixture.contributor",
                WorldKind::Contributor,
                caps(&[]),
                dispatch,
                SandboxLimits::default(),
            ),
        )
        .await
        .expect("host still usable after fuel exhaustion");
    host.invoke_contributor(
        id,
        ContributionInput {
            run_id: "r1".into(),
            stage: PipelineStage::ContextAssemble,
            items: vec![],
        },
    )
    .await
    .expect("other component invokes fine");
}

#[tokio::test]
async fn memory_ceiling_maps_to_budget_exceeded_naming_the_limit() {
    let bytes = fixture_bytes("knowme_fixture_hook");
    let host = WasmtimePluginHost::new(None).expect("host");
    let component = host.load(&bytes).await.expect("load");

    let dispatch = Arc::new(InMemoryDispatch::new());
    let id = host
        .instantiate(
            &component,
            config(
                "knowme.fixture.hook",
                WorldKind::Hook,
                caps(&[HostCapability::Storage, HostCapability::Events]),
                dispatch,
                SandboxLimits::default(),
            ),
        )
        .await
        .expect("instantiate");

    let err = host
        .invoke_hook(id, "mem", test_event())
        .await
        .expect_err("growth past the ceiling must be denied");
    assert_eq!(
        err,
        SandboxError::MemoryExceeded {
            limit_bytes: 64 * 1024 * 1024
        }
    );
    let lifted = err.to_host_error();
    assert!(
        matches!(lifted, crate::sandbox::error::HostError::BudgetExceeded(ref s) if s.contains("67108864")),
        "budget-exceeded names the memory limit, got {lifted:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn wall_clock_timeout_maps_to_timeout() {
    let bytes = fixture_bytes("knowme_fixture_hook");
    let host = WasmtimePluginHost::new(None)
        .expect("host")
        .with_epoch_tick(Duration::from_millis(20));
    let component = host.load(&bytes).await.expect("load");

    let dispatch = Arc::new(InMemoryDispatch::new());
    let mut limits = SandboxLimits::default();
    limits.max_event_duration_secs = 1;
    let id = host
        .instantiate(
            &component,
            config(
                "knowme.fixture.hook",
                WorldKind::Hook,
                caps(&[HostCapability::Storage, HostCapability::Events]),
                dispatch,
                limits,
            ),
        )
        .await
        .expect("instantiate");

    let started = std::time::Instant::now();
    let err = host
        .invoke_hook(id, "sleep", test_event())
        .await
        .expect_err("sleeping guest must hit the wall-clock budget");
    let elapsed = started.elapsed();
    assert!(
        matches!(err, SandboxError::Timeout { budget_ms: 1000, .. }),
        "got: {err:?}"
    );
    assert!(
        elapsed < Duration::from_secs(4),
        "timeout must fire within budget + one epoch tick-ish, took {elapsed:?}"
    );
    assert_eq!(
        err.to_host_error(),
        crate::sandbox::error::HostError::Timeout
    );
}

// ---------------------------------------------------------------------------
// Task 7.2 / spec "Contributor world end-to-end": typed contribution with
// stage, items, and provenance intact.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contributor_world_end_to_end() {
    let bytes = fixture_bytes("knowme_fixture_contributor");
    let host = WasmtimePluginHost::new(None).expect("host");
    let component = host.load(&bytes).await.expect("load");

    let dispatch = Arc::new(InMemoryDispatch::new());
    let id = host
        .instantiate(
            &component,
            config(
                "knowme.fixture.contributor",
                WorldKind::Contributor,
                caps(&[]),
                dispatch,
                SandboxLimits::default(),
            ),
        )
        .await
        .expect("instantiate");

    let contribution = host
        .invoke_contributor(
            id,
            ContributionInput {
                run_id: "run-42".into(),
                stage: PipelineStage::ContextAssemble,
                items: vec![],
            },
        )
        .await
        .expect("contribute");

    assert_eq!(contribution.items.len(), 1);
    match &contribution.items[0] {
        ContributionItem::Custom(item) => {
            assert_eq!(item.source, "knowme.fixture.contributor");
            assert_eq!(item.name, "system-prompt-fragment");
            assert!(
                item.payload_json.contains("ContextAssemble"),
                "stage must round-trip into the payload: {}",
                item.payload_json
            );
            assert!(item.payload_json.contains("concise assistant"));
        }
        other => panic!("expected a custom contribution item, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Task 7.3 / spec "JS-style guest links without using HTTP" and "Unstable
// world refused".
// ---------------------------------------------------------------------------

#[tokio::test]
async fn http_types_guest_instantiates_under_deny_by_default_http_context() {
    let bytes = fixture_bytes("knowme_fixture_http_types");
    let host = WasmtimePluginHost::new(None).expect("host");
    let component = host.load(&bytes).await.expect("load");

    let dispatch = Arc::new(InMemoryDispatch::new());
    let id = host
        .instantiate(
            &component,
            config(
                "knowme.fixture.http-types",
                WorldKind::Hook,
                caps(&[HostCapability::Events]),
                dispatch.clone(),
                SandboxLimits::default(),
            ),
        )
        .await
        .expect("guest importing wasi:http/types must instantiate");

    host.invoke_hook(id, "ok", test_event())
        .await
        .expect("Fields roundtrip succeeds without any outbound handler");
    let logs = dispatch.captured_logs();
    assert!(
        logs.iter()
            .any(|(_, _, msg)| msg.contains("Fields roundtrip ok")),
        "got {logs:?}"
    );
}

#[tokio::test]
async fn unstable_world_is_refused_at_instantiation() {
    let bytes = fixture_bytes("knowme_fixture_hook");
    let host = WasmtimePluginHost::new(None).expect("host");
    let component = host.load(&bytes).await.expect("load");

    let dispatch = Arc::new(InMemoryDispatch::new());
    let err = host
        .instantiate(
            &component,
            config(
                "knowme.fixture.hook",
                WorldKind::Workflow,
                caps(&[]),
                dispatch,
                SandboxLimits::default(),
            ),
        )
        .await
        .expect_err("workflow must be refused");
    assert_eq!(
        err,
        SandboxError::UnsupportedWorld("workflow".to_string())
    );
}

#[tokio::test]
async fn invoke_on_mismatched_world_is_a_typed_error() {
    let bytes = fixture_bytes("knowme_fixture_contributor");
    let host = WasmtimePluginHost::new(None).expect("host");
    let component = host.load(&bytes).await.expect("load");

    let dispatch = Arc::new(InMemoryDispatch::new());
    let id = host
        .instantiate(
            &component,
            config(
                "knowme.fixture.contributor",
                WorldKind::Contributor,
                caps(&[]),
                dispatch,
                SandboxLimits::default(),
            ),
        )
        .await
        .expect("instantiate");

    let err = host
        .invoke_hook(id, "ok", test_event())
        .await
        .expect_err("hook invoke on a contributor instance must fail");
    assert!(matches!(err, SandboxError::Execution(_)), "got: {err:?}");
}
