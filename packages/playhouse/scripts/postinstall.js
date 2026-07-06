'use strict';

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execFileSync } = require('child_process');

const { getReleaseTarget, assetName } = require('./lib/platform');
const {
  ROOT,
  VENDOR_DIR,
  packageVersion,
  githubRepo,
  bundledBinaryPath,
  vendorExists,
} = require('./lib/paths');

function log(msg) {
  console.log(`[playhouse] ${msg}`);
}

function warn(msg) {
  console.warn(`[playhouse] ${msg}`);
}

function shouldSkip() {
  if (process.env.PLAYHOUSE_SKIP_DOWNLOAD === '1' || process.env.PLAYHOUSE_SKIP_DOWNLOAD === 'true') {
    return 'PLAYHOUSE_SKIP_DOWNLOAD';
  }
  if (process.env.PLAYHOUSE_BIN) {
    return 'PLAYHOUSE_BIN';
  }
  if (process.env.npm_config_ignore_scripts === 'true') {
    return 'npm_config_ignore_scripts';
  }
  return null;
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    const request = (currentUrl) => {
      https
        .get(currentUrl, (res) => {
          if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
            res.resume();
            request(res.headers.location);
            return;
          }
          if (res.statusCode !== 200) {
            file.close(() => fs.unlink(dest, () => {}));
            reject(new Error(`HTTP ${res.statusCode} for ${currentUrl}`));
            return;
          }
          res.pipe(file);
          file.on('finish', () => file.close(resolve));
        })
        .on('error', (err) => {
          file.close(() => fs.unlink(dest, () => {}));
          reject(err);
        });
    };
    request(url);
  });
}

function extractZip(archivePath, destDir) {
  if (process.platform === 'win32') {
    const ps = [
      'Expand-Archive',
      `-Path '${archivePath.replace(/'/g, "''")}'`,
      `-DestinationPath '${destDir.replace(/'/g, "''")}'`,
      '-Force',
    ].join(' ');
    execFileSync('powershell', ['-NoProfile', '-Command', ps], { stdio: 'inherit' });
    return;
  }
  execFileSync('unzip', ['-o', archivePath, '-d', destDir], { stdio: 'inherit' });
}

function extractTarGz(archivePath, destDir) {
  execFileSync('tar', ['-xzf', archivePath, '-C', destDir], { stdio: 'inherit' });
}

function findBinary(dir, binName) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isFile() && entry.name === binName) {
      return full;
    }
    if (entry.isDirectory()) {
      const nested = findBinary(full, binName);
      if (nested) return nested;
    }
  }
  return null;
}

function chmodUnix(filePath) {
  if (process.platform !== 'win32') {
    fs.chmodSync(filePath, 0o755);
  }
}

async function main() {
  const skip = shouldSkip();
  const { triple, ext, binName } = getReleaseTarget();

  if (vendorExists(binName)) {
    log(`Binary already present at vendor/${binName}`);
    return;
  }

  if (skip) {
    warn(`Skipping download (${skip}).`);
    if (!vendorExists(binName) && !process.env.PLAYHOUSE_BIN) {
      warn(
        'No bundled binary found. Set PLAYHOUSE_BIN or run: npm run link-local (from a Rust build)',
      );
    }
    return;
  }

  const version = process.env.PLAYHOUSE_VERSION || packageVersion();
  const repo = githubRepo();
  const name = assetName(version, triple, ext);
  const url = `https://github.com/${repo}/releases/download/v${version}/${name}`;

  fs.mkdirSync(VENDOR_DIR, { recursive: true });
  const archivePath = path.join(VENDOR_DIR, name);
  const extractDir = path.join(VENDOR_DIR, 'extract');

  log(`Downloading ${name}…`);
  try {
    await download(url, archivePath);
  } catch (err) {
    warn(`Download failed: ${err.message}`);
    warn(
      `Release v${version} may not exist yet. Options:\n` +
        `  • cargo install --path . --force\n` +
        `  • PLAYHOUSE_BIN=/path/to/playhouse npm run link-local\n` +
        `  • PLAYHOUSE_SKIP_DOWNLOAD=1 (if binary already on PATH)`,
    );
    process.exit(0);
  }

  fs.rmSync(extractDir, { recursive: true, force: true });
  fs.mkdirSync(extractDir, { recursive: true });

  if (ext === 'zip') {
    extractZip(archivePath, extractDir);
  } else {
    extractTarGz(archivePath, extractDir);
  }

  const found = findBinary(extractDir, binName);
  if (!found) {
    warn(`Binary ${binName} not found inside ${name}`);
    process.exit(0);
  }

  const dest = bundledBinaryPath(binName);
  fs.copyFileSync(found, dest);
  chmodUnix(dest);
  fs.rmSync(extractDir, { recursive: true, force: true });
  fs.unlinkSync(archivePath);

  log(`Installed native binary to vendor/${binName}`);
}

main().catch((err) => {
  warn(err.message || String(err));
  process.exit(0);
});
