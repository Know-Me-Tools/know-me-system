# Tasks

- [x] Define the backward-compatible multimodal inference request contract.
- [x] Pin, download, and verify matching projector artifacts.
- [x] Enable and initialize mtmd on iOS and Android.
- [x] Route validated local image attachments through the agent.
- [x] Add multimodal contract and failure tests.
- [ ] Verify image understanding on both physical phones.
      **DEFERRED to a future phase** by operator decision, 2026-07-27. All
      image-recognition work moves out of this phase together, rather than
      being partially certified here.

      The task is also not achievable as written today, and the reason is
      architectural rather than unfinished work: Android's default lane is
      Gemma 4 E2B via LiteRT-LM, which is TEXT-ONLY and never receives image
      tensors. `chat.rs:292` gates the multimodal path on
      `!cfg!(target_os = "android")` deliberately — images reach that model as
      Xberg-extracted text, not as vision input. iOS retains the multimodal
      path and is where this task is meaningful.

      Left UNTICKED rather than reworded into something passable. The future
      phase needs a scope decision first: narrow to iOS, or ship a
      vision-capable Android model.
