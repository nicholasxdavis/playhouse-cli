'use strict';

/**
 * Map Node platform/arch to Rust release target triple and archive layout.
 * Must stay in sync with .github/workflows/release.yml asset names.
 */
function getReleaseTarget() {
  const { platform, arch } = process;

  if (platform === 'win32' && arch === 'x64') {
    return {
      triple: 'x86_64-pc-windows-msvc',
      ext: 'zip',
      binName: 'playhouse.exe',
    };
  }
  if (platform === 'linux' && arch === 'x64') {
    return {
      triple: 'x86_64-unknown-linux-gnu',
      ext: 'tar.gz',
      binName: 'playhouse',
    };
  }
  if (platform === 'linux' && arch === 'arm64') {
    return {
      triple: 'aarch64-unknown-linux-gnu',
      ext: 'tar.gz',
      binName: 'playhouse',
    };
  }
  if (platform === 'darwin' && arch === 'x64') {
    return {
      triple: 'x86_64-apple-darwin',
      ext: 'tar.gz',
      binName: 'playhouse',
    };
  }
  if (platform === 'darwin' && arch === 'arm64') {
    return {
      triple: 'aarch64-apple-darwin',
      ext: 'tar.gz',
      binName: 'playhouse',
    };
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
