#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

const cargoToml = fs.readFileSync(path.join(root, 'Cargo.toml'), 'utf8');
const cargoMatch = cargoToml.match(/^version\s*=\s*"([^"]+)"/m);
const npmPkg = JSON.parse(
  fs.readFileSync(path.join(root, 'packages/playhouse/package.json'), 'utf8'),
) as { version: string };
const homebrew = fs.readFileSync(
  path.join(root, 'packaging/homebrew/playhouse.rb'),
  'utf8',
);
const homebrewMatch = homebrew.match(/^\s*version\s+'([^']+)'/m);

if (!cargoMatch) {
  console.error('Could not read version from Cargo.toml');
  process.exit(1);
}

const cargoVer = cargoMatch[1];
const npmVer = npmPkg.version;
const homebrewVer = homebrewMatch?.[1] ?? null;

let failed = false;

if (cargoVer !== npmVer) {
  console.error(
    `Version mismatch: Cargo.toml=${cargoVer} packages/playhouse/package.json=${npmVer}`,
  );
  failed = true;
}

if (!homebrewVer) {
  console.error('Could not read version from packaging/homebrew/playhouse.rb');
  failed = true;
} else if (cargoVer !== homebrewVer) {
  console.error(
    `Version mismatch: Cargo.toml=${cargoVer} packaging/homebrew/playhouse.rb=${homebrewVer}`,
  );
  failed = true;
}

if (homebrew.includes('REPLACE_ON_RELEASE')) {
  console.warn(
    'Warning: packaging/homebrew/playhouse.rb still has REPLACE_ON_RELEASE sha256 placeholders',
  );
}

if (failed) {
  process.exit(1);
}

console.log(`Version sync OK: ${cargoVer} (cargo, npm, homebrew)`);
