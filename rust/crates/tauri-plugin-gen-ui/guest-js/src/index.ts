// TJ-ARCH-MOB-001 compliant
// Guest bindings for tauri-plugin-gen-ui. Typed wrappers over the plugin's
// commands + typed listeners for its event channels.
//
// LAYER CONTRACT (TJ-ARCH-MOB-001): call these ONLY from Zustand stores — never
// from a React component or hook. Components import hooks; hooks compose stores;
// stores own invoke()/listen().
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// Command names are prefixed by the plugin ("gen-ui") — Tauri maps them to
// "plugin:gen-ui|<command>".
const cmd = (name: string) => `plugin:gen-ui|${name}`;

// --- Mirrored gen_ui_types shapes (kept in lockstep with the Rust seams) -----
export interface EntityRecord {
  id: string;
  entityType: string;
  dataJson: string;
}
export interface ListResult {
  items: EntityRecord[];
  nextCursor: string | null;
}
export type FilterOp = "eq" | "ne" | "lt" | "lte" | "gt" | "gte" | "in" | "like";
export interface FilterSpec {
  field: string;
  op: FilterOp;
  valueJson: string;
}
export interface SortSpec {
  field: string;
  descending: boolean;
}
export interface ViewDescriptor {
  entityType: string;
  filters: FilterSpec[];
  sorts: SortSpec[];
  limit: number | null;
  cursor: string | null;
}

// --- Command wrappers --------------------------------------------------------

/** Boot-order invariant, step 1 of 3: open the config store + run migrations. */
export function runMigrations(): Promise<void> {
  return invoke<void>(cmd("run_migrations"));
}
/** Boot-order invariant, step 2 of 3. */
export function loadSeeds(): Promise<void> {
  return invoke<void>(cmd("load_seeds"));
}
/** Boot-order invariant, step 3 of 3. */
export function attachSyncShapes(): Promise<void> {
  return invoke<void>(cmd("attach_sync_shapes"));
}
export function entityRuntimeStart(tenantId: string): Promise<void> {
  return invoke<void>(cmd("entity_runtime_start"), { tenantId });
}
export function entityRuntimeStop(): Promise<void> {
  return invoke<void>(cmd("entity_runtime_stop"));
}
export function memoryIngest(text: string): Promise<string> {
  return invoke<string>(cmd("memory_ingest"), { text });
}
/** Chat inference lane: the liter-llm cloud gateway, or an on-device model. */
export type ChatLane = "cloud" | "local";

/**
 * Start a chat turn on the currently active lane ({@link getActiveLane}).
 * Returns the run_id whose ContentBlock events arrive on {@link onChatEvent} —
 * identical event shape on both lanes, so callers never branch on lane.
 * `messages` is the prior turn history flattened to alternating role/text
 * pairs, oldest first.
 *
 * On the local lane the first call may take minutes: the model downloads before
 * the run_id returns, so a resolved promise means generation is genuinely
 * underway rather than still fetching weights.
 */
export function streamAgentA2ui(userMessage: string, messages: string[]): Promise<string> {
  return invoke<string>(cmd("stream_agent_a2ui"), { userMessage, messages });
}
/** Which lane chat turns currently run on. Zero-config installs prefer local. */
export function getActiveLane(): Promise<ChatLane> {
  return invoke<ChatLane>(cmd("get_active_lane"));
}
/**
 * Switch lanes. Rejects if `lane` is "local" on a build with no local engine,
 * rather than silently answering from the cloud on the next turn — check
 * {@link hasLocalEngine} before offering the switch.
 */
