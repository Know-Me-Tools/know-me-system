package tools.knowme.knowme

import com.google.ai.edge.litertlm.Backend
import com.google.ai.edge.litertlm.Content
import com.google.ai.edge.litertlm.Contents
import com.google.ai.edge.litertlm.Conversation
import com.google.ai.edge.litertlm.ConversationConfig
import com.google.ai.edge.litertlm.Engine
import com.google.ai.edge.litertlm.EngineConfig
import com.google.ai.edge.litertlm.Message
import com.google.ai.edge.litertlm.MessageCallback
import com.google.ai.edge.litertlm.OpenApiTool
import com.google.ai.edge.litertlm.SamplerConfig
import com.google.ai.edge.litertlm.ToolProvider
import com.google.ai.edge.litertlm.tool
import android.util.Log
import android.app.Application
import android.content.pm.ApplicationInfo
import org.json.JSONArray
import org.json.JSONObject
import java.io.File
import java.util.Locale

/**
 * Thin Android runtime binding for Rust-owned local inference.
 *
 * Flutter does not call this class. Rust captures the process JavaVM in
 * JNI_OnLoad and invokes this adapter from gen_ui_inference. Keep policy,
 * prompt/context assembly, tool permissioning, persistence, and event
 * normalization in Rust; this file only binds the official LiteRT-LM Android
 * API.
 */
object KnowMeLiteRtLmBridge {
    private const val TAG = "KnowMeLiteRtLmBridge"
    private const val FORCE_TOOL_RESULT_FALLBACK_FILE = "knowme-force-litertlm-tool-result-fallback"

    private var engine: Engine? = null

    // Volatile because `cancel` reads this WITHOUT holding the object lock (see
    // the comment there); every writer still holds it, so publication is safe.
    @Volatile
    private var conversation: Conversation? = null

    // Streaming buffer. Written on the engine's callback thread, drained on the
    // Rust polling thread, so every access takes `streamLock` — deliberately a
    // separate lock from the object monitor, which `sendMessage` holds for a
    // whole blocking generation.
    private val streamLock = Any()
    private val streamChunks = mutableListOf<String>()
    private var streamDone = false
    private var streamError: String? = null

    /// Set when a generation is cancelled, cleared once the engine is rebuilt.
    ///
    /// `cancelProcess()` cancels state inside the native engine that outlives the
    /// Kotlin `Conversation` wrapper — closing and recreating the conversation is
    /// NOT enough, verified on device: the next `nativeSendMessage` still threw
    /// "CANCELLED: Task cancelled". `Engine` exposes no reset (only
    /// `initialize`/`close`/`createConversation`), so the only way to clear it is
    /// a full engine rebuild, done lazily on the next send so the cancel itself
    /// stays instant.
    @Volatile
    private var engineNeedsRebuild = false
    private var modelPath: String? = null
    private var cacheDir: String? = null

    /// Rebuild the engine if a cancel poisoned it.
    ///
    /// Callers must hold the object lock. The reload re-reads the same model from
    /// the same cache dir, so the XNNPack weight cache is warm — measured
    /// `load_ms` of 243-291ms on a Samsung SM-S936U, which is why this is
    /// acceptable on the turn after a stop rather than requiring a restart.
    private fun rebuildEngineIfPoisoned() {
        if (!engineNeedsRebuild) return
        val path = modelPath
        val cache = cacheDir
        engineNeedsRebuild = false
        if (path == null || cache == null) return
        Log.i(TAG, "rebuilding LiteRT-LM engine after cancellation")
        close()
        load(path, cache)
    }

    @JvmStatic
    @Synchronized
    fun load(modelPath: String, cacheDir: String): String {
        if (engine != null && this.modelPath == modelPath && this.cacheDir == cacheDir) {
            return diagnosticsJson("ready")
        }
        close()
        val config = EngineConfig(
            modelPath = modelPath,
            backend = Backend.CPU(),
            visionBackend = null,
            audioBackend = null,
            maxNumTokens = null,
            maxNumImages = null,
            cacheDir = cacheDir,
        )
        val loadedEngine = Engine(config)
        loadedEngine.initialize()
        engine = loadedEngine
        this.modelPath = modelPath
        this.cacheDir = cacheDir
        return diagnosticsJson("loaded")
    }

