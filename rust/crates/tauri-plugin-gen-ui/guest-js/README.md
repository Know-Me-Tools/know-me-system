# @prometheus-ags/tauri-plugin-gen-ui

Guest-JS bindings for `tauri-plugin-gen-ui`. Typed `invoke()` wrappers + event
listeners for the `gen_ui_core` intent surface (chat, entity CRUD, memory/graph).

**Layer contract (TJ-ARCH-MOB-001):** import these only from Zustand stores —
never from a React component or hook.
