# feature: prompt-lab (Tile 05 · FUNC-SPEC §04 "Prompt Lab")

Single-turn sandbox: full sampling parameter exposure, template library with
variables, side-by-side model comparison, ephemeral by default (no history
pollution). Not yet wired into `app/navigation.ts` / `app/router.tsx`.

Planned decomposition:

- `screens/PromptLabScreen.tsx` — prompt editor, parameter panel
  (temperature/top-p/max-tokens/system/stop), dual-model compare split.
- `hooks/usePromptLab.ts`
- `stores/promptLabStore.ts` — reuses the Chat inference API surface with
  `ephemeral: true`; template CRUD against SurrealDB template store.
- Rust: no new engine work — same `InferenceProvider` path as Chat; add a
  `prompt_lab_run` command that skips conversation persistence.

Spec anchors: FUNC-SPEC §04 tile 5; roadmap M4.