    @JvmStatic
    @Synchronized
    fun sendMessage(requestJson: String): String {
        rebuildEngineIfPoisoned()
        val currentEngine = engine ?: error("LiteRT-LM engine is not loaded")
        val request = JSONObject(requestJson)
        // Same reason as `startStream`: a cancelled conversation left open makes
        // the next send fail with "CANCELLED: Task cancelled".
        conversation?.let { previous ->
            conversation = null
            runCatching { previous.close() }
        }
        val activeConversation = currentEngine.createConversation(
            conversationConfigFromRequest(request),
        )
        conversation = activeConversation

        val messages = request.optJSONArray("messages") ?: JSONArray()
        val lastUser = lastUserContent(messages)
            ?: request.optString("prompt").takeIf { it.isNotBlank() }
            ?: error("request contains no user prompt")
        val response = activeConversation.sendMessage(lastUser)
        return messageToJson(response)
    }

    /**
     * Begin a streaming generation.
     *
     * The blocking [sendMessage] returns only when generation finishes, so
     * cancelling it discarded everything the model had produced — the text never
     * crossed the JNI boundary. `sendMessageAsync` delivers each chunk to a
     * callback instead, and this buffers those chunks for Rust to drain, so a
     * stop keeps whatever arrived before it.
     *
     * Deliberately NOT `@Synchronized`: it must return immediately so Rust can
     * poll [drainStream] while generation continues. Holding the object lock for
     * the whole run is what made the old `cancel()` unable to interrupt.
     */
    @JvmStatic
    fun startStream(requestJson: String) {
        val currentEngine = synchronized(this) {
            rebuildEngineIfPoisoned()
            engine
        } ?: error("LiteRT-LM engine is not loaded")
        val request = JSONObject(requestJson)
        // Close the previous conversation before starting a new one. A cancelled
        // conversation stays cancelled: leaving it open made the *next*
        // `nativeSendMessage` fail immediately with "CANCELLED: Task cancelled",
        // so one stop poisoned every following turn.
        synchronized(this) {
            conversation?.let { previous ->
                conversation = null
                runCatching { previous.close() }
            }
        }
        val activeConversation = currentEngine.createConversation(
            conversationConfigFromRequest(request),
        )
        synchronized(this) { conversation = activeConversation }

        val messages = request.optJSONArray("messages") ?: JSONArray()
        val lastUser = lastUserContent(messages)
            ?: request.optString("prompt").takeIf { it.isNotBlank() }
            ?: error("request contains no user prompt")

        synchronized(streamLock) {
            streamChunks.clear()
            streamDone = false
            streamError = null
        }

        activeConversation.sendMessageAsync(
            Message.user(lastUser),
            object : MessageCallback {
                override fun onMessage(message: Message) {
                    synchronized(streamLock) { streamChunks += messageToJson(message) }
                }

                override fun onDone() {
                    synchronized(streamLock) { streamDone = true }
                }

                override fun onError(error: Throwable) {
                    // Recorded rather than thrown: the throw would land on the
                    // engine's callback thread where Rust cannot observe it.
                    synchronized(streamLock) {
                        streamError = error.message ?: error.toString()
                        streamDone = true
                    }
                }
            },
        )
    }

    /**
     * Hand Rust everything buffered since the last drain.
     *
     * Returns `{"chunks":[...],"done":bool,"error":string?}`. Chunks are removed
     * as they are read, so a cancel mid-generation still yields whatever the
     * model produced up to that point rather than losing the run.
     */
    @JvmStatic
    fun drainStream(): String {
        val payload = JSONObject()
        val chunks = JSONArray()
        synchronized(streamLock) {
            streamChunks.forEach { chunks.put(JSONObject(it)) }
            streamChunks.clear()
            payload.put("done", streamDone)
            streamError?.let { payload.put("error", it) }
        }
        payload.put("chunks", chunks)
        return payload.toString()
    }

