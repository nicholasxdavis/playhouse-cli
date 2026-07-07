import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export interface NpmPlatform {
  platform: string;
  arch: string;
}

export interface ReleaseTarget {
  triple: string;
  archive: 'zip' | 'tar.gz';
  binName: string;
  npm?: NpmPlatform;
  homebrew: boolean;
}

export interface ReleaseTargetsManifest {
  targets: ReleaseTarget[];
}

const manifestPath = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  '..',
  'manifest',
  'release-targets.json',
);

export function loadReleaseTargets(): ReleaseTargetsManifest {
  const raw = fs.readFileSync(manifestPath, 'utf8');
  const data = JSON.parse(raw) as ReleaseTargetsManifest;
  validateManifest(data);
  return data;
}

export function assetName(version: string, triple: string, archive: string): string {
  return `playhouse-${version}-${triple}.${archive}`;
}

export function homebrewTargets(manifest: ReleaseTargetsManifest): ReleaseTarget[] {
  return manifest.targets.filter((t) => t.homebrew);
}

function validateManifest(manifest: ReleaseTargetsManifest): void {
  if (!Array.isArray(manifest.targets) || manifest.targets.length === 0) {
    throw new Error('release-targets.json: targets must be a non-empty array');
  }
  const triples = new Set<string>();
  for (const t of manifest.targets) {
    if (!t.triple || !t.archive || !t.binName) {
      throw new Error(`release-targets.json: invalid target entry: ${JSON.stringify(t)}`);
    }
    if (triples.has(t.triple)) {
      throw new Error(`release-targets.json: duplicate triple ${t.triple}`);
    }
    triples.add(t.triple);
    if (t.archive !== 'zip' && t.archive !== 'tar.gz') {
      throw new Error(`release-targets.json: invalid archive for ${t.triple}`);
    }
  }
}
