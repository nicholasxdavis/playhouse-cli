'use strict';

const path = require('path');
const fs = require('fs');

const ROOT = path.resolve(__dirname, '..', '..');
const VENDOR_DIR = path.join(ROOT, 'vendor');

function packageVersion() {
  const pkg = require(path.join(ROOT, 'package.json'));
  return pkg.version;
}

function githubRepo() {
  return process.env.PLAYHOUSE_GITHUB_REPO || 'nicholasxdavis/playhouse-cli';
}

function bundledBinaryPath(binName) {
  return path.join(VENDOR_DIR, binName);
}

function vendorExists(binName) {
  try {
    return fs.existsSync(bundledBinaryPath(binName));
  } catch {
    return false;
  }
}

module.exports = {
  ROOT,
  VENDOR_DIR,
  packageVersion,
  githubRepo,
  bundledBinaryPath,
  vendorExists,
};
