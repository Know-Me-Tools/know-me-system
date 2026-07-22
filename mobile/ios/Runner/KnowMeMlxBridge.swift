// TJ-ARCH-MOB-001 compliant
//
// Thin Swift binding for Rust-owned MLX inference on iOS.
//
// Flutter never calls this. Rust (gen_ui_inference::ios_mlx_ffi) owns the model
// catalog, download, verification, prompt/tool assembly, and event
// normalization; this file only binds MLXLLM generation and streams tokens
// back through the C vtable Rust registered at launch. Mirror of Android's
// KnowMeLiteRtLmBridge.kt.

import Foundation
import MLX
import MLXLLM
import MLXLMCommon

// The event JSON the Rust side deserializes (see MlxBridgeEvent in mlx.rs).
private enum BridgeEvent {
    case delta(String)
    case final
    case cancelled
    case error(String)

    var json: String {
        switch self {
        case .delta(let text):
            let escaped = KnowMeMlxBridge.encodeString(text)
            return "{\"type\":\"delta\",\"text\":\(escaped)}"
        case .final:
            return "{\"type\":\"final\"}"
        case .cancelled:
            return "{\"type\":\"cancelled\"}"
        case .error(let message):
            let escaped = KnowMeMlxBridge.encodeString(message)
            return "{\"type\":\"error\",\"message\":\(escaped)}"
        }
    }
}

/// Serialized inference request Rust sends to `generate` (see build_request_json).
private struct MlxRequest: Decodable {
    struct Message: Decodable {
        let role: String
        let content: String
    }
    struct GenerationSettings: Decodable {
        let temperature: Float?
        let top_p: Float?
        let max_tokens: UInt32?
    }
    let messages: [Message]
    let generationSettings: GenerationSettings?
}

/// The single serialized MLX runner. MLX GPU evaluation must not run
/// concurrently; one generation at a time is enforced here (the Rust host
/// serializes anyway).
final class KnowMeMlxBridge {
    static let shared = KnowMeMlxBridge()

    private let queue = DispatchQueue(label: "tools.knowme.mlx")
    private var container: ModelContainer?
    private var modelDir: String?
    private var cancelled = false

    // MARK: load

    /// Load the model whose files Rust already downloaded into `dir`.
    func load(modelDir dir: String) throws {
        try queue.sync {
            if container != nil && modelDir == dir {
                return
            }
            let url = URL(fileURLWithPath: dir)
            let configuration = ModelConfiguration(directory: url)
            let loaded = try runBlocking {
                try await LLMModelFactory.shared.loadContainer(configuration: configuration)
            }
            container = loaded
            modelDir = dir
        }
    }

    // MARK: generate

    /// Run one generation, invoking `emit` per streamed event.
    func generate(requestJSON: String, emit: (String) -> Void) {
        cancelled = false
        guard let container = queue.sync(execute: { self.container }) else {
            emit(BridgeEvent.error("MLX model is not loaded").json)
            return
        }
        let request: MlxRequest
        do {
            request = try JSONDecoder().decode(
                MlxRequest.self, from: Data(requestJSON.utf8))
        } catch {
            emit(BridgeEvent.error("MLX request decode: \(error)").json)
            return
        }

        let messages: [[String: String]] = request.messages.map {
            ["role": $0.role, "content": $0.content]
        }
        let params = GenerateParameters(
            temperature: request.generationSettings?.temperature ?? 0.7,
            topP: request.generationSettings?.top_p ?? 0.9)
        let maxTokens = Int(request.generationSettings?.max_tokens ?? 1024)

        do {
            try runBlocking {
                try await container.perform { context in
                    let input = try await context.processor.prepare(
                        input: UserInput(messages: messages))
                    let stream = try MLXLMCommon.generate(
                        input: input, parameters: params, context: context)
                    for await item in stream {
                        if self.cancelled {
                            emit(BridgeEvent.cancelled.json)
                            return
                        }
                        switch item {
                        case .chunk(let text):
                            emit(BridgeEvent.delta(text).json)
                        case .info:
                            break
                        @unknown default:
                            break
                        }
                    }
                    emit(BridgeEvent.final.json)
                }
            }
        } catch {
            emit(BridgeEvent.error("MLX generation: \(error)").json)
        }
    }

