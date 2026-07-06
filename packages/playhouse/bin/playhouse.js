#!/usr/bin/env node
'use strict';

const { spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const { getReleaseTarget } = require('../scripts/lib/platform');
const { ROOT, bundledBinaryPath, vendorExists } = require('../scripts/lib/paths');

function resolveBinary() {
  if (process.env.PLAYHOUSE_BIN) {
    const bin = process.env.PLAYHOUSE_BIN;
    if (!fs.existsSync(bin)) {
      console.error(`[playhouse] PLAYHOUSE_BIN not found: ${bin}`);
      process.exit(1);
    }
    return { bin, method: 'PLAYHOUSE_BIN' };
  }

  const { binName } = getReleaseTarget();
  const bundled = bundledBinaryPath(binName);
  if (vendorExists(binName)) {
    return { bin: bundled, method: 'npm' };
  }

  const onPath = process.platform === 'win32' ? 'playhouse.exe' : 'playhouse';
  return { bin: onPath, method: 'path' };
}

function main() {
  const { bin, method } = resolveBinary();
  const env = { ...process.env };

  if (method === 'npm') {
    env.PLAYHOUSE_INSTALL_METHOD = 'npm';
    env.PLAYHOUSE_NPM_ROOT = ROOT;
  } else if (method === 'PLAYHOUSE_BIN') {
    env.PLAYHOUSE_INSTALL_METHOD = 'PLAYHOUSE_BIN';
  }

  const result = spawnSync(bin, process.argv.slice(2), {
    stdio: 'inherit',
    env,
    windowsHide: true,
  });

  if (result.error) {
    if (result.error.code === 'ENOENT') {
      console.error(
        '[playhouse] Native binary not found.\n' +
          '  npm: reinstall or run postinstall (needs GitHub Release for your platform)\n' +
          '  dev: npm run link-local  (from packages/playhouse after cargo build)\n' +
          '  override: PLAYHOUSE_BIN=/path/to/playhouse',
      );
    } else {
      console.error(`[playhouse] ${result.error.message}`);
    }
    process.exit(1);
  }

  if (result.signal) {
    process.exit(1);
  }

  process.exit(result.status ?? 1);
}

main();
