// TJ-ARCH-MOB-001 compliant
//! SurrealDB 3.2 schema DDL. Kept as `OVERWRITE`/`IF NOT EXISTS` statements so
//! `GraphStore::init` is idempotent — safe to run on every boot (greenfield; no
//! in-place 2.x→3.x migration path per the analysis, which is fine for new stores).
//!
//! 3.x syntax notes (breaking vs 2.x, verified against SurrealDB 3.2 docs):
//!   * vector index is `HNSW DIMENSION n DIST COSINE` (MTREE was removed in 3.0)
//!   * full-text index is `FULLTEXT ANALYZER <a> BM25` (was `SEARCH ANALYZER`)
//!   * `search::rrf([...], k, limit)` fuses ranked lists natively (added in 3.0)

/// Analyzer + entity/memory tables + HNSW and BM25 indexes + the RELATE edge table.
/// One string so `init` runs it in a single `.query()` round-trip.
pub const SCHEMA_DDL: &str = r#"
-- Full-text analyzer: whitespace + class + camelCase splitting, English stemming.
DEFINE ANALYZER OVERWRITE gu_simple
    TOKENIZERS blank, class, camel, punct
    FILTERS lowercase, snowball(english);

-- entity: nodes in the knowledge graph (projects, notes, people, ...).
-- SCHEMAFULL (not SCHEMALESS): FLEXIBLE on `data` below requires it — SurrealDB
-- 3.x rejects FLEXIBLE fields on schemaless tables outright ("FLEXIBLE can only
-- be used in SCHEMAFULL tables"). This also gives Rule-29 typed enforcement on
-- entity_type/label, with `data` as the one deliberate escape-hatch JSON blob.
DEFINE TABLE IF NOT EXISTS entity SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS entity_type ON entity TYPE string;
DEFINE FIELD IF NOT EXISTS label       ON entity TYPE string;
DEFINE FIELD IF NOT EXISTS data        ON entity TYPE option<object> FLEXIBLE;

-- memory: retrievable text with its embedding; optionally linked to an entity.
DEFINE TABLE IF NOT EXISTS memory SCHEMALESS;
DEFINE FIELD IF NOT EXISTS text      ON memory TYPE string;
DEFINE FIELD IF NOT EXISTS kind      ON memory TYPE string DEFAULT 'note';
DEFINE FIELD IF NOT EXISTS entity    ON memory TYPE option<record<entity>>;
DEFINE FIELD IF NOT EXISTS embedding ON memory TYPE array<float>;
DEFINE FIELD IF NOT EXISTS created   ON memory TYPE datetime DEFAULT time::now();

-- HNSW vector index — semantic recall. 384 dims to match all-MiniLM-L6-v2 / bge-small.
DEFINE INDEX OVERWRITE memory_hnsw ON memory
    FIELDS embedding HNSW DIMENSION 384 DIST COSINE;

-- BM25 full-text index — lexical recall the vector lane misses.
DEFINE INDEX OVERWRITE memory_ft ON memory
    FIELDS text FULLTEXT ANALYZER gu_simple BM25;

-- relates_to: typed graph edges between entities, traversed by graph_expand.
DEFINE TABLE IF NOT EXISTS relates_to SCHEMALESS TYPE RELATION FROM entity TO entity;
DEFINE FIELD IF NOT EXISTS rel ON relates_to TYPE string DEFAULT 'related';

-- Config DB v1 (mobile only — desktop/web use the Postgres-dialect schema in
-- gen_ui_db instead; see gen_ui_db::relational::config). api_key_ref is a
-- reference into platform-secure storage, never a plaintext secret.
DEFINE TABLE IF NOT EXISTS provider SCHEMALESS;
DEFINE FIELD IF NOT EXISTS kind        ON provider TYPE string;
DEFINE FIELD IF NOT EXISTS base_url    ON provider TYPE option<string>;
DEFINE FIELD IF NOT EXISTS api_key_ref ON provider TYPE option<string>;
DEFINE FIELD IF NOT EXISTS enabled     ON provider TYPE bool DEFAULT true;

-- One record per (surface, lane), e.g. surface='chat', lane='cloud'|'local'.
-- SCHEMAFULL: see entity's comment above — FLEXIBLE on `params` requires it.
DEFINE TABLE IF NOT EXISTS model_pref SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS surface     ON model_pref TYPE string;
DEFINE FIELD IF NOT EXISTS lane        ON model_pref TYPE string;
DEFINE FIELD IF NOT EXISTS provider_id ON model_pref TYPE option<string>;
DEFINE FIELD IF NOT EXISTS model_id    ON model_pref TYPE string;
DEFINE FIELD IF NOT EXISTS params      ON model_pref TYPE option<object> FLEXIBLE;

-- SCHEMAFULL: see entity's comment above — FLEXIBLE on `value` requires it.
DEFINE TABLE IF NOT EXISTS app_setting SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS value ON app_setting TYPE option<object> FLEXIBLE;
"#;

/// Hybrid recall: vector lane + BM25 lane, fused by native `search::rrf`.
/// Binds: `$qvec` (query embedding), `$q` (query text), `$k` (rows to return).
/// Returns one ranked list of `{ id, text, kind, entity, score }`.
///
/// **The KNN `k` is a literal, not `$k`.** SurrealDB's `<|k,ef|>` operator parses its
/// arguments at query-parse time and rejects a bound parameter:
///   `Parse error: Unexpected token 'a parameter', expected an unsigned integer`
/// So every call failed before this. `$k` is still bound — it belongs on the final
/// LIMIT, where it does the caller's truncation.
///
/// Both lanes now hand RRF up to 64 candidates regardless of `k`, which is what fusion
/// wants: RRF can only rank what it is given, so a lane returning exactly `k` rows
/// cannot rescue a hit the other lane ranked poorly — the whole point of fusing two.
/// Only the fused output is cut to `$k`.
///
/// `<|64,64|>` = up to 64 HNSW neighbours, exploring 64 candidates (ef).
/// RRF k=60 is the standard smoothing constant; 128 caps fusion input.
pub const HYBRID_SEARCH_QUERY: &str = r#"
LET $vs = SELECT id, text, kind, entity, vector::distance::knn() AS distance
    FROM memory WHERE embedding <|64,64|> $qvec
    ORDER BY distance ASC LIMIT 64;
LET $ft = SELECT id, text, kind, entity, search::score(0) AS ft_score
    FROM memory WHERE text @0@ $q
    ORDER BY ft_score DESC LIMIT 64;
SELECT meta::id(id) AS id, text, kind, rrf_score AS score
    FROM search::rrf([$vs, $ft], 60, 128)
    LIMIT $k;
"#;

/// Vector-only recall: the HNSW lane alone, no BM25, no fusion (C-111 T3).
///
/// Exists to make the hybrid lane's value *demonstrable* rather than asserted — run the
/// same query both ways and watch a rare exact term (a product name, an error code) that
/// vector recall smooths away come back ranked first under fusion.
///
/// `score` is derived as `1/(1+distance)` so it stays "higher = better" like the fused
/// RRF score, keeping `MemoryHit`'s contract identical across modes. It is NOT comparable
/// to an RRF score in magnitude — only within a mode. The UI must never rank the two
/// together, which is why the mode is an explicit parameter rather than a hidden default.
///
/// Binds `$qvec` and `$k`. No `$q`: without a lexical lane there is nothing to match text
/// against, and binding an unused parameter would imply otherwise.
pub const VECTOR_SEARCH_QUERY: &str = r#"
SELECT meta::id(id) AS id, text, kind,
       math::fixed(1.0 / (1.0 + vector::distance::knn()), 6) AS score
    FROM memory WHERE embedding <|64,64|> $qvec
    ORDER BY score DESC
    LIMIT $k;
"#;
