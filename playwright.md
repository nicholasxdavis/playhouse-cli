# Functional and Visual Verification Engine: Playwright CLI Integration

> **Status:** Partially implemented. Playhouse runs `@playwright/test` with JSON reporter for `playhouse verify` and scoring. Interactive Playwright CLI agent workflows, semantic accessibility distillation, and visual regression described below are **planned**, not shipped yet.

## Overview

Playhouse CLI integrates `@playwright/cli` as its primary engine for end-to-end functional verification, visual layout testing, user journey simulation, and automated regression prevention. While traditional automated testing requires manually writing verbose, brittle scripts, Playhouse CLI exposes Playwright's interactive command line interface directly to AI coding agents. This architecture allows agents to navigate web applications, interact with DOM elements, verify visual rendering across screen sizes, and assert complex business logic in real time.

## Why Playwright CLI for AI Agents

When designing agentic coding workflows, token efficiency and deterministic execution are paramount. Sending full HTML DOM trees or extensive Chrome DevTools console logs to a Large Language Model consumes thousands of tokens per step and introduces hallucinations due to structural noise.

Playhouse CLI resolves this through **Semantic Accessibility Distillation**. When an agent requests a browser snapshot, the CLI compresses the page layout by stripping decorative styling, wrapper divs, and non-interactive scripts. It exposes only accessible interactive components, assigning each element a concise alphanumeric reference (for example, `e1`, `e2`, `e15`). Agents execute commands against these references directly without parsing complex CSS selectors or XPath queries, reducing token overhead by over 90 percent.

## Provisioning and Installation

During project setup, Playhouse CLI verifies the Playwright CLI ecosystem and installs required browser dependencies automatically:

```bash
# Verify local CLI availability
npx --no-install playwright-cli --version

# Global installation command executed during initialization
npm install -g @playwright/cli@latest

# Provision required headless browser binaries
npx playwright install --with-deps chromium firefox webkit
```

## Core Agent Workflows and Commands

Playhouse CLI wraps and orchestrates fundamental Playwright CLI operations to simulate real user behavior deterministically:

### 1. Session Initialization and Profile Persistence
Agents launch isolated browser sessions or connect to persistent user profiles to maintain login credentials and cached storage across test runs:

```bash
# Launch a default headless Chromium session
playwright-cli open https://staging.example.com

# Launch a persistent session named "admin-suite" using Chrome
playwright-cli -s=admin-suite open https://staging.example.com --browser=chrome --persistent

# Navigate within an active session
playwright-cli goto https://staging.example.com/checkout
```

### 2. Element Interaction via Snapshot References
After any navigation or interaction, the CLI outputs a distilled snapshot containing interactive element references. Agents use these references to simulate input:

```bash
# Click a button or link referenced as e15 in the snapshot
playwright-cli click e15

# Fill an input field and submit the form immediately
playwright-cli fill e5 "test-user@example.com" --submit

# Select an option from a dropdown menu
playwright-cli select e9 "premium-tier"

# Check a checkbox or toggle switch
playwright-cli check e12
```

### 3. State Persistence and Authentication
To eliminate redundant login steps during multi-stage testing, Playhouse CLI allows agents to capture and restore browser cookies and local storage state:

```bash
# Save authentication state after logging in
playwright-cli state-save .playhouse/auth-state.json

# Load authentication state in subsequent test runs
playwright-cli state-load .playhouse/auth-state.json

# Programmatically verify or modify local storage
playwright-cli localstorage-set theme dark
```

### 4. Network Interception and API Mocking
Agents isolate frontend components from backend instabilities by mocking API endpoints and blocking third-party trackers:

```bash
# Mock a backend API response for deterministic testing
playwright-cli route "https://api.example.com/v1/user*" --body='{"id": 101, "name": "Verified User", "status": "active"}'

# Abort requests for heavy media assets to accelerate test execution
playwright-cli route "**/*.{png,jpg,mp4}" --status=404
```

### 5. Multi-Tab Orchestration
For complex workflows such as OAuth redirects or multi-window administrative dashboards, agents manage multiple browser tabs seamlessly:

```bash
# Open a new tab and navigate to a secondary workflow
playwright-cli tab-new https://staging.example.com/portal

# List all active tabs and their corresponding index IDs
playwright-cli tab-list

# Switch context back to the primary tab
playwright-cli tab-select 0
```

## Advanced Visual Regression Testing

Functional end-to-end tests often pass even when CSS layouts are broken by overlapping modals, zero-opacity containers, or responsive z-index collisions. Playhouse CLI incorporates an advanced visual regression engine:

