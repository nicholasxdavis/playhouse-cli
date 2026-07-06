#!/usr/bin/env node
'use strict';

const fs = require('fs');
const path = require('path');

const root = path.resolve(__dirname, '..');
const cargoToml = fs.readFileSync(path.join(root, 'Cargo.toml'), 'utf8');
const cargoMatch = cargoToml.match(/^version\s*=\s*"([^"]+)"/m);
const npmPkg = require(path.join(root, 'packages/playhouse/package.json'));

if (!cargoMatch) {
  console.error('Could not read version from Cargo.toml');
  process.exit(1);
}

const cargoVer = cargoMatch[1];
const npmVer = npmPkg.version;

if (cargoVer !== npmVer) {
  console.error(`Version mismatch: Cargo.toml=${cargoVer} packages/playhouse/package.json=${npmVer}`);
  process.exit(1);
}

console.log(`Version sync OK: ${cargoVer}`);
