// TJ-ARCH-MOB-001 compliant
//! Demo seed corpus (C-111 T2).
//!
//! Curated notes derived from `docs/reference-app/` — the KnowMe functional spec and
//! moodboard/user journeys — so a fresh install's memory search returns results about
//! the product this PoC actually demonstrates, rather than lorem ipsum. Search demos
//! land on real domain vocabulary (Hands, BossFang, Cedar, WASM plugins, on-device
//! inference) instead of nonsense that proves nothing about ranking quality.
//!
//! Deliberately NOT reusing `gen_ui_db::relational::SeedBundle`: that emits SQL for the
//! relational config store, whereas memory lives in embedded SurrealDB and every note
//! must be embedded by the `Embedder` at ingest time. Different store, different path.
//!
//! Each note carries a **stable id**, so `seed_corpus` is idempotent — re-running it
//! (every app start calls `load_seeds`) upserts rather than duplicating.
use crate::store::{GraphStore, MemoryRecord};
use crate::GraphError;

/// One seeded note: stable id, body, and kind.
///
/// `kind` is the hit's category in search results. The mix is intentional — a corpus
/// of one kind can't demonstrate that filtering or display-by-kind works.
struct Seed {
    id: &'static str,
    kind: &'static str,
    text: &'static str,
}

/// The corpus. Sourced from docs/reference-app/knowme-functional-specification-architecture.html
/// (FUNC-SPEC v1.0, May 2026) and the moodboard/user-journeys document.
///
/// Sized for a demo, not for benchmarking: enough distinct topics that hybrid search
/// has something to rank and that a rare lexical term (BM25's lane) can be told apart
/// from a semantic near-match (the vector lane). The C-111 plan said "a few hundred";
/// that is corpus-for-its-own-sake — every note here is one a user could plausibly
/// have written, and each embeds at startup, so padding costs boot time for no signal.
const SEEDS: &[Seed] = &[
    // ── Product identity ────────────────────────────────────────────────────
    Seed {
        id: "seed_overview",
        kind: "note",
        text: "KnowMe is a sovereign personal AI platform that runs the same workflow on \
               phone, tablet, and desktop from one Rust and React 19 codebase. The local \
               device is the primary runtime; the cloud is optional and opt-in.",
    },
    Seed {
        id: "seed_sovereignty",
        kind: "principle",
        text: "Sovereignty is structural, not configurable. Most private-AI products \
               express privacy as a setting the user toggles and the vendor promises to \
               honour. KnowMe enforces it by where the code runs: there is no private \
               mode because there is no other mode.",
    },
    Seed {
        id: "seed_no_central_account",
        kind: "principle",
        text: "There is no central account holding conversations and no training pipeline \
               ingesting user data. Conversation history, agent memory, and the encrypted \
               skills registry all live in an embedded SurrealDB on the user's machine.",
    },
    Seed {
        id: "seed_offline",
        kind: "note",
        text: "KnowMe works on the subway, on a flight, and in a SCIF. It behaves the same \
               whether the user is online or whether KnowMe LLC is still in business — the \
               convenience profile of a consumer app with the trust profile of paper.",
    },
    // ── The eight tools ─────────────────────────────────────────────────────
    Seed {
        id: "seed_eight_tools",
        kind: "note",
        text: "The eight thinking tools are Chat, Hands, Ask Image, Audio Scribe, Prompt \
               Lab, Skills, Models, and Settings — one coherent surface, fitted to each \
               screen size.",
    },
    Seed {
        id: "seed_audio_scribe",
        kind: "note",
        text: "Audio Scribe records and transcribes on-device with whisper, then saves the \
               transcript straight to memory in one tap. Nothing is uploaded to transcribe.",
    },
    Seed {
        id: "seed_prompt_lab",
        kind: "note",
        text: "Prompt Lab is where prompts get drafted, versioned, and compared side by \
               side before being promoted into a skill or an agent template.",
    },
    Seed {
        id: "seed_models_tool",
        kind: "note",
        text: "The Models tool manages which local weights are downloaded and which lane \
               each surface uses — swapping a model is a first-class action, not a \
               reinstall.",
    },
    // ── Inference ───────────────────────────────────────────────────────────
    Seed {
        id: "seed_inference_engines",
        kind: "note",
        text: "Language models run on the user's own hardware — Gemma, Qwen, Phi, and \
               Mistral families — using mistral.rs on desktop with Metal acceleration and \
               llama.cpp on mobile, both behind the InferenceProvider trait.",
    },
    Seed {
        id: "seed_inference_seam",
        kind: "principle",
        text: "UI layers and the agent depend only on the InferenceProvider trait, never \
               on an engine crate, so swapping or adding an inference lane never ripples \
               past the inference module.",
    },
    // ── Hands ───────────────────────────────────────────────────────────────
    Seed {
        id: "seed_hands",
        kind: "note",
        text: "Hands is the autonomous-agent system: six templates that run on a schedule, \
               on the user's device or their own BossFang instance, working while the user \
               sleeps.",
    },
    Seed {
        id: "seed_hands_summarize",
        kind: "task",
        text: "A canned Hands agent summarises this week's memories on demand, streaming \
               its thinking, tool calls, and final artifact as it works, then saves the \
               summary back to memory.",
    },
    // ── Sync ────────────────────────────────────────────────────────────────
    Seed {
        id: "seed_bossfang",
        kind: "note",
        text: "BossFang sync is peer-to-peer over the OFP gossip protocol. The user's \
               devices talk directly to their own BossFang instances — no hub, no broker, \
               no leak — and only when the user chooses.",
    },
    Seed {
        id: "seed_local_first",
        kind: "principle",
        text: "Local-first means an edit made in airplane mode is not a degraded \
               experience: it is written locally, queued, and replayed when connectivity \
               returns. Offline is a normal state, not an error.",
    },
    // ── Plugins & policy ────────────────────────────────────────────────────
    Seed {
        id: "seed_plugins",
        kind: "note",
        text: "The plugin system is content-addressed: WASM Component Model modules \
               distributed over IPFS and signed with Ed25519, sandboxed by Wasmtime and \
               governed by Cedar policy. Third parties extend the surface without ever \
               touching user data.",
    },
    Seed {
        id: "seed_cedar",
        kind: "principle",
        text: "Cloud calls require explicit Cedar policy approval at the runtime level — \
               the policy engine is the gate, not a checkbox in a settings panel.",
    },
    // ── Memory / graph-RAG (what this screen demonstrates) ───────────────────
    Seed {
        id: "seed_graph_rag",
        kind: "note",
        text: "Memory is graph-RAG over embedded SurrealDB: notes are embedded on ingest, \
               retrieved by fusing a vector lane and a BM25 lexical lane with reciprocal \
               rank fusion, and expanded outward across RELATE edges.",
    },
    Seed {
        id: "seed_rrf",
        kind: "note",
        text: "Reciprocal rank fusion combines rankings without needing the lanes' scores \
               to be comparable. A rare exact term the vector lane misses can still rank \
               first because the lexical lane put it at the top.",
    },
    Seed {
        id: "seed_hybrid_vs_vector",
        kind: "note",
        text: "Hybrid retrieval beats plain vector recall when a query hinges on a specific \
               token — a product name, an error code, an unusual proper noun — because \
               embeddings smooth exactly the distinctions that make such tokens useful.",
    },
    // ── Cross-platform ──────────────────────────────────────────────────────
    Seed {
        id: "seed_one_codebase",
        kind: "note",
        text: "One codebase ships to macOS, Windows, Linux, iOS, and Android. All \
               networking, LLM interaction, inference, MCP, agent logic, and persistence \
               live in the shared Rust core and are never re-implemented per platform.",
    },
    Seed {
        id: "seed_content_blocks",
        kind: "principle",
        text: "Every agent-to-UI event maps to exactly one ContentBlock variant — text, \
               thinking, code, citation, memory, tool use, tool result, skill, artifact, \
               image, divider. The Dart and TypeScript compilers enforce exhaustiveness, \
               so a missing case is a build error rather than a blank screen.",
    },
];

