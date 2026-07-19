package tools.knowme.knowme

import com.google.ai.edge.litertlm.Backend
import com.google.ai.edge.litertlm.Content
import com.google.ai.edge.litertlm.Contents
import com.google.ai.edge.litertlm.Conversation
import com.google.ai.edge.litertlm.ConversationConfig
import com.google.ai.edge.litertlm.Engine
import com.google.ai.edge.litertlm.EngineConfig
import com.google.ai.edge.litertlm.Message
import com.google.ai.edge.litertlm.OpenApiTool
import com.google.ai.edge.litertlm.SamplerConfig
import com.google.ai.edge.litertlm.ToolProvider
import com.google.ai.edge.litertlm.tool
import org.json.JSONArray
import org.json.JSONObject
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
    private var engine: Engine? = null
    private var conversation: Conversation? = null
    private var modelPath: String? = null
    private var cacheDir: String? = null

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
        val currentEngine = engine ?: error("LiteRT-LM engine is not loaded")
        val request = JSONObject(requestJson)
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

    @JvmStatic
    @Synchronized
    fun cancel() {
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
            if (content.isBlank()) continue
            when (role) {
                "system" -> systemInstruction = Contents.of(content)
                "assistant" -> initialMessages += Message.model(Contents.of(content))
                "tool" -> initialMessages += Message.tool(Contents.of(content))
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

    private fun messageToJson(message: Message): String {
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
        result.put("raw", message.toString())
        return result.toString()
    }

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