    func cancel() {
        cancelled = true
    }

    func close() {
        queue.sync {
            container = nil
            modelDir = nil
        }
    }

    // MARK: helpers

    /// JSON-encode a string (with surrounding quotes) so it can be spliced into
    /// the small event JSON without a full encoder per token.
    static func encodeString(_ value: String) -> String {
        if let data = try? JSONEncoder().encode(value),
            let json = String(data: data, encoding: .utf8)
        {
            return json
        }
        return "\"\""
    }

    /// Run an async body to completion on the current thread. The Rust host
    /// already calls us from a blocking worker, so blocking here is intended.
    private func runBlocking<T>(_ body: @escaping () async throws -> T) rethrows -> T {
        let semaphore = DispatchSemaphore(value: 0)
        var result: Result<T, Error>!
        Task {
            do {
                result = .success(try await body())
            } catch {
                result = .failure(error)
            }
            semaphore.signal()
        }
        semaphore.wait()
        return try result.get()
    }
}

// MARK: - C vtable registration

// Trampolines with C calling convention that the Rust vtable stores. Each
// forwards to the shared bridge; string pointers are valid only for the call.

private func mlxLoad(
    _ modelDir: UnsafePointer<CChar>?,
    _ configJSON: UnsafePointer<CChar>?,
    _ ctx: UnsafeMutableRawPointer?,
    _ onError: @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?) -> Void
) -> Int32 {
    guard let modelDir = modelDir else { return -1 }
    let dir = String(cString: modelDir)
    do {
        try KnowMeMlxBridge.shared.load(modelDir: dir)
        return 0
    } catch {
        "\(error)".withCString { onError(ctx, $0) }
        return 1
    }
}

private func mlxGenerate(
    _ requestJSON: UnsafePointer<CChar>?,
    _ ctx: UnsafeMutableRawPointer?,
    _ onEvent: @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?) -> Void
) -> Int32 {
    guard let requestJSON = requestJSON else { return -1 }
    let request = String(cString: requestJSON)
    KnowMeMlxBridge.shared.generate(requestJSON: request) { eventJSON in
        eventJSON.withCString { onEvent(ctx, $0) }
    }
    return 0
}

private func mlxCancel() {
    KnowMeMlxBridge.shared.cancel()
}

private func mlxClose() {
    KnowMeMlxBridge.shared.close()
}

/// The C vtable layout must match `MlxBridgeVTable` in ios_mlx_ffi.rs exactly.
private struct MlxBridgeVTable {
    let load:
        @convention(c) (
            UnsafePointer<CChar>?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?,
            @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?) -> Void
        ) -> Int32
    let generate:
        @convention(c) (
            UnsafePointer<CChar>?, UnsafeMutableRawPointer?,
            @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?) -> Void
        ) -> Int32
    let cancel: @convention(c) () -> Void
    let close: @convention(c) () -> Void
}

// Imported from the force-loaded Rust staticlib (libgen_ui_ffi.a).
@_silgen_name("knowme_mlx_register_bridge")
private func knowme_mlx_register_bridge(_ vtable: UnsafePointer<MlxBridgeVTable>) -> Int32

/// Register the MLX bridge with Rust. Call once at app launch, before Flutter
/// boot triggers model loading.
func registerKnowMeMlxBridge() {
    var vtable = MlxBridgeVTable(
        load: mlxLoad,
        generate: mlxGenerate,
        cancel: mlxCancel,
        close: mlxClose)
    _ = withUnsafePointer(to: &vtable) { knowme_mlx_register_bridge($0) }
}
