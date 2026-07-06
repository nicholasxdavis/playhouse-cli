#!/usr/bin/env node
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const artifactsDir = process.argv[2]
  ? path.resolve(process.argv[2])
  : path.join(root, 'artifacts');

interface PlatformSpec {
  target: string;
  asset: string;
}

const cargoToml = fs.readFileSync(path.join(root, 'Cargo.toml'), 'utf8');
const versionMatch = cargoToml.match(/^version\s*=\s*"([^"]+)"/m);
if (!versionMatch) {
  console.error('Could not read version from Cargo.toml');
  process.exit(1);
}

const version = versionMatch[1];
const tag = `v${version}`;

const specs: PlatformSpec[] = [
  {
    target: 'aarch64-apple-darwin',
    asset: `playhouse-${version}-aarch64-apple-darwin.tar.gz`,
  },
  {
    target: 'x86_64-apple-darwin',
    asset: `playhouse-${version}-x86_64-apple-darwin.tar.gz`,
  },
  {
    target: 'aarch64-unknown-linux-gnu',
    asset: `playhouse-${version}-aarch64-unknown-linux-gnu.tar.gz`,
  },
  {
    target: 'x86_64-unknown-linux-gnu',
    asset: `playhouse-${version}-x86_64-unknown-linux-gnu.tar.gz`,
  },
];

function sha256(filePath: string): string {
  const data = fs.readFileSync(filePath);
  return crypto.createHash('sha256').update(data).digest('hex');
}

function findAsset(name: string): string | null {
  const direct = path.join(artifactsDir, name);
  if (fs.existsSync(direct)) return direct;
  for (const entry of fs.readdirSync(artifactsDir, { withFileTypes: true })) {
    const nested = path.join(artifactsDir, entry.name, name);
    if (fs.existsSync(nested)) return nested;
  }
  return null;
}

const hashes: Record<string, string> = {};
for (const spec of specs) {
  const file = findAsset(spec.asset);
  if (!file) {
    console.error(`Missing artifact: ${spec.asset} in ${artifactsDir}`);
    process.exit(1);
  }
  hashes[spec.target] = sha256(file);
  console.log(`${spec.asset}: ${hashes[spec.target]}`);
}

const formulaPath = path.join(root, 'packaging/homebrew/playhouse.rb');
const formula = `# frozen_string_literal: true

# Auto-updated by release workflow (scripts/update-homebrew-formula.ts).
# Usage: brew install ./packaging/homebrew/playhouse.rb

class Playhouse < Formula
  desc 'QA CLI for security, functional testing, performance audits, and agent handoff'
  homepage 'https://github.com/nicholasxdavis/playhouse-cli'
  version '${version}'
  license 'MIT'

  on_macos do
    on_arm do
      url 'https://github.com/nicholasxdavis/playhouse-cli/releases/download/${tag}/playhouse-${version}-aarch64-apple-darwin.tar.gz'
      sha256 '${hashes['aarch64-apple-darwin']}'
    end
    on_intel do
      url 'https://github.com/nicholasxdavis/playhouse-cli/releases/download/${tag}/playhouse-${version}-x86_64-apple-darwin.tar.gz'
      sha256 '${hashes['x86_64-apple-darwin']}'
    end
  end

  on_linux do
    on_arm do
      url 'https://github.com/nicholasxdavis/playhouse-cli/releases/download/${tag}/playhouse-${version}-aarch64-unknown-linux-gnu.tar.gz'
      sha256 '${hashes['aarch64-unknown-linux-gnu']}'
    end
    on_intel do
      url 'https://github.com/nicholasxdavis/playhouse-cli/releases/download/${tag}/playhouse-${version}-x86_64-unknown-linux-gnu.tar.gz'
      sha256 '${hashes['x86_64-unknown-linux-gnu']}'
    end
  end

  def install
    bin.install 'playhouse'
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/playhouse --version")
  end
end
`;

fs.writeFileSync(formulaPath, formula);
console.log(`Updated ${formulaPath} for ${tag}`);
