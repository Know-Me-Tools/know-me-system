//! The wasmtime implementation of [`PluginHost`] (design D3, D5) and the
//! sandbox module root.

pub mod bindings;
pub mod cache;
pub mod error;
pub mod limits;
pub mod state;

#[cfg(test)]
mod e2e;
#[cfg(test)]
mod fixtures;

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

use crate::host::{InstanceConfig, InstanceId, LoadedComponent, PluginHost, WorldKind};
use crate::types::{A2uiAction, AgUiEvent, Contribution, ContributionInput};
use crate::SandboxLimits;

use bindings::{app_shell, contributor, hook, wire_a2ui_interface, wire_shared_interfaces};
use cache::ComponentCache;
use error::SandboxError;
use limits::fuel_seed;
use state::{GuestContext, HostCapability, PluginHostState};

/// Default epoch tick (design open question: 50 ms until on-device
/// measurement says otherwise).
const DEFAULT_EPOCH_TICK: Duration = Duration::from_millis(50);

/// A live guest instance: its store plus the typed per-world bindings.
struct Instance {
    store: Store<PluginHostState>,
    kind: InstanceKind,
    limits: SandboxLimits,
}

#[allow(clippy::large_enum_variant)]
enum InstanceKind {
    Hook(bindings::hook::Hook),
    Contributor(bindings::contributor::Contributor),
    AppShell(bindings::app_shell::AppShell),
}

/// Wasmtime-46 Component Model host for `knowme:plugin@1.0.0` components.
///
/// One process-wide engine; instances are stored behind per-instance async
/// mutexes so invokes on the same instance serialize while distinct
/// instances proceed concurrently. Wall-clock enforcement uses epoch
/// interruption: a ticker increments the engine epoch every
/// `epoch_tick` while at least one invocation is active (design's battery
/// note), and each invocation sets a deadline derived from its limits.
pub struct WasmtimePluginHost {
    engine: Engine,
    cache: Option<ComponentCache>,
    instances: Mutex<HashMap<InstanceId, Arc<tokio::sync::Mutex<Instance>>>>,
    next_instance: Mutex<u64>,
    active_invocations: Arc<AtomicUsize>,
    ticker: Mutex<Option<tokio::task::JoinHandle<()>>>,
    epoch_tick: Duration,
}

impl WasmtimePluginHost {
    /// New host. `cache_dir` enables the `.cwasm` precompiled cache
    /// (design D6); `None` compiles JIT every load.
    pub fn new(cache_dir: Option<std::path::PathBuf>) -> Result<Self, SandboxError> {
        let mut config = Config::new();
        // wasmtime 46: async support is unconditional (`Config::async_support`
        // is deprecated and has no effect); async instantiate/call APIs are
        // always available.
        config.consume_fuel(true);
        config.epoch_interruption(true);
        let engine = Engine::new(&config)
            .map_err(|e| SandboxError::Compilation(format!("engine config: {e:#}")))?;
        Ok(Self {
            engine,
            cache: cache_dir.map(ComponentCache::new),
            instances: Mutex::new(HashMap::new()),
            next_instance: Mutex::new(0),
            active_invocations: Arc::new(AtomicUsize::new(0)),
            ticker: Mutex::new(None),
            epoch_tick: DEFAULT_EPOCH_TICK,
        })
    }

    /// Test hook: shorter epoch tick for faster wall-clock tests.
    #[cfg(test)]
    pub fn with_epoch_tick(mut self, tick: Duration) -> Self {
        self.epoch_tick = tick;
        self
    }

