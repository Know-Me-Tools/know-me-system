//! The `PluginHost` public trait (spec: "PluginHost public trait") and its
//! supporting types. This is the crate's sole public runtime surface — later
//! changes (lifecycle, hook pipeline, app-shell) program against this trait
//! and never touch wasmtime directly.

use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;

use crate::dispatch::HostDispatch;
use crate::sandbox::error::SandboxError;
use crate::sandbox::state::HostCapability;
use crate::types::{A2uiAction, AgUiEvent, Contribution, ContributionInput};
use crate::SandboxLimits;

/// The manifest `worlds` vocabulary (change #1), mirrored for instantiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorldKind {
    Hook,
    Contributor,
    AppShell,
    Agent,
    Provider,
    Service,
    Workflow,
}

impl WorldKind {
    /// The world name as written in the WIT package and manifests.
    pub fn name(self) -> &'static str {
        match self {
            WorldKind::Hook => "hook",
            WorldKind::Contributor => "contributor",
            WorldKind::AppShell => "app-shell",
            WorldKind::Agent => "agent",
            WorldKind::Provider => "provider",
            WorldKind::Service => "service",
            WorldKind::Workflow => "workflow",
        }
    }

    /// Whether the world's WIT is stable enough to instantiate. The
    /// `unstable`-annotated worlds (`agent`, `provider`, `service`,
    /// `workflow`) are compile-checked only until their refining changes
    /// land (task 4.3).
    pub fn is_stable(self) -> bool {
        matches!(
            self,
            WorldKind::Hook | WorldKind::Contributor | WorldKind::AppShell
        )
    }

    /// Refuse instantiation of unstable worlds (spec scenario "Unstable
    /// world refused").
    pub fn ensure_stable(self) -> Result<(), SandboxError> {
        if self.is_stable() {
            Ok(())
        } else {
            Err(SandboxError::UnsupportedWorld(self.name().to_string()))
        }
    }
}

/// Opaque handle to a loaded (compiled) component. Cloning is cheap (the
/// underlying wasmtime component is reference-counted).
#[derive(Clone)]
pub struct LoadedComponent {
    pub(crate) component: wasmtime::component::Component,
    sha256: String,
}

impl LoadedComponent {
    pub(crate) fn new(component: wasmtime::component::Component, sha256: String) -> Self {
        Self { component, sha256 }
    }

    /// Content hash of the component bytes (hex-encoded SHA-256).
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

/// Opaque id of a live instance, returned by `instantiate` and consumed by
/// the invoke methods and `unload`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceId(pub u64);

/// Per-instantiation configuration.
pub struct InstanceConfig {
    /// Manifest id of the installed plugin. Own-namespace rules (KV,
    /// settings) are enforced against it.
    pub plugin_id: String,
    /// The world to instantiate for.
    pub world: WorldKind,
    /// Granted capabilities — exactly this set is wired into the linker
    /// (plus the ungated log/time interfaces).
    pub capabilities: HashSet<HostCapability>,
    /// The dispatch backend installed for this guest.
    pub dispatch: Arc<dyn HostDispatch>,
    /// Fuel / memory / wall-clock limits.
    pub limits: SandboxLimits,
}

/// The crate's sole public runtime surface (spec: "PluginHost public
/// trait"). Object-safe: `Arc<dyn PluginHost>` is the intended handle.
#[async_trait]
pub trait PluginHost: Send + Sync {
    /// Compile component bytes (through the `.cwasm` cache when configured).
    async fn load(&self, bytes: &[u8]) -> Result<LoadedComponent, SandboxError>;

    /// Instantiate a loaded component for a declared world and capability
    /// set. A component importing a host interface outside the set fails
    /// with `SandboxError::MissingCapability` before any guest code runs.
    /// The guest's `lifecycle.init` export is called before this returns.
    async fn instantiate(
        &self,
        component: &LoadedComponent,
        config: InstanceConfig,
    ) -> Result<InstanceId, SandboxError>;

    /// `hook.handle-event`.
    async fn invoke_hook(
        &self,
        id: InstanceId,
        topic: &str,
        event: AgUiEvent,
    ) -> Result<(), SandboxError>;

    /// `contributor.contribute`.
    async fn invoke_contributor(
        &self,
        id: InstanceId,
        input: ContributionInput,
    ) -> Result<Contribution, SandboxError>;

    /// `app-shell.init-shell`.
    async fn invoke_shell_init(&self, id: InstanceId) -> Result<(), SandboxError>;

    /// `app-shell.on-action`.
    async fn invoke_shell_action(
        &self,
        id: InstanceId,
        action: A2uiAction,
    ) -> Result<(), SandboxError>;

    /// Drop an instance (calling `lifecycle.shutdown` best-effort first).
    async fn unload(&self, id: InstanceId) -> Result<(), SandboxError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::error::SandboxError;

    /// Task 4.3: every unstable world name is refused with
    /// `SandboxError::UnsupportedWorld` naming the world.
    #[test]
    fn unstable_worlds_are_refused_by_name() {
        for (world, name) in [
            (WorldKind::Agent, "agent"),
            (WorldKind::Provider, "provider"),
            (WorldKind::Service, "service"),
            (WorldKind::Workflow, "workflow"),
        ] {
            let err = world.ensure_stable().expect_err("must be refused");
            assert_eq!(
                err,
                SandboxError::UnsupportedWorld(name.to_string()),
                "world {name}"
            );
            assert!(err.to_string().contains(name));
        }
        for world in [WorldKind::Hook, WorldKind::Contributor, WorldKind::AppShell] {
            world.ensure_stable().expect("stable worlds instantiate");
        }
    }

    /// Spec scenario: `Arc<dyn PluginHost>` compiles (object safety).
    #[test]
    fn trait_is_object_safe() {
        fn assert_object_safe(_: Arc<dyn PluginHost>) {}
        let _ = assert_object_safe;
    }
}
