# Static Security, Dependencies, Secrets, and Container Engine: Trivy Integration

> **Status:** Partially implemented. Playhouse runs `trivy fs` with vuln + secret scanners for workspace verify. Container image scanning, IaC misconfiguration rules, and SBOM export described below are **planned** or depend on Trivy flags not yet wired in Playhouse.

## Overview

Playhouse CLI integrates **Trivy** as its comprehensive static security, dependency vulnerability, secret detection, and Infrastructure as Code (IaC) verification engine. While OWASP ZAP evaluates running applications for dynamic web vulnerabilities and Lighthouse verifies runtime performance, Trivy analyzes static project files, package dependencies, container images, and deployment configurations. By executing a single unified filesystem command (`trivy fs .`), Playhouse CLI provides AI coding agents and human developers with immediate, multi-layered security auditing across the entire software supply chain.

## The All-In-One Modern Static Scanner

In traditional development environments, teams must configure separate tools for dependency checking, secret scanning, linting Dockerfiles, and generating Software Bill of Materials (SBOMs). Trivy consolidates these critical security checks into a single, high-speed binary.

When an AI coding agent runs `trivy fs .` via Playhouse CLI, the engine automatically inspects:
* Package Dependencies: Scans manifest files and lockfiles across language ecosystems including Node.js (`package-lock.json`, `pnpm-lock.yaml`), Python (`requirements.txt`, `poetry.lock`), Rust (`Cargo.lock`), and Go (`go.mod`).
* Hardcoded Secrets: Detects leaked API keys, authentication tokens, private SSH keys, database connection strings, and certificates before they are committed to version control.
* Container Images and Dockerfiles: Audits container configurations for insecure base images, root user privileges, missing health checks, and unpatched operating system CVEs.
* Infrastructure as Code (IaC): Scans Terraform configurations, Kubernetes deployment manifests, and AWS CloudFormation templates for cloud security misconfigurations.
* SBOM Generation: Automatically generates industry-standard Software Bill of Materials artifacts (CycloneDX or SPDX) for compliance and auditing.

## Provisioning and Installation

During workspace initialization, Playhouse CLI checks for the presence of the Trivy command-line utility and installs or provisions it automatically:

```bash
# Verify local installation
trivy --version

# Global installation via Homebrew (macOS/Linux) or script provisioning
brew install trivy

# Download and update the vulnerability and secret detection database
trivy image --download-db-only
```

## Core Verification Workflows and Commands

Playhouse CLI wraps Trivy command execution with structured JSON formatting (`--format json`) to enable seamless programmatic ingestion by AI agents:

### 1. Unified Filesystem Security Sweep
The primary command executed during Playhouse quality gates. It scans the local repository for CVE vulnerabilities, exposed secrets, and configuration flaws simultaneously:

```bash
# Execute full filesystem scan with JSON output and severity filtering
trivy fs --scanners vuln,secret,config --severity HIGH,CRITICAL --format json --output .playhouse/reports/trivy-results.json .
```

### 2. Dedicated Secret and Credential Scanning
To prevent accidental credential leaks during automated agent refactoring sessions, Playhouse CLI can trigger targeted secret sweeps across local files or entire Git repositories:

```bash
# Scan local filesystem specifically for hardcoded credentials
trivy fs --scanners secret --format json --output .playhouse/reports/secrets.json .

# Scan entire Git repository history for exposed secrets
trivy repo --scanners secret https://github.com/example/repository.git
```

### 3. Container and IaC Configuration Auditing
Before shipping deployment artifacts, agents verify container Dockerfiles and Kubernetes manifests for structural security weaknesses:

```bash
# Scan Docker image for OS and library vulnerabilities
trivy image --severity CRITICAL --format json --output .playhouse/reports/image-vulns.json my-app:latest

# Scan Infrastructure as Code configurations
trivy config --format json --output .playhouse/reports/iac-config.json ./terraform/
```

### 4. SBOM Generation
To support enterprise supply chain security compliance, Playhouse CLI can generate structured Software Bill of Materials reports:

```bash
# Generate CycloneDX SBOM for project dependencies
trivy fs --format cyclonedx --output .playhouse/reports/sbom.json .
```

## Automated Self-Healing Loops and JSON Diagnostic Parsing

When Trivy detects a critical dependency vulnerability or hardcoded secret, returning thousands of lines of raw terminal text to an AI agent is inefficient. Playhouse CLI parses `trivy-results.json` and extracts concise, high-priority remediation prompts.

### Example Diagnostic Parsing
If Trivy identifies a critical CVE in an npm package and a hardcoded API token in a source file, Playhouse CLI generates the following structured diagnostic payload:

```json
{
  "status": "static_security_failure",
  "engine": "trivy",
  "summary": {
    "vulnerabilities": 1,
    "secrets": 1,
    "misconfigurations": 0
  },
  "violations": [
    {
      "type": "Vulnerability",
      "target": "package-lock.json",
      "pkgName": "axios",
      "installedVersion": "0.21.1",
      "fixedVersion": "0.21.4",
      "severity": "CRITICAL",
      "cveId": "CVE-2021-3749",
      "title": "Regular Expression Denial of Service (ReDoS)",
      "remediation": "Upgrade package axios from version 0.21.1 to fixed version 0.21.4 in package.json and regenerate package-lock.json."
    },
    {
      "type": "Secret",
      "target": "src/services/payment.ts",
      "startLine": 14,
      "endLine": 14,
      "ruleId": "stripe-access-token",
      "severity": "CRITICAL",
      "title": "Stripe Secret Key Detected",
      "match": "sk_live_51M*********",
      "remediation": "Remove hardcoded Stripe secret key from source code immediately. Move credential to environment variable STRIPE_SECRET_KEY and rotate the compromised key in the Stripe dashboard."
    }
  ]
}
```

## Best Practices for Agentic Supply Chain Security

To ensure software integrity without causing unnecessary build failures, Playhouse CLI enforces these static security best practices:

1. Immediate Build Blocking on Exposed Secrets: Any hardcoded secret or credential detected by Trivy results in an immediate build failure (exit code `4`). AI agents are instructed to replace hardcoded strings with environment variable references (`process.env.VARIABLE_NAME`) before proceeding.
2. Automated Dependency Patching: When Trivy reports a dependency vulnerability with a known `fixedVersion`, Playhouse CLI instructs the AI agent to update the relevant manifest file (`package.json`, `Cargo.toml`, `requirements.txt`) and run package installation tests automatically.
3. False Positive Governance via Exclusion Rules: Configure a `.trivyignore` file in the project root to whitelist documented test fixtures, mock certificates, or non-exploitable transitive dependencies, keeping verification scorecards clean and actionable.
4. Shift-Left Execution: Enforce `trivy fs .` execution as a pre-commit hook and within the `playhouse verify` core suite so that insecure containers and vulnerable packages are caught on developer workstations before reaching CI/CD pipelines.