    /// Start the epoch ticker on first use (needs an ambient tokio runtime).
    /// The ticker only increments the epoch while an invocation is active.
    fn ensure_ticker(&self) {
        let mut guard = self.ticker.lock().expect("ticker mutex poisoned");
        if guard.is_some() {
            return;
        }
        if tokio::runtime::Handle::try_current().is_err() {
            return;
        }
        let engine = self.engine.clone();
        let active = self.active_invocations.clone();
        let tick = self.epoch_tick;
        *guard = Some(tokio::spawn(async move {
            loop {
                tokio::time::sleep(tick).await;
                if active.load(Ordering::Relaxed) > 0 {
                    engine.increment_epoch();
                }
            }
        }));
    }

    /// Seed fuel, arm the epoch deadline, and reset the memory-denied flag
    /// for one invocation (or for instantiation, which also runs guest code).
    fn prepare_invocation(
        &self,
        store: &mut Store<PluginHostState>,
        limits: &SandboxLimits,
    ) -> Result<(), SandboxError> {
        self.ensure_ticker();
        store
            .set_fuel(fuel_seed(limits.fuel_limit))
            .map_err(|e| SandboxError::Execution(format!("set_fuel: {e:#}")))?;
        let tick_ms = self.epoch_tick.as_millis().max(1) as u64;
        let budget_ms = limits.max_event_duration_secs.saturating_mul(1000);
        let ticks = (budget_ms / tick_ms).max(1) + 1;
        store.set_epoch_deadline(ticks);
        store.data_mut().limiter.take_denied();
        Ok(())
    }

    /// Map a failed invocation to the spec-pinned taxonomy (design D5):
    /// memory ceiling → `MemoryExceeded`, fuel → `OutOfFuel`, wall-clock →
    /// `Timeout`, anything else → `Execution` with the trap message
    /// preserved.
    fn classify_failure(
        &self,
        store: &mut Store<PluginHostState>,
        trap: wasmtime::Error,
        started: Instant,
        limits: &SandboxLimits,
    ) -> SandboxError {
        if store.data_mut().limiter.take_denied() {
            return SandboxError::MemoryExceeded {
                limit_bytes: limits.max_linear_memory_bytes,
            };
        }
        if matches!(store.get_fuel(), Ok(0)) {
            return SandboxError::OutOfFuel;
        }
        let budget = Duration::from_secs(limits.max_event_duration_secs);
        let elapsed = started.elapsed();
        if elapsed >= budget {
            return SandboxError::Timeout {
                elapsed_ms: elapsed.as_millis() as u64,
                budget_ms: budget.as_millis() as u64,
            };
        }
        SandboxError::Execution(format!("{trap:#}"))
    }

    /// Post-invocation memory-ceiling check. The limiter records every
    /// rejection; a guest may recover gracefully from a failed allocation
    /// (e.g. `try_reserve`) and return Ok, so the check must run on every
    /// invocation outcome, not just traps (spec: memory ceiling →
    /// budget-exceeded naming the limit).
    fn post_limit_check(
        &self,
        store: &mut Store<PluginHostState>,
        limits: &SandboxLimits,
    ) -> Option<SandboxError> {
        if store.data_mut().limiter.take_denied() {
            Some(SandboxError::MemoryExceeded {
                limit_bytes: limits.max_linear_memory_bytes,
            })
        } else {
            None
        }
    }

    fn get_instance(
        &self,
        id: InstanceId,
    ) -> Result<Arc<tokio::sync::Mutex<Instance>>, SandboxError> {
        self.instances
            .lock()
            .expect("instances mutex poisoned")
            .get(&id)
            .cloned()
            .ok_or_else(|| SandboxError::Execution(format!("unknown instance id {}", id.0)))
    }

