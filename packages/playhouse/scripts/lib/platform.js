'use strict';

const fs = require('node:fs');
const path = require('node:path');

/**
 * Map Node platform/arch to Rust release target triple and archive layout.
 * Data source: scripts/manifest/release-targets.json (copied here for npm publish).
 */
const manifest = JSON.parse(
  fs.readFileSync(path.join(__dirname, 'release-targets.json'), 'utf8'),
);

function getReleaseTarget() {
  const { platform, arch } = process;

  for (const target of manifest.targets) {
    if (
      target.npm &&
      target.npm.platform === platform &&
      target.npm.arch === arch
    ) {
      return {
        triple: target.triple,
        ext: target.archive,
        binName: target.binName,
      };
    }
  }

  throw new Error(
    `Unsupported platform: ${platform}-${arch}. ` +
      'Use PLAYHOUSE_BIN to point at a prebuilt binary, or cargo install playhouse.',
  );
}

function assetName(version, triple, ext) {
  return `playhouse-${version}-${triple}.${ext}`;
}

module.exports = { getReleaseTarget, assetName };
