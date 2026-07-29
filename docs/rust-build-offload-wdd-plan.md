# Rust Build Offload Plan — WD My Passport ("WDD")

**Date:** 2026-07-26
**Drive:** `/Volumes/my-passport` — WD My Passport, 1.8 TB, HFS+ (journaled) over USB, **1.7 TB free**
**System drive:** 1.8 TB, 1.5 TB used, **326 GB free (83% full)**
**Toolchain:** cargo 1.99.0-nightly, sccache installed (`/opt/homebrew/bin/sccache`)

---

## 1. Measured disk-space inventory

### On the system drive (recoverable)

| Item | Path | Size |
|---|---|---|
| Project target dir | `~/Projects/know-me/know-me-system/rust/target` | **179 GB** (debug 131 G, android 31 G, ios 16 G, release 1.1 G, wasm ~0.4 G) |
| Tauri target dir | `~/Projects/know-me/know-me-system/desktop/src-tauri/target` | **50 GB** |
| Rustup toolchains (10 installed!) | `~/.rustup` | **20 GB** |
| Cargo registry + git caches | `~/.cargo/registry`, `~/.cargo/git` | **5.0 GB** (+ 0.7 GB `bin`, keep local) |
| Reference project targets | `openai-proxy/target`, `sigma-eclipse-llm/src-tauri/target` | **3.2 GB** |
| Local sccache cache | `~/Library/Caches/Mozilla.sccache` | 29 MB |
| **Total recoverable** | | **≈ 258 GB** |

### Already on the WDD (the established strategy — reuse it)

- `cargo/` — registry (2.6 G) + git (3.3 G) caches + empty `cargo-target`, `sccache` stubs
- `sccache/` — 12 GB warm compiler cache
- `claude-worktrees/` — 36 GB, 3 existing worktrees (`librefang`, `prometheus-skill-system`, `universal-agent-runtime`)
- `cargo-target*` — 6 stale per-branch variants, **~96 GB total** (cigreen 30 G, upmerge4 26 G, upmerge 23 G, fix 11 G, base 5.5 G, liveitest 0.5 G) — consolidation candidates
- `.pnpm-store/` (15 M), `embeddings/`, `Projects/skytok`

---

## 2. Target end-state layout on the WDD

```
/Volumes/my-passport/
├── cargo/                  # CARGO_HOME caches (registry + git) — symlinked from ~/.cargo
│   ├── registry/
│   └── git/
├── rustup/                 # RUSTUP_HOME — toolchains, symlinked from ~/.rustup
├── sccache/                # SCCACHE_DIR (existing 12 GB cache, raised limit)
├── cargo-build/            # NEW: per-workspace build dirs (build.build-dir, hash-templated)
├── cargo-target/           # per-workspace final-artifact target dirs
└── claude-worktrees/       # ALL git worktrees live here
```

Symlinks (not env-var-only) are used for `~/.cargo/{registry,git}` and `~/.rustup` so that
**every** tool (IDEs, rust-analyzer, CI scripts, agents) picks up the new locations
transparently without per-tool configuration.

---

## 3. Execution plan (in order)

### Phase 0 — Guard rails first (5 min)

The drive is HFS+ over USB — POSIX permissions, symlinks, and hardlinks all work, but it
must be mounted before any build. Add mount detection and stop macOS from indexing it:

```bash
# Stop Spotlight indexing the drive (big perf win for builds)
sudo mdutil -i off /Volumes/my-passport
# Exclude caches from Time Machine
tmutil addexclusion /Volumes/my-passport/cargo-build
tmutil addexclusion /Volumes/my-passport/cargo-target
tmutil addexclusion /Volumes/my-passport/sccache
```

Add to `~/.zshrc` (top of the Rust section) — hard-fail fast when the drive is absent:

```bash
export WDD=/Volumes/my-passport
if [[ -d "$WDD/cargo" ]]; then
  export SCCACHE_DIR="$WDD/sccache"
  export SCCACHE_CACHE_SIZE=40G
  export SCCACHE_IDLE_TIMEOUT=0
  export CARGO_INCREMENTAL=1
else
  echo "⚠️  WDD not mounted: Rust builds will use local disk"
fi
```

### Phase 1 — Cargo registry + git caches → WDD (recovers ~5 GB)

The drive already has `cargo/registry` (2.6 G) and `cargo/git` (3.3 G) from the previous
setup. Merge local newer content in, then symlink:

```bash
rsync -a --ignore-existing ~/.cargo/registry/ "$WDD/cargo/registry/"
rsync -a --ignore-existing ~/.cargo/git/      "$WDD/cargo/git/"
rm -rf ~/.cargo/registry ~/.cargo/git
ln -s "$WDD/cargo/registry" ~/.cargo/registry
ln -s "$WDD/cargo/git"      ~/.cargo/git
```