### 1. Visual Snapshot Verification
Playhouse CLI allows agents to capture visual baseline screenshots and perform pixel-level comparisons against future code builds using Playwright's native visual assertions:

```typescript
// Capture visual snapshot and assert against baseline
await expect(page).toHaveScreenshot('checkout-summary.png', {
  maxDiffPixels: 100,
});
```

### 2. Automated Dynamic Content Masking
Visual tests frequently suffer from flakiness caused by dynamic data such as live timestamps, user avatars, randomized transaction IDs, or ad banners. Playhouse CLI provides an automated masking flag that hides dynamic DOM elements before capturing screenshots:

```bash
# Execute visual comparison while masking dynamic time and ID containers
playwright-cli --raw run-code "await expect(page).toHaveScreenshot({ mask: [page.locator('.live-timestamp'), page.locator('.transaction-id')] })"
```

When a visual regression occurs, Playhouse CLI generates a side-by-side image diff (`.playhouse/reports/visual-diffs/checkout-summary-diff.png`) and exposes the localized file path so the agent can inspect the layout failure.

## Hybrid Semantic Self-Healing Architecture

A primary bottleneck in autonomous agent development is test suite brittleness. When an agent refactors application UI, DOM selectors change, causing tests to fail. Playhouse CLI implements a game-changing **Hybrid Semantic Self-Healing Engine** that classifies test failures into six distinct categories: selectors, timing, runtime errors, test data, visual assertions, and interaction steps.

When a locator failure occurs, the engine executes a two-layer healing sequence:

### Layer 1: Deterministic Fuzzy Matching
The engine first scans the DOM for invariant attributes. It evaluates element test IDs (`data-testid`), ARIA roles, accessible names, and input names to locate the relocated element without relying on AI inference.

### Layer 2: AI Semantic Intent Analysis
If deterministic matching fails, the engine analyzes the surrounding DOM context to determine the semantic intent of the original step (for example, identifying that step 4 intended to click the primary checkout submission button). It evaluates candidate buttons and assigns a confidence score to the top match.

### The Strict Confidence Gate
To prevent silent false positives and erroneous test modifications, Playhouse CLI enforces a strict **0.75 Confidence Gate**:
* Confidence >= 0.75: The engine dynamically updates the target locator in memory, executes the test step successfully, and writes a refactoring record to `.playhouse/healed-locators.json`.
* Confidence < 0.75: The engine refuses to guess. It fails the test loudly, captures a trace archive and screenshot, and returns an actionable diagnostic payload to the agent.

### Technical Debt Logging
Every auto-healed locator is treated as technical debt. Playhouse CLI maintains a structured backlog of healed selectors so engineering teams and agents can update source test files during weekly code reviews:

```json
{
  "testFile": "tests/e2e/checkout-flow.spec.ts",
  "stepIndex": 4,
  "originalSelector": "button.btn-primary.submit-btn",
  "healedSelector": "getByRole('button', { name: 'Complete Order' })",
  "confidenceScore": 0.92,
  "timestamp": "2026-07-05T23:51:00Z",
  "status": "pending_source_update"
}
```

## Best Practices for Agentic Verification

To ensure test suites remain stable, maintainable, and production ready, Playhouse CLI enforces these engineering best practices:

### Spec-Driven Testing (Plan, Generate, Heal)
1. Plan: Before writing code, the agent launches an exploratory Playwright CLI session to map application workflows and identify required assertions.
2. Generate: The agent outputs modular TypeScript test files following the Page Object Model (POM) pattern, separating page structure from test logic.
3. Heal: During verification runs, the Hybrid Self-Healing engine maintains test continuity while logging necessary selector updates.

### Robust Locator Strategies
While snapshot references (`e1`, `e2`) are optimal for live exploratory sessions, permanent test suites generated by agents must rely on user-facing attributes:

```typescript
// Good: Resilient to layout changes and styling updates
await page.getByRole('button', { name: 'Submit Order' }).click();
await page.getByTestId('checkout-total').innerText();

// Avoid: Brittle selectors that break on DOM reordering
await page.locator('#main > div:nth-child(2) > ul > li.active').click();
```

## Interactive Design Review Mode

When an agent encounters underspecified UI requirements or layout ambiguity, Playhouse CLI can invoke Playwright's human-in-the-loop annotation tool:

```bash
# Open live application and invite human reviewer to annotate
playwright-cli open https://staging.example.com/new-feature
playwright-cli show --annotate
```

In this mode, the application opens on the user's desktop. The user can draw bounding boxes directly on the live webpage and type feedback notes. Once submitted, Playhouse CLI packages the annotated screenshot, region snapshot, and text notes into a structured payload and delivers it directly to the agent for immediate implementation.
