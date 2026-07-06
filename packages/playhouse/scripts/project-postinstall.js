#!/usr/bin/env node
'use strict';

/**
 * Optional postinstall hook for consumer projects.
 *
 * In your app's package.json:
 *   "scripts": {
 *     "postinstall": "playhouse-install-tools"
 *   }
 *
 * Or call directly:
 *   node node_modules/playhouse/scripts/project-postinstall.js
 *
 * Set PLAYHOUSE_INSTALL_STRICT=1 to fail npm install when playhouse install fails.
 */

const { spawnSync } = require('child_process');

const strict = process.env.PLAYHOUSE_INSTALL_STRICT === '1'
  || process.env.PLAYHOUSE_INSTALL_STRICT === 'true';

const profile = process.argv.includes('--minimal') ? '--minimal' : '--full';

const result = spawnSync('playhouse', ['install', profile, '--json'], {
  stdio: 'inherit',
  shell: process.platform === 'win32',
});

const code = result.status ?? 1;

if (code !== 0 && !strict) {
  console.warn(
    '[playhouse] install failed (exit %d). Verify may fail until you run: playhouse install %s',
    code,
    profile,
  );
  process.exit(0);
}

process.exit(code);