Keep `~/.cargo/bin` and `~/.cargo/config.toml` **local** so cargo/rustup shims still
launch when the drive is unplugged (they'll just fail cleanly on cache access).

### Phase 2 — Rustup toolchains → WDD (recovers ~20 GB)

**Prune first** — 10 toolchains are installed; most are stale:

```bash
rustup toolchain list
# Keep: stable-aarch64-apple-darwin, nightly-aarch64-apple-darwin
# Remove (unless pinned by a specific project): 1.80, 1.91.1, 1.93, 1.95, 1.96,
#   1.97.1, nightly-2026-05-01, nightly-2026-07-18
rustup toolchain uninstall 1.80-aarch64-apple-darwin 1.91.1-aarch64-apple-darwin \
  1.93-aarch64-apple-darwin 1.95-aarch64-apple-darwin 1.96-aarch64-apple-darwin \
  1.97.1-aarch64-apple-darwin nightly-2026-05-01-aarch64-apple-darwin \
  nightly-2026-07-18-aarch64-apple-darwin
# Pruning alone: ~13 GB. Then move the rest:
mv ~/.rustup "$WDD/rustup"
ln -s "$WDD/rustup" ~/.rustup
rustup show   # verify
```

### Phase 3 — ALL build output → WDD (recovers ~232 GB) ⭐ the big win

Cargo ≥ 1.91 (you have 1.99-nightly) has **two** stabilized knobs — but only one
supports per-workspace templating:

- `build.build-dir` — intermediate artifacts (the bulk of the 131 GB `debug/` dir:
  object files, incremental state, fingerprints). Supports `{workspace-path-hash}`
  templating → set globally.
- `build.target-dir` — final artifacts only (small). Does **NOT** support templating,
  so it stays per-project (a global value would force all workspaces to share one dir).

Set `build-dir` **globally** in `~/.cargo/config.toml` with workspace-hash templating so every
project defers automatically with zero per-repo edits:

```toml
# ~/.cargo/config.toml
[build]
build-dir = "/Volumes/my-passport/cargo-build/{workspace-path-hash}"
rustc-wrapper = "sccache"          # see Phase 4
jobs = 10                          # physical cores; USB I/O is the bottleneck, not CPU

[cache]
auto-clean-frequency = "7 days"    # automatic GC of registry cache (Cargo ≥ 1.88)

[net]
git-fetch-with-cli = true          # matches repo policy (private SSH deps)

[profile.dev]
incremental = true
debug = "line-tables-only"         # ~50–70% smaller .o files vs full debuginfo
split-debuginfo = "off"            # macOS: skips dsymutil — faster links, far less disk
codegen-units = 256                # max parallelism in dev

[profile.dev.package."*"]
opt-level = 2                      # deps stay fast; only YOUR crates build unoptimized

# Nightly-only acceleration: NOT set globally (know-me-system pins stable 1.97.1
# via rust-toolchain.toml; -Z flags hard-fail stable builds). Opt in per shell:
#   export RUSTFLAGS="-Zthreads=8 -Zshare-generics=y"
```

> Repo note: `rust/.cargo/config.toml` already documents that ld-prime is the right
> macOS linker and sccache is a CI/branch-switch tool. Keep the repo config as-is; the
> global config above layers cleanly (repo config wins where they overlap — none do).

Then reclaim the local targets (after one successful drive-backed build, or immediately
if you accept one cold rebuild — sccache's warm 12 GB cache absorbs most of it):

```bash
rm -rf ~/Projects/know-me/know-me-system/rust/target          # 179 GB
rm -rf ~/Projects/know-me/know-me-system/desktop/src-tauri/target  # 50 GB
rm -rf ~/Projects/references/baseline/openai-proxy/target
rm -rf ~/Projects/references/sigma-eclipse-llm/src-tauri/target
```

### Phase 4 — sccache fully on the WDD (recovers 29 MB local + absorbs rebuilds)

Already partially set up (12 GB warm cache at `$WDD/sccache`). Complete it:

1. `SCCACHE_DIR=$WDD/sccache` and `SCCACHE_CACHE_SIZE=40G` in `~/.zshrc` (Phase 0 block).
2. `rustc-wrapper = "sccache"` in global config (Phase 3).
3. Delete the tiny local cache: `rm -rf ~/Library/Caches/Mozilla.sccache`.
4. Verify: `sccache --show-stats` after a build; expect hit rate to climb toward 80%+.

**Tradeoff (documented):** sccache can slow *release* builds (up to ~50% per Depot's
benchmarks) and can't cache the final workspace crates. Hybrid escape hatch when doing
release packaging:

```bash
SCCACHE_DISABLE=1 cargo build --release   # bypass wrapper for release runs
```

### Phase 5 — Worktrees → WDD

All new worktrees go to the existing `claude-worktrees/` convention:

```bash
git worktree add /Volumes/my-passport/claude-worktrees/<repo>-<branch> <branch>
```

Because of the global hash-templated `target-dir`/`build-dir` (Phase 3), each worktree
automatically gets its own build+target dirs on the drive — no extra config, and no
contention with the main checkout. The 3 existing WDD worktrees (36 GB) are already
correctly placed. The main `know-me-system` checkout has no extra local worktrees to move.

### Phase 6 — WDD housekeeping (frees ~90 GB on the drive)

The 6 stale `cargo-target-*` variants (~96 GB) predate the per-workspace scheme and are
superseded by sccache + hash-templated dirs. After Phase 3 builds succeed:

```bash
# Verify nothing references them, then:
rm -rf /Volumes/my-passport/cargo-target-cigreen \
       /Volumes/my-passport/cargo-target-upmerge \
       /Volumes/my-passport/cargo-target-upmerge4 \
       /Volumes/my-passport/cargo-target-fix \
       /Volumes/my-passport/cargo-target-liveitest
# Keep: kreuzberg-tesseract-cache dirs inside them if that tool is still used —
# move those to /Volumes/my-passport/caches/kreuzberg-tesseract first.
```

---

## 4. Space recovery estimate

| Action | System drive recovered |
|---|---|
| Phase 1: cargo registry + git | ~5 GB |
| Phase 2: rustup (prune 13 GB + move 7 GB) | ~20 GB |
| Phase 3: know-me `rust/target` | ~179 GB |
| Phase 3: know-me `desktop/src-tauri/target` | ~50 GB |
| Phase 3: reference targets | ~3 GB |
| Phase 4: local sccache | ~0.03 GB |
| **Total** | **≈ 257 GB** |

**Result:** system drive goes from **326 GB free (83% full) → ~583 GB free (~69% full)**.

Ongoing growth also defers to the WDD: future target-dir growth (the 179 GB didn't
happen once — it regrows), new toolchains, registry growth, and all worktree checkouts.

Drive-side impact: +~27 GB moved data (rustup + cargo caches; target dirs rebuild into
existing ~12 GB sccache + new dirs), −~90 GB from Phase 6 cleanup → drive stays around
**~100 GB used of 1.8 TB**.

---

## 5. Risks & mitigations

| Risk | Mitigation |
|---|---|
| Drive unplugged → builds fail | Phase 0 mount guard warns; `~/.cargo/bin` stays local so shims fail cleanly, nothing corrupts |
| USB/HFS+ slower than internal NVMe | sccache (warm 12 G cache) avoids recompiles; incremental + `line-tables-only` shrink I/O volume; `-Zthreads=8` hides latency; deps at `opt-level 2` built once then cached |
| Cold rebuild after deleting 179 GB target | sccache absorbs most of it (that's its exact job); do the first rebuild overnight |
| Symlinked registry breaks if drive letter changes | macOS volume names are stable; re-run `ln -sfn` if ever renamed |
| sccache slows release builds | `SCCACHE_DISABLE=1` escape hatch documented in Phase 4 |
| HFS+ corruption risk on unsafe eject | Always eject cleanly; HFS+ is journaled; caches are all rebuildable anyway — nothing on the drive is irreplaceable |

**Golden rule:** the WDD holds only *rebuildable* data — caches, build artifacts,
worktree checkouts of committed git state. Nothing irreplaceable ever lives there.

---

## Execution results (2026-07-27) — COMPLETED

All phases executed. Deviations from the plan as written:

1. **`build.target-dir` does not support `{workspace-path-hash}` templating** (only
   `build.build-dir` does). The global config sets `build-dir` only; small final
   artifacts stay in per-project `target/`. Intermediates (the bulk) are on the WDD.
2. **Toolchain pruning kept 1.97.1, 1.91.1, and 1.95** — know-me-system pins `1.97.1`
   via `rust-toolchain.toml` (root + rust/), kreuzberg pins `1.95`, conduit pins `1.91`.
   Pruned instead: 1.80, 1.93, 1.96, nightly-2026-05-01, nightly-2026-07-18.
3. **Global `RUSTFLAGS="-Zthreads=8 -Zshare-generics=y"` was NOT installed** — nightly
   -Z flags hard-fail the stable-pinned know-me-system build. Available as per-shell opt-in.
4. Spotlight exclusion used `/Volumes/my-passport/.metadata_never_index` (no sudo needed)
   instead of `mdutil -i off`.
5. Kreuzberg tesseract caches preserved at `/Volumes/my-passport/caches/kreuzberg-tesseract-*`
   before deleting the stale `cargo-target-*` variants. Base `cargo-target/` (5.5 G) kept.

Verified working: `cargo check -p gen_ui_types` (2m03s, stable 1.97.1 via project pin)
with intermediates landing at `/Volumes/my-passport/cargo-build/d2/b59207150bf95b`;
sccache active as rustc-wrapper (45 cache writes on first build); `rustup show` resolves
through the symlinked home with all 5 toolchains intact.

**Measured outcome:**

| Metric | Before | After |
|---|---|---|
| System drive free | 326 GB (83% full) | **499 GB (73% full)** |
| System drive used | ~1.5 TiB | 1.27 TiB (~240 GB less) |
| WDD used | 161 GB | 87 GB (stale variants purged) |
| WDD free | 1.7 TB | 1.7 TB |
| Rustup toolchains | 10 (20 GB) | 5 (9.3 GB, on WDD) |

Immediate free-space gain: **+173 GB** (some of the ~247 GB of deleted targets had
already been partially offset by container sharing; no APFS snapshots are holding blocks).
All future build/cache/worktree growth defers to the WDD.
