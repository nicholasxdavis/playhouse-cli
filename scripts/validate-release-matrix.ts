import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadReleaseTargets } from './lib/release-targets.js';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const workflowPath = path.join(root, '.github/workflows/release.yml');

interface WorkflowTarget {
  triple: string;
  archive: string;
  binName: string;
}

function parseReleaseWorkflowMatrix(yaml: string): WorkflowTarget[] {
  const blocks = yaml.split(/^          - os:/m).slice(1);
  const targets: WorkflowTarget[] = [];

  for (const block of blocks) {
    const triple = block.match(/^\s*target:\s*(\S+)/m)?.[1];
    const archive = block.match(/^\s*archive:\s*(\S+)/m)?.[1];
    const binName = block.match(/^\s*bin:\s*(\S+)/m)?.[1];
    if (!triple || !archive || !binName) {
      continue;
    }
    targets.push({ triple, archive, binName });
  }

  return targets;
}

function main(): void {
  const yaml = fs.readFileSync(workflowPath, 'utf8');
  const manifest = loadReleaseTargets();
  const workflow = parseReleaseWorkflowMatrix(yaml);

  if (workflow.length === 0) {
    console.error('Could not parse release.yml build matrix targets');
    process.exit(1);
  }

  let failed = false;
  const manifestByTriple = new Map(
    manifest.targets.map((t) => [t.triple, t]),
  );

  for (const entry of workflow) {
    const spec = manifestByTriple.get(entry.triple);
    if (!spec) {
      console.error(`release.yml target not in manifest: ${entry.triple}`);
      failed = true;
      continue;
    }
    if (spec.archive !== entry.archive) {
      console.error(
        `Archive mismatch for ${entry.triple}: manifest=${spec.archive} release.yml=${entry.archive}`,
      );
      failed = true;
    }
    if (spec.binName !== entry.binName) {
      console.error(
        `bin mismatch for ${entry.triple}: manifest=${spec.binName} release.yml=${entry.binName}`,
      );
      failed = true;
    }
  }

  for (const spec of manifest.targets) {
    if (!workflow.some((w) => w.triple === spec.triple)) {
      console.error(`Manifest target missing from release.yml matrix: ${spec.triple}`);
      failed = true;
    }
  }

  if (failed) {
    process.exit(1);
  }

  console.log(
    `Release matrix OK: ${workflow.length} targets match scripts/manifest/release-targets.json`,
  );
}

main();
