'use strict';

/**
 * Dev helper: copy a locally built Rust binary into vendor/ for npm testing.
 *
 *   cargo build --release
 *   cd packages/playhouse && npm run link-local
 *
 * Or: PLAYHOUSE_LOCAL_BIN=../../target/release/playhouse npm run link-local
 */

const fs = require('fs');
const path = require('path');
const { getReleaseTarget } = require('./lib/platform');
const { ROOT, VENDOR_DIR, bundledBinaryPath } = require('./lib/paths');

const { binName } = getReleaseTarget();

const candidates = [
  process.env.PLAYHOUSE_LOCAL_BIN,
  path.join(ROOT, '..', '..', 'target', 'release', binName),
  path.join(ROOT, '..', '..', 'target', 'debug', binName),
].filter(Boolean);

let src = null;
for (const c of candidates) {
  const resolved = path.resolve(c);
  if (fs.existsSync(resolved)) {
    src = resolved;
    break;
  }
}

if (!src) {
  console.error(
    '[playhouse] No local binary found. Build first:\n' +
      '  cargo build --release\n' +
      'Or set PLAYHOUSE_LOCAL_BIN',
  );
  process.exit(1);
}

fs.mkdirSync(VENDOR_DIR, { recursive: true });
const dest = bundledBinaryPath(binName);
fs.copyFileSync(src, dest);
if (process.platform !== 'win32') {
  fs.chmodSync(dest, 0o755);
}

console.log(`[playhouse] Linked ${src} -> ${dest}`);
