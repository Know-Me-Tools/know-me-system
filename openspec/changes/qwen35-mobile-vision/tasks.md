# Tasks

- [x] Define the backward-compatible multimodal inference request contract.
- [x] Pin, download, and verify matching projector artifacts.
- [x] Enable and initialize mtmd on iOS and Android.
- [x] Route validated local image attachments through the agent.
- [x] Add multimodal contract and failure tests.
- [ ] Verify image understanding on both physical phones.
      NOT ACHIEVABLE AS WRITTEN, and the reason is architectural rather than a
      gap in the work: Android's default lane is Gemma 4 E2B via LiteRT-LM,
      which is TEXT-ONLY and never receives image tensors. `chat.rs:292` gates
      the multimodal path on `!cfg!(target_os = "android")` deliberately —
      images still reach the model there as extracted text/OCR through Xberg,
      but not as vision input.
      So "both phones" cannot be satisfied while Android runs a text-only
      model. iOS retains the multimodal path and is the surface where this task
      is meaningful.
      Left UNTICKED rather than reworded to something I could pass. Closing it
      needs a scope decision: either narrow the task to iOS, or ship a
      vision-capable Android model.
