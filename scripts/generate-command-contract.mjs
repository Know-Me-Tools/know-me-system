#!/usr/bin/env node
import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const manifestPath = resolve(
  root,
  "rust/crates/tauri-plugin-gen-ui/command-manifest.json",
);
const manifestBytes = await readFile(manifestPath);
const manifest = JSON.parse(manifestBytes);
const source = await readFile(
  resolve(root, "rust/crates/tauri-plugin-gen-ui/src/commands.rs"),
  "utf8",
);
const guest = await readFile(
  resolve(root, "rust/crates/tauri-plugin-gen-ui/guest-js/src/index.ts"),
  "utf8",
);

if (manifest.schemaVersion !== 1 || !Array.isArray(manifest.commands)) {
  throw new Error("unsupported command manifest");
}
const unique = new Set(manifest.commands);
if (unique.size !== manifest.commands.length) {
  throw new Error("command manifest contains duplicate command names");
}

for (const command of manifest.commands) {
  const rustPattern = new RegExp(`pub\\s+async\\s+fn\\s+${command}\\b|pub\\s+fn\\s+${command}\\b`);
  if (!rustPattern.test(source)) {
    throw new Error(`missing Rust handler for ${command}`);
  }
  if (!guest.includes(`cmd("${command}")`)) {
    throw new Error(`missing TypeScript binding for ${command}`);
  }
}

const digest = createHash("sha256").update(manifestBytes).digest("hex");
const header = `// Generated from command-manifest.json (sha256:${digest}). Do not edit.\n`;
const ts = `${header}export const GEN_UI_COMMANDS = ${JSON.stringify(manifest.commands, null, 2)} as const;\nexport type GenUiCommand = (typeof GEN_UI_COMMANDS)[number];\nexport const GEN_UI_EVENTS = ${JSON.stringify(manifest.events, null, 2)} as const;\n`;
const dartCommands = manifest.commands.map((name) => `  '${name}',`).join("\n");
const dartEvents = manifest.events.map((name) => `  '${name}',`).join("\n");
const dart = `${header}const genUiCommands = <String>[\n${dartCommands}\n];\nconst genUiEvents = <String>[\n${dartEvents}\n];\n`;

await writeFile(
  resolve(root, "rust/crates/tauri-plugin-gen-ui/guest-js/src/command_contract.generated.ts"),
  ts,
);
await writeFile(
  resolve(root, "mobile/lib/bridge/command_contract.generated.dart"),
  dart,
);
process.stdout.write(
  `Generated and verified ${manifest.commands.length} commands and ${manifest.events.length} events.\n`,
);
