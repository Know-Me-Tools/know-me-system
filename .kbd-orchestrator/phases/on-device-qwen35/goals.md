# Goals

- Ship Qwen3.5-9B Q4_K_M with matching vision projector on iOS using Metal
- Ship Qwen3.5-4B Q4_K_M with matching vision projector on Android using Vulkan and CPU fallback
- Add verified resumable per-platform model downloads and device capability preflight
- Raise local context and output limits for useful instruction following
- Wire multimodal image input and host-side structured tool calling through Rust gen_ui_core
- Verify inference on physical iPhone and Android devices