    /// Pre-instantiation import scan (task 6.2): every `knowme:plugin/*`
    /// import with a callable surface must be either ungated or explicitly
    /// granted — fail with `MissingCapability` naming the interface before
    /// any guest code runs. Types-only imports (e.g. `host-types`,
    /// `contribution-types` pulled in by export signatures) carry no
    /// callable surface, so there is nothing to gate. Non-`knowme:plugin`
    /// imports (WASI) are always provided by the deny-by-default contexts.
    fn scan_imports(
        &self,
        component: &Component,
        config: &InstanceConfig,
    ) -> Result<(), SandboxError> {
        for (name, item) in component.component_type().imports(&self.engine) {
            let Some(rest) = name.strip_prefix("knowme:plugin/") else {
                continue;
            };
            let interface = rest.split('@').next().unwrap_or(rest);

            // An import with no functions is types-only: skip gating.
            let has_callable_surface = match &item.ty {
                wasmtime::component::types::ComponentItem::ComponentInstance(instance) => {
                    instance.exports(&self.engine).any(|(_, export)| {
                        matches!(
                            export.ty,
                            wasmtime::component::types::ComponentItem::ComponentFunc(_)
                                | wasmtime::component::types::ComponentItem::CoreFunc(_)
                        )
                    })
                }
                wasmtime::component::types::ComponentItem::ComponentFunc(_)
                | wasmtime::component::types::ComponentItem::CoreFunc(_) => true,
                _ => false,
            };
            if !has_callable_surface {
                continue;
            }

            match HostCapability::from_interface_name(interface) {
                Some(cap) if cap.is_ungated() => {}
                Some(cap) if config.capabilities.contains(&cap) => {}
                // Unknown (newer WIT) or ungranted: deny by default.
                _ => return Err(SandboxError::MissingCapability(interface.to_string())),
            }
        }
        Ok(())
    }
}

impl Drop for WasmtimePluginHost {
    fn drop(&mut self) {
        if let Some(handle) = self.ticker.get_mut().expect("ticker mutex poisoned").take() {
            handle.abort();
        }
    }
}

/// RAII counter so the epoch ticker only ticks while an invocation runs.
struct ActiveGuard(Arc<AtomicUsize>);

impl ActiveGuard {
    fn new(counter: Arc<AtomicUsize>) -> Self {
        counter.fetch_add(1, Ordering::Relaxed);
        Self(counter)
    }
}

impl Drop for ActiveGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

#[async_trait]
impl PluginHost for WasmtimePluginHost {
    async fn load(&self, bytes: &[u8]) -> Result<LoadedComponent, SandboxError> {
        let (component, _from_cache) = match &self.cache {
            Some(cache) => cache.load_or_compile(&self.engine, bytes)?,
            None => (cache::compile_uncached(&self.engine, bytes)?, false),
        };
        Ok(LoadedComponent::new(component, cache::content_hash(bytes)))
    }

