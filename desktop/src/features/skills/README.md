# feature: skills (Tile 06 · FUNC-SPEC §04 "Agent Skills")

The audit + governance surface for UAR skills: enable/disable, per-skill
consent policy editor, invocation history with full input/output, per-Hand
allowed-skills matrix. Not yet wired into `app/navigation.ts` /
`app/router.tsx`.

Planned decomposition:

- `screens/SkillsScreen.tsx` — registry list, MCP server installer
  (npm or local), invocation history drawer.
- `screens/SkillAuditScreen.tsx` — the append-only, hash-chained audit trail
  (FUNC-SPEC §09 "The audit surface") with signed-bundle export.
- `hooks/useSkills.ts`, `stores/skillsStore.ts`
- Rust: `gen_ui_mcp` + `gen_ui_agent::tools` already expose the registry;
  add commands `skills_list`, `skills_set_grant`, `skills_invocations`,
  `audit_export_bundle`.

Spec anchors: FUNC-SPEC §04 tile 6, §09; roadmap M5.
