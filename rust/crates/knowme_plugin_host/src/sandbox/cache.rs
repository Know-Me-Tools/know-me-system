//! `.cwasm` precompiled-component cache (design D6).
//!
//! Cache entries are keyed by the SHA-256 of the component bytes. Engine
//! compatibility is enforced by `Component::deserialize` itself: the
//! `.cwasm` artifact format embeds a fingerprint of the compiler/engine
//! version and build flags, and deserialization fails on any mismatch — at
//! which point the entry is evicted and the component is recompiled from
//! source bytes (UAR `plugin_loader.rs`'s version-mismatch invalidation
//! pattern, adopted with the fingerprint check wasmtime already performs).

use std::path::PathBuf;

use sha2::Digest as _;
use wasmtime::component::Component;
use wasmtime::Engine;

use super::error::SandboxError;

/// sha256 hex digest of the component bytes — the cache key.
pub fn content_hash(bytes: &[u8]) -> String {
    let digest = sha2::Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for b in digest {
        use std::fmt::Write as _;
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// A directory-backed `.cwasm` cache. The directory is injected by the
/// caller (lifecycle resolves platform dirs), keeping the sandbox
/// platform-agnostic and unit-testable with tempdirs.
pub struct ComponentCache {
    dir: PathBuf,
}

impl ComponentCache {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn entry_path(&self, hash: &str) -> PathBuf {
        self.dir.join(format!("{hash}.cwasm"))
    }

    /// Load the component for `bytes`, using the cache when possible.
    /// Returns the component plus whether it came from the cache.
    pub fn load_or_compile(
        &self,
        engine: &Engine,
        bytes: &[u8],
    ) -> Result<(Component, bool), SandboxError> {
        let hash = content_hash(bytes);
        let path = self.entry_path(&hash);

        if let Ok(cwasm) = std::fs::read(&path) {
            // SAFETY: the artifact was written by this cache (same process,
            // same engine) and `deserialize` validates the embedded
            // engine/compiler fingerprint before any code is loaded —
            // mismatched or corrupted artifacts return an error, handled
            // below by eviction + recompile.
            match unsafe { Component::deserialize(engine, &cwasm) } {
                Ok(component) => return Ok((component, true)),
                Err(_) => {
                    // Stale (engine version/flags changed) or corrupted —
                    // invalidate and fall through to recompile.
                    let _ = std::fs::remove_file(&path);
                }
            }
        }

        let component = Component::new(engine, bytes)
            .map_err(|e| SandboxError::Compilation(format!("{e:#}")))?;

        // Best-effort cache population (atomic via tmp + rename). Failures
        // to write are not fatal to loading.
        if let Ok(cwasm) = component.serialize() {
            if std::fs::create_dir_all(&self.dir).is_ok() {
                let tmp = self.dir.join(format!(".{hash}.tmp"));
                if std::fs::write(&tmp, &cwasm).is_ok() {
                    let _ = std::fs::rename(&tmp, &path);
                }
            }
        }

        Ok((component, false))
    }

    /// Whether a cache entry exists for these bytes (test support).
    #[cfg(test)]
    pub fn contains(&self, bytes: &[u8]) -> bool {
        self.entry_path(&content_hash(bytes)).exists()
    }

    /// Path of the cache entry for these bytes (test support).
    #[cfg(test)]
    pub fn path_of(&self, bytes: &[u8]) -> PathBuf {
        self.entry_path(&content_hash(bytes))
    }
}

/// Convenience for hosts without a cache: compile straight from bytes.
pub fn compile_uncached(engine: &Engine, bytes: &[u8]) -> Result<Component, SandboxError> {
    Component::new(engine, bytes).map_err(|e| SandboxError::Compilation(format!("{e:#}")))
}

/// Test-only helper: the directory the cache writes into.
#[cfg(test)]
pub fn cache_dir(cache: &ComponentCache) -> &std::path::Path {
    &cache.dir
}