    @JvmStatic
    @Synchronized
    fun sendToolResults(resultsJson: String): String {
        val activeConversation = conversation ?: error("LiteRT-LM conversation is not active")
        val results = JSONArray(resultsJson)
        val responses = mutableListOf<Content.ToolResponse>()
        for (index in 0 until results.length()) {
            val item = results.optJSONObject(index) ?: continue
            val name = item.optString("toolName")
            if (name.isBlank()) continue
            val output = item.optString("outputJson", "{}")
            responses += Content.ToolResponse(name, output)
        }
        if (responses.isEmpty()) {
            error("tool result continuation requires at least one named tool result")
        }
        val response = activeConversation.sendMessage(Message.tool(Contents.of(responses)))
        val forceFallback = shouldForceToolResultFallback()
        if (messageHasVisibleOutput(response) && !forceFallback) {
            return messageToJson(response)
        }
        if (forceFallback) {
            Log.w(TAG, "Forcing LiteRT-LM tool-result fallback for Android certification")
        }

        val followUp = activeConversation.sendMessage(
            "Use the tool result that was just provided to answer the user's original request. " +
                "If the tool result reports an error, explain the error briefly. " +
                "Do not call another tool unless it is required to answer."
        )
        return messageToJson(followUp, toolResultFallbackApplied = true)
    }

    private fun shouldForceToolResultFallback(): Boolean {
        if (!isDebuggableApp()) return false
        val dir = cacheDir ?: return false
        return File(dir, FORCE_TOOL_RESULT_FALLBACK_FILE).exists()
    }

    private fun isDebuggableApp(): Boolean {
        return try {
            val activityThread = Class.forName("android.app.ActivityThread")
            val application = activityThread
                .getMethod("currentApplication")
                .invoke(null) as? Application ?: return false
            (application.applicationInfo.flags and ApplicationInfo.FLAG_DEBUGGABLE) != 0
        } catch (_: Throwable) {
            false
        }
    }

    /**
     * Interrupt an in-flight generation.
     *
     * Deliberately NOT `@Synchronized`. `sendMessage` holds the object lock for
     * the whole generation — tens of seconds for a long answer — so a
     * synchronized `cancel` would block until generation finished and then
     * cancel nothing. Device certification caught exactly that: a run completed
     * with 1506 characters after the stop tap.
     *
     * `conversation` is `@Volatile` and `cancelProcess` is the LiteRT-LM API's
     * own thread-safe interrupt, so reading the reference without the lock is
     * safe. Cancelling when nothing is running is a no-op.
     */
    @JvmStatic
    fun cancel() {
        // Mark before cancelling: the native state is poisoned the moment
        // cancelProcess lands, and the next send must rebuild regardless of
        // whether this call throws.
        engineNeedsRebuild = true
        conversation?.cancelProcess()
    }

    @JvmStatic
    @Synchronized
    fun close() {
        conversation?.close()
        conversation = null
        engine?.close()
        engine = null
        modelPath = null
        cacheDir = null
    }

    @JvmStatic
    @Synchronized
    fun diagnosticsJson(): String = diagnosticsJson(if (engine?.isInitialized() == true) "ready" else "not_loaded")

