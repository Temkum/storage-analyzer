#!/usr/bin/env node
/**
 * Syncs a release version into every manifest that carries the product
 * version: tauri.conf.json (Tauri bundle), Cargo.toml (Rust crate), and
 * CMakeLists.txt (C++ project). The web app's package.json is intentionally
 * not touched — nothing in the web UI consumes it and syncing it would only
 * create drift against the pnpm lockfile.
 *
 * Usage: node scripts/sync-release-version.mjs <version>
 *        (or omit the argument to read VERSION — but the release workflow
 *        always passes the tag version explicitly)
 */
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = dirname(dirname(fileURLToPath(import.meta.url)));
const argVersion = process.argv[2];
const version = (
  argVersion ?? readFileSync(join(repoRoot, 'VERSION'), 'utf8')
).trim();

if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error(`Invalid semantic version: "${version}"`);
  process.exit(1);
}

const read = (...parts) => readFileSync(join(repoRoot, ...parts), 'utf8');
const write =
  (...parts) =>
  (content) =>
    writeFileSync(join(repoRoot, ...parts), content);

// apps/desktop/src-tauri/tauri.conf.json
{
  const path = ['apps', 'desktop', 'src-tauri', 'tauri.conf.json'];
  const json = JSON.parse(read(...path));
  json.version = version;
  write(...path)(`${JSON.stringify(json, null, 2)}\n`);
}

// apps/desktop/src-tauri/Cargo.toml
{
  const path = ['apps', 'desktop', 'src-tauri', 'Cargo.toml'];
  const content = read(...path).replace(
    /^(version\s*=\s*")([^"]+)(")/m,
    `$1${version}$3`
  );
  write(...path)(content);
}

// CMakeLists.txt
{
  const path = 'CMakeLists.txt';
  const content = read(path).replace(
    /(project\(SystemAnalyzer\s*\n\s*VERSION\s+)[0-9A-Za-z.-]+/,
    `$1${version}`
  );
  write(path)(content);
}

console.log(`Version synced to ${version} across all manifests.`);