export function setActiveLane(lane: ChatLane): Promise<void> {
  return invoke<void>(cmd("set_active_lane"), { lane });
}
/** Whether this build has a local-inference engine — gate the toggle on this. */
export function hasLocalEngine(): Promise<boolean> {
  return invoke<boolean>(cmd("has_local_engine"));
}
export interface ProviderCatalogEntry {
  id: string;
  displayName: string;
  defaultBaseUrl: string | null;
  requiresApiKey: boolean;
  supportsChat: boolean;
}
export interface ConfiguredProvider {
  id: string;
  kind: string;
  baseUrl: string | null;
  enabled: boolean;
  hasApiKey: boolean;
}
export interface SaveProviderRequest {
  id: string;
  kind: string;
  baseUrl: string | null;
  apiKey: string | null;
  enabled: boolean;
}
export interface ConfiguredCloudModel {
  providerId: string;
  modelId: string;
}
export function providerCatalog(): Promise<ProviderCatalogEntry[]> {
  return invoke<ProviderCatalogEntry[]>(cmd("provider_catalog"));
}
export function providerList(): Promise<ConfiguredProvider[]> {
  return invoke<ConfiguredProvider[]>(cmd("provider_list"));
}
export function providerSave(request: SaveProviderRequest): Promise<void> {
  return invoke<void>(cmd("provider_save"), { request });
}
export function providerDelete(id: string): Promise<void> {
  return invoke<void>(cmd("provider_delete"), { id });
}
export function cloudModelGet(): Promise<ConfiguredCloudModel | null> {
  return invoke<ConfiguredCloudModel | null>(cmd("cloud_model_get"));
}
export function cloudModelSave(providerId: string, modelId: string): Promise<void> {
  return invoke<void>(cmd("cloud_model_save"), { request: { providerId, modelId } });
}
export function entityList(view: ViewDescriptor): Promise<ListResult> {
  return invoke<ListResult>(cmd("entity_list"), { view });
}
export function entityGet(entityType: string, id: string): Promise<EntityRecord | null> {
  return invoke<EntityRecord | null>(cmd("entity_get"), { entityType, id });
}
export function entityCreate(record: EntityRecord): Promise<EntityRecord> {
  return invoke<EntityRecord>(cmd("entity_create"), { record });
}
export function entityUpdate(record: EntityRecord): Promise<EntityRecord> {
  return invoke<EntityRecord>(cmd("entity_update"), { record });
}
export function entityDelete(entityType: string, id: string): Promise<void> {
  return invoke<void>(cmd("entity_delete"), { entityType, id });
}
/** A hybrid graph-RAG search hit. Mirrors `gen_ui_db_graph::MemoryHit`. */
export interface MemoryHit {
  id: string;
  text: string;
  kind: string;
  score: number;
}

/** A neighbour reached by graph expansion. Mirrors `gen_ui_db_graph::RelatedEntity`. */
export interface RelatedEntity {
  id: string;
  label: string;
  /** snake_case on the wire: the Rust struct carries no serde `rename_all`, so
      field names serialize verbatim. */
  entity_type: string;
  score: number;
}

/**
 * Which retrieval lane memory search runs.
 *
 * - `hybrid` — vector + BM25, RRF-fused. The product path; the default.
 * - `vector` — HNSW vector recall alone. A DIAGNOSTIC, for showing what fusion buys
 *   on the same query.
 *
 * Scores compare only WITHIN a mode — an RRF score and a vector-similarity score are
 * different scales. Never merge or rank results from both together.
 */
export type SearchMode = "hybrid" | "vector";

/** Omit `mode` for the product path (hybrid). */
export function memorySearch(query: string, k: number, mode?: SearchMode): Promise<MemoryHit[]> {
  return invoke<MemoryHit[]>(cmd("memory_search"), { query, k, mode });
}
export function graphExpand(entityId: string, depth: number): Promise<RelatedEntity[]> {
  return invoke<RelatedEntity[]>(cmd("graph_expand"), { entityId, depth });
}
/** Start a Scribe microphone recording. Errors if one is already in progress. */
export function scribeStart(): Promise<void> {
  return invoke<void>(cmd("scribe_start"));
}
/** Stop the in-flight Scribe recording and return its on-device transcript. */
export function scribeStop(): Promise<string> {
  return invoke<string>(cmd("scribe_stop"));
}

// --- Event channels (mirror gen_ui_ffi StreamSink feeds) ---------------------
export const CHAT_EVENT = "gen-ui://chat-event";
export const ENTITY_CHANGE = "gen-ui://entity-change";
export const SYNC_STATUS = "gen-ui://sync-status";

export function onChatEvent<T>(handler: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(CHAT_EVENT, (e) => handler(e.payload));
}
export function onEntityChange<T>(handler: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(ENTITY_CHANGE, (e) => handler(e.payload));
}
export function onSyncStatus<T>(handler: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(SYNC_STATUS, (e) => handler(e.payload));
}