    private fun conversationConfigFromRequest(request: JSONObject): ConversationConfig {
        val messages = request.optJSONArray("messages") ?: JSONArray()
        val initialMessages = mutableListOf<Message>()
        var systemInstruction: Contents? = null
        for (index in 0 until messages.length()) {
            val item = messages.optJSONObject(index) ?: continue
            val role = item.optString("role")
            val content = item.optString("content")
            // Only user turns are safe to drop when empty. Dropping a blank
            // assistant or tool turn breaks call/result pairing: the model would
            // see a tool result with no preceding call (or vice versa) and lose
            // the thread. Rust renders tool calls into `content`, so a blank
            // assistant turn here means genuinely empty output, which we still
            // preserve as a placeholder to keep the transcript ordered.
            if (content.isBlank() && role != "assistant" && role != "tool") continue
            when (role) {
                "system" -> systemInstruction = Contents.of(content)
                "assistant" -> initialMessages += Message.model(Contents.of(nonEmpty(content)))
                "tool" -> initialMessages += Message.tool(Contents.of(nonEmpty(content)))
                // The latest user message is sent through sendMessage() so the
                // Conversation API computes only the incremental prompt.
                "user" -> if (index < messages.length() - 1) initialMessages += Message.user(content)
            }
        }

        val settings = request.optJSONObject("generation_settings")
        val samplerConfig = SamplerConfig(
            settings?.optInt("top_k", 20) ?: 20,
            settings?.optDouble("top_p", 0.8) ?: 0.8,
            settings?.optDouble("temperature", 0.7) ?: 0.7,
            0xC0FFEE.toInt(),
        )

        return ConversationConfig(
            systemInstruction = systemInstruction,
            initialMessages = initialMessages,
            tools = toolProvidersFromRequest(request),
            samplerConfig = samplerConfig,
            automaticToolCalling = false,
            channels = emptyList(),
            extraContext = mapOf("enable_thinking" to false),
        )
    }

    /**
     * Keep a preserved-but-empty turn representable. `Contents.of("")` is not a
     * safe assumption across LiteRT-LM versions, and an ordered transcript with
     * an explicit placeholder beats a silently dropped turn.
     */
    private fun nonEmpty(content: String): String =
        content.ifBlank { "(no output)" }

    private fun toolProvidersFromRequest(request: JSONObject): List<ToolProvider> {
        val tools = request.optJSONArray("tools") ?: return emptyList()
        val providers = mutableListOf<ToolProvider>()
        for (index in 0 until tools.length()) {
            val item = tools.optJSONObject(index) ?: continue
            val name = item.optString("name")
            if (name.isBlank()) continue
            val description = item.optString("description")
            val parameters = item.optJSONObject("parameters") ?: JSONObject().put("type", "object")
            val schema = JSONObject()
                .put("name", name)
                .put("description", description)
                .put("parameters", parameters)
            providers += tool(RegisteredOpenApiTool(schema.toString()))
        }
        return providers
    }

    private fun lastUserContent(messages: JSONArray): String? {
        for (index in messages.length() - 1 downTo 0) {
            val item = messages.optJSONObject(index) ?: continue
            if (item.optString("role").lowercase(Locale.US) == "user") {
                return item.optString("content").takeIf { it.isNotBlank() }
            }
        }
        return null
    }

    private fun messageToJson(message: Message, toolResultFallbackApplied: Boolean = false): String {
        val result = JSONObject()
        result.put("role", message.role.value)
        result.put("text", message.contents.contents.joinToString(separator = "") { contentToText(it) })
        val calls = JSONArray()
        for (toolCall in message.toolCalls) {
            val item = JSONObject()
            item.put("name", toolCall.name)
            item.put("arguments", JSONObject(toolCall.arguments))
            calls.put(item)
        }
        result.put("toolCalls", calls)
        result.put("toolResultFallbackApplied", toolResultFallbackApplied)
        result.put("raw", message.toString())
        return result.toString()
    }

    private fun messageHasVisibleOutput(message: Message): Boolean =
        message.contents.contents.any { contentToText(it).isNotBlank() } ||
            message.toolCalls.isNotEmpty()

    private fun contentToText(content: Content): String =
        when (content) {
            is Content.Text -> content.text
            else -> content.toString()
        }

    private fun diagnosticsJson(state: String): String =
        JSONObject()
            .put("state", state)
            .put("backend", "litert-lm-cpu")
            .put("modelPath", modelPath)
            .put("cacheDir", cacheDir)
            .toString()

    private class RegisteredOpenApiTool(
        private val schemaJson: String,
    ) : OpenApiTool {
        override fun getToolDescriptionJsonString(): String = schemaJson

        override fun execute(paramsJsonString: String): String =
            """{"error":"tool execution is host-managed by Rust"}"""
    }
}