/// Ingest the demo corpus. Idempotent: notes carry stable ids, so repeat calls upsert
/// rather than duplicate — `load_seeds` runs on every app start.
///
/// Returns the number of notes seeded.
///
/// Embedding is the expensive part (one forward pass per note), and it happens inside
/// `memory_ingest`. Sequential on purpose: this is a boot path, and saturating the
/// embedder's threads here would compete with the UI's first paint for no user-visible
/// gain on a corpus this size.
pub async fn seed_corpus(store: &GraphStore) -> Result<usize, GraphError> {
    for seed in SEEDS {
        store
            .memory_ingest(MemoryRecord {
                id: Some(seed.id.to_string()),
                text: seed.text.to_string(),
                kind: seed.kind.to_string(),
                entity: None,
            })
            .await?;
    }
    Ok(SEEDS.len())
}

/// How many notes the corpus holds. Lets callers log/report without ingesting.
pub fn corpus_len() -> usize {
    SEEDS.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_ids_are_unique() {
        // Duplicate ids would silently overwrite each other at ingest, shrinking the
        // corpus without any error — the kind of bug that only shows up as "search
        // results feel thin".
        let mut ids: Vec<&str> = SEEDS.iter().map(|s| s.id).collect();
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), total, "seed ids must be unique");
    }

    #[test]
    fn seeds_are_non_empty_and_substantial() {
        for seed in SEEDS {
            assert!(!seed.text.trim().is_empty(), "{} has empty text", seed.id);
            assert!(!seed.kind.trim().is_empty(), "{} has empty kind", seed.id);
            // memory_ingest rejects empty text; a one-word note would embed to noise
            // and pollute ranking rather than demonstrate it.
            assert!(
                seed.text.split_whitespace().count() >= 8,
                "{} is too short to be a meaningful memory",
                seed.id
            );
        }
    }
}