    async fn instantiate(
        &self,
        component: &LoadedComponent,
        config: InstanceConfig,
    ) -> Result<InstanceId, SandboxError> {
        config.world.ensure_stable()?;
        self.scan_imports(&component.component, &config)?;

        let limits = config.limits;
        let capabilities = config.capabilities.clone();
        let state = PluginHostState::new(GuestContext {
            plugin_id: config.plugin_id,
            capabilities: capabilities.clone(),
            dispatch: config.dispatch,
            limits,
        });
        let mut store = Store::new(&self.engine, state);
        store.limiter(|s| &mut s.limiter);
        // Instantiation and `init` run guest code: seed limits first.
        self.prepare_invocation(&mut store, &limits)?;

        let mut linker = Linker::new(&self.engine);
        let link_err = |e: wasmtime::Error| SandboxError::Instantiation(format!("{e:#}"));
        wasmtime_wasi::p2::add_to_linker_async(&mut linker).map_err(link_err)?;
        // Always-on deny-by-default HTTP context (design D4): bound without
        // an outbound handler so JS-style guests link even when unused.
        wasmtime_wasi_http::p2::add_only_http_to_linker_async(&mut linker).map_err(link_err)?;

        let component = &component.component;
        let kind = match config.world {
            WorldKind::Hook => {
                wire_shared_interfaces(&mut linker, &capabilities).map_err(link_err)?;
                let instance =
                    hook::Hook::instantiate_async(&mut store, component, &linker)
                        .await
                        .map_err(link_err)?;
                let init = instance
                    .knowme_plugin_lifecycle()
                    .call_init(&mut store)
                    .await
                    .map_err(|e| SandboxError::Instantiation(format!("lifecycle init: {e:#}")))?;
                init.map_err(|guest_err| {
                    SandboxError::Instantiation(format!("guest rejected init: {guest_err}"))
                })?;
                InstanceKind::Hook(instance)
            }
            WorldKind::Contributor => {
                wire_shared_interfaces(&mut linker, &capabilities).map_err(link_err)?;
                let instance = contributor::Contributor::instantiate_async(
                    &mut store,
                    component,
                    &linker,
                )
                .await
                .map_err(link_err)?;
                let init = instance
                    .knowme_plugin_lifecycle()
                    .call_init(&mut store)
                    .await
                    .map_err(|e| SandboxError::Instantiation(format!("lifecycle init: {e:#}")))?;
                init.map_err(|guest_err| {
                    SandboxError::Instantiation(format!("guest rejected init: {guest_err}"))
                })?;
                InstanceKind::Contributor(instance)
            }
            WorldKind::AppShell => {
                wire_shared_interfaces(&mut linker, &capabilities).map_err(link_err)?;
                wire_a2ui_interface(&mut linker, &capabilities).map_err(link_err)?;
                let instance =
                    app_shell::AppShell::instantiate_async(&mut store, component, &linker)
                        .await
                        .map_err(link_err)?;
                let init = instance
                    .knowme_plugin_lifecycle()
                    .call_init(&mut store)
                    .await
                    .map_err(|e| SandboxError::Instantiation(format!("lifecycle init: {e:#}")))?;
                init.map_err(|guest_err| {
                    SandboxError::Instantiation(format!("guest rejected init: {guest_err}"))
                })?;
                InstanceKind::AppShell(instance)
            }
            // ensure_stable() above guarantees this is unreachable.
            other => return Err(SandboxError::UnsupportedWorld(other.name().to_string())),
        };

        let id = {
            let mut next = self.next_instance.lock().expect("next_instance poisoned");
            let id = InstanceId(*next);
            *next += 1;
            id
        };
        self.instances
            .lock()
            .expect("instances mutex poisoned")
            .insert(
                id,
                Arc::new(tokio::sync::Mutex::new(Instance {
                    store,
                    kind,
                    limits,
                })),
            );
        Ok(id)
    }

