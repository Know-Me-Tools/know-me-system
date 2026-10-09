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

interface WireEntityRecord {
  id: string;
  entity_type: string;
  data_json: string;
}

interface WireListResult {
  items: WireEntityRecord[];
  next_cursor: string | null;
}

interface WireViewDescriptor {
  entity_type: string;
  filters: FilterSpec[];
  sorts: SortSpec[];
  limit: number | null;
  cursor: string | null;
}

function toWireView(view: ViewDescriptor): WireViewDescriptor {
  return {
    entity_type: view.entityType,
    filters: view.filters,
    sorts: view.sorts,
    limit: view.limit,
    cursor: view.cursor,
  };
}

function toWireEntity(record: EntityRecord): WireEntityRecord {
  return {
    id: record.id,
    entity_type: record.entityType,
    data_json: record.dataJson,
  };
}

function fromWireEntity(record: WireEntityRecord): EntityRecord {
  return {
    id: record.id,
    entityType: record.entity_type,
    dataJson: record.data_json,
  };
}

function fromWireList(result: WireListResult): ListResult {
  return {
    items: result.items.map(fromWireEntity),
    nextCursor: result.next_cursor,
  };
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
/** Chat inference lane: UAR service, the liter-llm cloud gateway, or an on-device model. */
export type ChatLane = "cloud" | "local" | "uar";

/**
 * Start a chat turn on the currently active lane ({@link getActiveLane}).
 * Returns the run_id whose ContentBlock events arrive on {@link onChatEvent} —
 * identical event shape on every lane, so callers never branch on lane.
 * `messages` is the prior turn history flattened to alternating role/text
 * pairs, oldest first.
 *
 * On the local lane the first call may take minutes: the model downloads before
 * the run_id returns, so a resolved promise means generation is genuinely
 * underway rather than still fetching weights.
 */
export interface InferenceHistoryMessage {
  role: "system" | "user" | "assistant" | "tool";
  content: string;
  name?: string;
  tool_call_id?: string;
}

export function streamAgentA2ui(
  userMessage: string,
  messages: InferenceHistoryMessage[],
  conversationId: string,
  branchId: string,
  /**
   * Absolute filesystem paths of user attachments. Rust reads and extracts
   * these; passing bytes would mean copying multi-megabyte files across the
   * IPC boundary for no benefit. Requires `userMessageId` — Rust correlates the
   * extracted content back to that message for memory ingest.
   */
  attachmentPaths?: string[],
  userMessageId?: string,
): Promise<string> {
  return invoke<string>(cmd("stream_agent_a2ui"), {
    userMessage,
    messages,
    conversationId,
    branchId,
    attachmentPaths: attachmentPaths ?? [],
    userMessageId: userMessageId ?? null,
  });
}
/** A file copied into durable app storage, described by Rust. */
export interface StoredAttachment {
  /** Durable path in app-owned storage. Safe to persist in a transcript. */
  path: string;
  fileName: string;
  mime: string;
  sizeBytes: number;
  isImage: boolean;
  isVideo: boolean;
  isDocument: boolean;
  /** Intrinsic pixel dimensions, present for images with a readable header. */
  width?: number;
  height?: number;
}
/**
 * Copy a picked file into durable app storage and describe it.
 *
 * Call at PICK time, before sending the turn. A browser `File` holds bytes with
 * no filesystem path, and Rust's attachment pipeline reads from disk — this
 * bridges the two once instead of pushing megabytes through IPC every turn.
 *
 * Returns metadata, not a bare path. The caller previously got only a string
 * and had to infer the kind from the file name, which produced `document` for
 * every attachment — that is why a picked PNG rendered as a grey chip. Mime,
 * image/video classification and dimensions are decided once in Rust and used
 * by both UIs.
 */
export function stageAttachment(
  conversationId: string,
  fileName: string,
  bytes: Uint8Array,
): Promise<StoredAttachment> {
  // Array.from because Tauri's IPC serialises to JSON — a Uint8Array would
  // arrive as an object of index keys rather than a byte sequence.
  return invoke<StoredAttachment>(cmd("stage_attachment"), {
    conversationId,
    fileName,
    bytes: Array.from(bytes),
  });
}
/** What a remote URL resolved to, fetched natively by Rust. */
export interface ResolvedRemoteContent {
  url: string;
  mime: string;
  /** Inline bytes when the content was small enough to embed. */
  dataBase64?: string;
  localPath?: string;
  /** Link-preview metadata, present for the `link` kind. */
  title?: string;
  description?: string;
  siteName?: string;
  imageUrl?: string;
}
/** What sort of remote content to resolve. Rust applies a size cap per kind. */
export type RemoteContentKind = "image" | "video" | "link" | "document";
/**
 * Fetch remote media natively rather than from the webview.
 *
 * The webview cannot always reach what the model references — CORS, mixed
 * content, or an origin it holds no credentials for. Flutter has resolved
 * through Rust since it shipped, which is why remote images and link previews
 * rendered on mobile and appeared broken on desktop.
 */
export async function resolveRemoteContent(
  url: string,
  kind: RemoteContentKind,
): Promise<ResolvedRemoteContent> {
  const raw = await invoke<string>(cmd("resolve_remote_content"), { url, kind });
  return JSON.parse(raw) as ResolvedRemoteContent;
}
export function chatApproveToolConfirmation(approvalId: string): Promise<string> {
  return invoke<string>(cmd("chat_approve_tool_confirmation"), { approvalId });
}
/**
 * Continue a chat run from a structured client/A2UI interaction.
 *
 * Stores pass the interaction as JSON so the Rust layer can validate shape,
 * extract typed UAR artifact responses, and preserve the shared stream/event
 * semantics. Components must go through the chat hook/store layer; they should
 * never call this binding directly.
 */
export function sendAgentInteractionA2ui(
  interactionJson: string,
  messages: InferenceHistoryMessage[],
  conversationId: string,
  branchId: string,
): Promise<string> {
  return invoke<string>(cmd("send_agent_interaction_a2ui"), {
    interactionJson,
    messages,
    conversationId,
    branchId,
  });
}

/** Persist a validated A2UI surface in the UAR-owned component library. */
export function promoteA2uiComponent(
  title: string,
  source: string,
  description?: string | null,
): Promise<Record<string, unknown>> {
  return invoke<Record<string, unknown>>(cmd("promote_a2ui_component"), {
    title,
    source,
    description: description ?? null,
  });
}
/** Which lane chat turns currently run on. Zero-config installs prefer local. */
export function getActiveLane(): Promise<ChatLane> {
  return invoke<ChatLane>(cmd("get_active_lane"));
}
/**
 * Switch lanes. Rejects if `lane` is "local" on a build with no local engine.
 * `uar` is the desktop/web service lane; mobile UAR must be embedded-library
 * mode and is not exposed through this Tauri binding.
 */
export function setActiveLane(lane: ChatLane): Promise<void> {
  return invoke<void>(cmd("set_active_lane"), { lane });
}
/** Whether a lane can be selected right now, and the reason when it cannot. */
export interface LaneAvailability {
  lane: ChatLane;
  available: boolean;
  /** Human-readable reason when `available` is false; null when usable. */
  reason: string | null;
}
/**
 * Availability for every selectable lane, in one call.
 *
 * The mode picker renders an unavailable lane as disabled WITH its reason
 * rather than offering a control that fails when clicked. Side-effect free —
 * it probes registration state and never loads an engine, so it is safe to
 * call on mount and after a failed switch.
 */
export async function laneAvailability(): Promise<LaneAvailability[]> {
  // Rust hands back JSON text (the FFI surface does the same) so the shape
  // stays serde-owned rather than duplicated across two bindings.
  const raw = await invoke<string>(cmd("lane_availability"));
  return JSON.parse(raw) as LaneAvailability[];
}
/** Whether this build has a local-inference engine — gate the toggle on this. */
export function hasLocalEngine(): Promise<boolean> {
  return invoke<boolean>(cmd("has_local_engine"));
}
export type ModelOperationState =
  | { state: "not_installed" }
  | { state: "downloading"; bytes_received: number; total_bytes: number; bytes_per_second: number; resumable: boolean }
  | { state: "verifying" }
  | { state: "loading" }
  | { state: "ready"; diagnostics: RuntimeDiagnostics }
  | { state: "cancelling" }
  | { state: "cancelled" }
  | { state: "failed"; error: { code: string; phase: string; message: string; retryable: boolean; diagnostics?: string } };
export interface RuntimeDiagnostics {
  modelId: string;
  backend: string;
  deviceName?: string;
  offloadedLayers?: number;
  fallbackReason?: string;
  gateReport?: RuntimeGateReport[];
}
export interface RuntimeGateReport {
  gate: number;
  name: string;
  status: 'pending' | 'passed' | 'failed' | 'skipped' | 'not_applicable';
  evidence?: string;
  errorCode?: string;
}
export type UarDeploymentMode = "embedded_library" | "http_service" | "cloud_service";
export type UarIntegrationState = "not_wired" | "projection_only" | "embedded_kernel" | "service_runtime";
export interface UarRuntimeStatus {
  authority: string;
  targetMode: UarDeploymentMode;
  currentState: UarIntegrationState;
  localProviderRole: string;
  mobileSidecarAllowed: boolean;
  localProviderBridgeAvailable: boolean;
  serviceRuntimeAvailable: boolean;
  serviceArtifactResponseAvailable: boolean;
  embeddedKernelAvailable: boolean;
  embeddedKernelBlocker: string | null;
  submodulePath: string;
  submoduleCommit: string | null;
}
export interface ConfiguredUarService {
  serviceUrl: string | null;
  hasApiKey: boolean;
  hasAgentArtifact: boolean;
  sessionId: string | null;
}
export interface SaveUarServiceRequest {
  serviceUrl: string;
  apiKey: string | null;
  agentArtifactJson: string;
  sessionId: string | null;
}
export interface UarServiceProbeResult {
  serviceUrl: string;
  ok: boolean;
  httpStatus: number | null;
  runtimeStatus: string | null;
  message: string;
}
export function cancelAgentRun(runId: string): Promise<boolean> {
  return invoke<boolean>(cmd("cancel_agent_run"), { runId });
}
export function prepareLocalModel(): Promise<void> {
  return invoke<void>(cmd("prepare_local_model"));
}
export function localModelStatus(): Promise<ModelOperationState> {
  return invoke<ModelOperationState>(cmd("local_model_status"));
}
export function cancelModelOperation(): Promise<void> {
  return invoke<void>(cmd("cancel_model_operation"));
}
export function localRuntimeDiagnostics(): Promise<RuntimeDiagnostics | null> {
  return invoke<RuntimeDiagnostics | null>(cmd("local_runtime_diagnostics"));
}
export function uarRuntimeStatus(): Promise<UarRuntimeStatus> {
  return invoke<UarRuntimeStatus>(cmd("uar_runtime_status"));
}
export function uarServiceConfig(): Promise<ConfiguredUarService> {
  return invoke<ConfiguredUarService>(cmd("uar_service_config"));
}
export function uarServiceSave(request: SaveUarServiceRequest): Promise<void> {
  return invoke<void>(cmd("uar_service_save"), { request });
}
export function uarServiceProbe(): Promise<UarServiceProbeResult> {
  return invoke<UarServiceProbeResult>(cmd("uar_service_probe"));
}
export interface ProviderCatalogEntry {
  id: string;
  displayName: string;
  defaultBaseUrl: string | null;
  requiresApiKey: boolean;
  configured: boolean;
  enabled: boolean;
  models: ProviderModelEntry[];
}
export interface ProviderModelEntry {
  id: string;
  displayName: string | null;
  contextWindow: number | null;
  maxOutputTokens: number | null;
  supportsVision: boolean;
  supportsTools: boolean;
  supportsReasoning: boolean;
  supportsStructuredOutput: boolean;
  supportsStreaming: boolean;
  enabled: boolean;
}
export interface ConfiguredProvider {
  id: string;
  displayName: string;
  baseUrl: string | null;
  enabled: boolean;
  hasApiKey: boolean;
  protocol: unknown;
  defaultModel: string | null;
  models: ProviderModelEntry[];
}
export interface SaveProviderRequest {
  id: string;
  kind: string;
  displayName?: string | null;
  baseUrl: string | null;
  apiKey: string | null;
  enabled: boolean;
  protocol?: unknown;
  defaultModel?: string | null;
  models?: ProviderModelEntry[];
}
export interface ProviderTestResult {
  ok: boolean;
  providerId: string;
  modelId: string;
  latencyMs: number;
  receivedText: boolean;
}
export interface ConfiguredCloudModel {
  providerId: string;
  modelId: string;
}
export function providerCatalog(): Promise<ProviderCatalogEntry[]> {
  return invoke<ProviderCatalogEntry[]>(cmd("provider_catalog"));
}
export type ResourceSelectionMode = "inherit" | "auto" | "all" | "none" | "selected";
export interface ResourceSelection {
  mode: ResourceSelectionMode;
  ids: string[];
  denied_ids: string[];
}
export interface ModelRoute {
  provider_id: string;
  model_id: string;
}
export interface RunPolicy {
  version: number;
  chat_mode: "local" | "uar" | "agent" | null;
  agent_id: string | null;
  model: ModelRoute | null;
  skills: ResourceSelection;
  tools: ResourceSelection;
  mcp_servers: ResourceSelection;
  knowledge_bases: ResourceSelection;
  memory_enabled: boolean | null;
  context_strategy: unknown | null;
  tool_approval: "inherit" | "auto" | "ask" | "deny";
}
export interface UnavailableResource {
  resource: string;
  reason: string;
  platformLimitation: boolean;
}
export interface ChatControlPlaneCatalog {
  agents: unknown;
  skills: unknown;
  tools: unknown;
  knowledgeBases: unknown;
  unavailable?: UnavailableResource[];
}
export interface UarAdminSnapshot {
  settings: unknown;
  settingTypes: unknown;
  agents: unknown;
  skills: unknown;
  tools: unknown;
  knowledgeBases: unknown;
  mcpServers: unknown;
  memories: unknown;
  unavailable?: UnavailableResource[];
}
export function chatControlCatalog(): Promise<ChatControlPlaneCatalog> {
  return invoke<ChatControlPlaneCatalog>(cmd("chat_control_catalog"));
}
export function uarAdminSnapshot(): Promise<UarAdminSnapshot> {
  return invoke<UarAdminSnapshot>(cmd("uar_admin_snapshot"));
}
export function uarSettingSave(key: string, value: unknown): Promise<unknown> {
  return invoke<unknown>(cmd("uar_setting_save"), { key, value });
}
export function uarAgentSave(agent: unknown): Promise<unknown> {
  return invoke<unknown>(cmd("uar_agent_save"), { agent });
}
export function uarAgentDelete(id: string): Promise<void> {
  return invoke<void>(cmd("uar_agent_delete"), { id });
}
export function uarMcpServerSave(name: string, server: unknown): Promise<unknown> {
  return invoke<unknown>(cmd("uar_mcp_server_save"), { name, server });
}
export function uarMcpServerDelete(name: string): Promise<unknown> {
  return invoke<unknown>(cmd("uar_mcp_server_delete"), { name });
}
export function uarMemoryAdd(content: string): Promise<unknown> {
  return invoke<unknown>(cmd("uar_memory_add"), { content });
}
export function uarMemoryUpdate(id: string, content: string): Promise<unknown> {
  return invoke<unknown>(cmd("uar_memory_update"), { id, content });
}
export function uarMemoryDelete(id: string): Promise<unknown> {
  return invoke<unknown>(cmd("uar_memory_delete"), { id });
}
export function uarKnowledgeBaseCreate(request: unknown): Promise<unknown> {
  return invoke<unknown>(cmd("uar_knowledge_base_create"), { request });
}
export function uarKnowledgeBaseDelete(id: string): Promise<void> {
  return invoke<void>(cmd("uar_knowledge_base_delete"), { id });
}
export function conversationPolicyGet(conversationId: string): Promise<RunPolicy | null> {
  return invoke<RunPolicy | null>(cmd("conversation_policy_get"), { conversationId });
}
export function conversationPolicySave(conversationId: string, policy: RunPolicy): Promise<unknown> {
  return invoke<unknown>(cmd("conversation_policy_save"), { conversationId, policy });
}
export function conversationPolicyDelete(conversationId: string): Promise<void> {
  return invoke<void>(cmd("conversation_policy_delete"), { conversationId });
}
export function conversationEffectiveConfig(conversationId: string): Promise<unknown> {
  return invoke<unknown>(cmd("conversation_effective_config"), { conversationId });
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
export function providerTest(id: string, model?: string | null): Promise<ProviderTestResult> {
  return invoke<ProviderTestResult>(cmd("provider_test"), { id, model: model ?? null });
}
export function cloudModelGet(): Promise<ConfiguredCloudModel | null> {
  return invoke<ConfiguredCloudModel | null>(cmd("cloud_model_get"));
}
export function cloudModelSave(providerId: string, modelId: string): Promise<void> {
  return invoke<void>(cmd("cloud_model_save"), { request: { providerId, modelId } });
}
export function entityList(view: ViewDescriptor): Promise<ListResult> {
  return invoke<WireListResult>(cmd("entity_list"), { view: toWireView(view) }).then(fromWireList);
}
export function entityGet(entityType: string, id: string): Promise<EntityRecord | null> {
  return invoke<WireEntityRecord | null>(cmd("entity_get"), { entityType, id })
    .then((record) => (record ? fromWireEntity(record) : null));
}
/** Retain commandId when retrying the same business submission across IPC. */
export function entityCreate(record: EntityRecord, commandId?: string): Promise<EntityRecord> {
  return invoke<WireEntityRecord>(cmd("entity_create"), { record: toWireEntity(record), commandId: commandId ?? null }).then(fromWireEntity);
}
export function entityUpdate(record: EntityRecord, commandId?: string): Promise<EntityRecord> {
  return invoke<WireEntityRecord>(cmd("entity_update"), { record: toWireEntity(record), commandId: commandId ?? null }).then(fromWireEntity);
}
export function entityDelete(entityType: string, id: string, commandId?: string): Promise<void> {
  return invoke<void>(cmd("entity_delete"), { entityType, id, commandId: commandId ?? null });
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
export function memoryDeleteConversationContext(conversationId: string): Promise<number> {
  return invoke<number>(cmd("memory_delete_conversation_context"), { conversationId });
}
export function memoryDeleteMessageContext(messageIds: string[]): Promise<number> {
  return invoke<number>(cmd("memory_delete_message_context"), { messageIds });
}
export function memoryPromoteContext(ids: string[]): Promise<number> {
  return invoke<number>(cmd("memory_promote_context"), { ids });
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
export const MODEL_STATUS = "gen-ui://model-status";

export function onChatEvent<T>(handler: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(CHAT_EVENT, (e) => handler(e.payload));
}
export function onEntityChange<T>(handler: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(ENTITY_CHANGE, (e) => handler(e.payload));
}
export function onSyncStatus<T>(handler: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(SYNC_STATUS, (e) => handler(e.payload));
}
export function onLocalModelStatus(handler: (payload: ModelOperationState) => void): Promise<UnlistenFn> {
  return listen<ModelOperationState>(MODEL_STATUS, (e) => handler(e.payload));
}

/** Placement is explicit; detach never cancels or admits a replacement execution. */
export interface ExecutionProfile {
  schemaVersion: 1; kind: 'standalone-embedded' | 'home' | 'personal-cloud' | 'enterprise';
  instanceId: string; serviceUrl: string | null; remoteInputApproved: boolean;
}
export interface ExecutionJob {
  hostRunId: string; conversationId: string; runtimeRunId: string | null;
  runtimeMode: string; profile: ExecutionProfile | null; cursor: number | null;
  state: 'admission-pending' | 'running' | 'observation-detached' | 'completed' | 'cancelled' | 'uncertain';
}
export const executionProfile = () => invoke<ExecutionProfile | null>('plugin:gen-ui|profile');
export const configureExecutionProfile = (profile: ExecutionProfile) => invoke<ExecutionProfile>('plugin:gen-ui|configure', { profile });
export const executionJobs = () => invoke<ExecutionJob[]>('plugin:gen-ui|jobs');
export const suspendExecution = (hostRunId: string) => invoke<ExecutionJob>('plugin:gen-ui|suspend', { hostRunId });
export const resumeExecution = (hostRunId: string) => invoke<ExecutionJob>('plugin:gen-ui|resume', { hostRunId });
export const cancelExecution = (hostRunId: string) => invoke<ExecutionJob>('plugin:gen-ui|cancel', { hostRunId });
