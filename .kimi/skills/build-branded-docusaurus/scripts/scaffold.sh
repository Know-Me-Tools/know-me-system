#!/usr/bin/env bash
set -euo pipefail

site_dir="${1:?usage: scaffold.sh SITE_DIR SITE_NAME SITE_URL BASE_URL}"
site_name="${2:?site name is required}"
site_url="${3:?site URL is required}"
base_url="${4:?base URL is required}"
skill_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if test -e "$site_dir"; then
  echo "refusing to overwrite existing path: $site_dir" >&2
  exit 1
fi

npx --yes create-docusaurus@3.10.1 "$site_dir" classic --javascript --package-manager npm --skip-install
mkdir -p "$site_dir/scripts"
rm -f "$site_dir/docusaurus.config.js"
install -m 0644 "$skill_root/assets/docusaurus.config.mjs" "$site_dir/docusaurus.config.mjs"
install -m 0644 "$skill_root/assets/flat2.css" "$site_dir/src/css/custom.css"
install -m 0644 "$skill_root/assets/index.jsx" "$site_dir/src/pages/index.js"
install -m 0644 "$skill_root/assets/content-sources.yaml" "$site_dir/content-sources.yaml"
install -m 0644 "$skill_root/assets/sanitize.mjs" "$site_dir/scripts/sanitize.mjs"

SITE_NAME="$site_name" SITE_URL="$site_url" BASE_URL="$base_url" node --input-type=module - "$site_dir" <<'NODE'
import fs from 'node:fs';
import path from 'node:path';
const root = process.argv[2];
const configPath = path.join(root, 'docusaurus.config.mjs');
let config = fs.readFileSync(configPath, 'utf8');
config = config
  .replaceAll('__SITE_NAME__', process.env.SITE_NAME)
  .replaceAll('__SITE_URL__', process.env.SITE_URL)
  .replaceAll('__BASE_URL__', process.env.BASE_URL);
fs.writeFileSync(configPath, config);
NODE

cd "$site_dir"
npm pkg set 'scripts.sanitize=node scripts/sanitize.mjs' \
  'scripts.build=npm run sanitize && docusaurus build' \
  'overrides.serialize-javascript=7.0.5'
npm install --save-exact @docusaurus/core@3.10.1 @docusaurus/preset-classic@3.10.1 \
  @docusaurus/theme-mermaid@3.10.1 @mdx-js/react@3.1.1 \
  @easyops-cn/docusaurus-search-local@0.55.2 mermaid@11.16.0
npm install --package-lock-only

echo "scaffolded $site_dir; classify sources before replacing starter content"