    async fn invoke_hook(
        &self,
        id: InstanceId,
        topic: &str,
        event: AgUiEvent,
    ) -> Result<(), SandboxError> {
        let instance = self.get_instance(id)?;
        let mut guard = instance.lock().await;
        let Instance {
            store,
            kind,
            limits,
        } = &mut *guard;
        let limits = *limits;
        let InstanceKind::Hook(hook) = kind else {
            return Err(SandboxError::Execution(format!(
                "instance {} is not a hook-world instance",
                id.0
            )));
        };
        self.prepare_invocation(store, &limits)?;
        let started = Instant::now();
        let _active = ActiveGuard::new(self.active_invocations.clone());
        let result = hook.call_handle_event(&mut *store, topic, &event).await;
        if let Some(err) = self.post_limit_check(store, &limits) {
            return Err(err);
        }
        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(guest_err)) => Err(SandboxError::Execution(format!(
                "guest returned error: {guest_err}"
            ))),
            Err(trap) => Err(self.classify_failure(&mut *store, trap, started, &limits)),
        }
    }

    async fn invoke_contributor(
        &self,
        id: InstanceId,
        input: ContributionInput,
    ) -> Result<Contribution, SandboxError> {
        let instance = self.get_instance(id)?;
        let mut guard = instance.lock().await;
        let Instance {
            store,
            kind,
            limits,
        } = &mut *guard;
        let limits = *limits;
        let InstanceKind::Contributor(contributor) = kind else {
            return Err(SandboxError::Execution(format!(
                "instance {} is not a contributor-world instance",
                id.0
            )));
        };
        self.prepare_invocation(store, &limits)?;
        let started = Instant::now();
        let _active = ActiveGuard::new(self.active_invocations.clone());
        let result = contributor.call_contribute(&mut *store, &input).await;
        if let Some(err) = self.post_limit_check(store, &limits) {
            return Err(err);
        }
        match result {
            Ok(Ok(contribution)) => Ok(contribution),
            Ok(Err(guest_err)) => Err(SandboxError::Execution(format!(
                "guest returned error: {guest_err}"
            ))),
            Err(trap) => Err(self.classify_failure(&mut *store, trap, started, &limits)),
        }
    }

    async fn invoke_shell_init(&self, id: InstanceId) -> Result<(), SandboxError> {
        let instance = self.get_instance(id)?;
        let mut guard = instance.lock().await;
        let Instance {
            store,
            kind,
            limits,
        } = &mut *guard;
        let limits = *limits;
        let InstanceKind::AppShell(shell) = kind else {
            return Err(SandboxError::Execution(format!(
                "instance {} is not an app-shell-world instance",
                id.0
            )));
        };
        self.prepare_invocation(store, &limits)?;
        let started = Instant::now();
        let _active = ActiveGuard::new(self.active_invocations.clone());
        let result = shell.call_init_shell(&mut *store).await;
        if let Some(err) = self.post_limit_check(store, &limits) {
            return Err(err);
        }
        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(guest_err)) => Err(SandboxError::Execution(format!(
                "guest returned error: {guest_err}"
            ))),
            Err(trap) => Err(self.classify_failure(&mut *store, trap, started, &limits)),
        }
    }

    async fn invoke_shell_action(
        &self,
        id: InstanceId,
        action: A2uiAction,
    ) -> Result<(), SandboxError> {
        let instance = self.get_instance(id)?;
        let mut guard = instance.lock().await;
        let Instance {
            store,
            kind,
            limits,
        } = &mut *guard;
        let limits = *limits;
        let InstanceKind::AppShell(shell) = kind else {
            return Err(SandboxError::Execution(format!(
                "instance {} is not an app-shell-world instance",
                id.0
            )));
        };
        self.prepare_invocation(store, &limits)?;
        let started = Instant::now();
        let _active = ActiveGuard::new(self.active_invocations.clone());
        let result = shell.call_on_action(&mut *store, &action).await;
        if let Some(err) = self.post_limit_check(store, &limits) {
            return Err(err);
        }
        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(guest_err)) => Err(SandboxError::Execution(format!(
                "guest returned error: {guest_err}"
            ))),
            Err(trap) => Err(self.classify_failure(&mut *store, trap, started, &limits)),
        }
    }

    async fn unload(&self, id: InstanceId) -> Result<(), SandboxError> {
        let instance = self
            .instances
            .lock()
            .expect("instances mutex poisoned")
            .remove(&id)
            .ok_or_else(|| SandboxError::Execution(format!("unknown instance id {}", id.0)))?;
        let mut guard = instance.lock().await;
        // Best-effort lifecycle shutdown; a trapping shutdown must not
        // block teardown, so errors are deliberately discarded.
        let Instance { store, kind, .. } = &mut *guard;
        match kind {
            InstanceKind::Hook(hook) => {
                let _ = hook
                    .knowme_plugin_lifecycle()
                    .call_shutdown(&mut *store)
                    .await;
            }
            InstanceKind::Contributor(contributor) => {
                let _ = contributor
                    .knowme_plugin_lifecycle()
                    .call_shutdown(&mut *store)
                    .await;
            }
            InstanceKind::AppShell(shell) => {
                let _ = shell
                    .knowme_plugin_lifecycle()
                    .call_shutdown(&mut *store)
                    .await;
            }
        }
        Ok(())
    }
}
