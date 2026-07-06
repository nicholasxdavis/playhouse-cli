# Quality and Performance Engine: Lighthouse CI and Flow Integration

> **Status:** Partially implemented. Playhouse runs bundled `lighthouse` from `.playhouse/npm` (or project `node_modules`) after `playhouse install --full`. Runtime `npx` fallback removed. Lighthouse CI (`@lhci/cli`), multi-run medians, and Lighthouse User Flows via Playwright CDP described below are **planned**, not shipped yet.

## Overview

Playhouse CLI integrates Google Lighthouse and Lighthouse CI (`@lhci/cli`) to enforce web performance standards, accessibility compliance, search engine optimization (SEO), and technical best practices. While functional end-to-end tests confirm that features work correctly, Lighthouse auditing guarantees that applications deliver fast, accessible, and high-quality user experiences. By wrapping Lighthouse CI and extending it with interactive Flow auditing, Playhouse CLI provides automated agents with quantitative scorecards and explicit failure diagnostics to prevent performance regressions before production deployment.

## Lighthouse CLI versus Lighthouse CI CLI

To build reliable automated verification pipelines, Playhouse CLI distinguishes between the two core tools in the Lighthouse ecosystem:

### 1. Standard Lighthouse CLI (`lighthouse`)
Designed for single-pass manual audits and quick ad-hoc inspection of individual static URLs. While useful for rapid exploration, standard Lighthouse runs can exhibit score variance due to temporary system CPU load or network fluctuations.

### 2. Lighthouse CI CLI (`@lhci/cli`)
Designed specifically for automated testing, continuous integration, and regression prevention. Lighthouse CI executes multiple consecutive audit runs against target pages, aggregates the variance into statistically reliable medians, and evaluates the results against strict performance budgets. Playhouse CLI relies primarily on `@lhci/cli` to govern quality gates.

## The Game Changer: Lighthouse User Flows via Playwright

Standard Lighthouse CI configurations only evaluate static URLs during initial page load. In modern web applications and single-page applications (SPAs), critical performance bottlenecks and layout shifts occur *after* the page loads during user interactions such as client-side routing, opening shopping cart drawers, filling multi-step checkout forms, and triggering dynamic modals.

Playhouse CLI bridges this gap by integrating **Lighthouse User Flows**. By launching Playwright with a remote debugging port (`--remote-debugging-port=9222`), Playhouse CLI connects Lighthouse directly to the active Playwright browser instance via the Chrome DevTools Protocol (CDP). This allows agents to audit full interactive user journeys across three distinct operational modes:

### 1. Navigation Audits
Measures standard page load metrics, time to first byte, and initial rendering performance when navigating to a new URL.

### 2. Timespan Audits
Measures performance metrics over a specific duration of time while user interactions occur. This captures Total Blocking Time (TBT), Cumulative Layout Shift (CLS), and JavaScript execution overhead caused by button clicks, form typing, menu animations, and DOM insertions.

### 3. Snapshot Audits
Evaluates accessibility, SEO, and best practices on a specific, frozen UI state at a single point in time, such as an open modal dialog, an expanded dropdown menu, or a multi-step form wizard screen.

## Provisioning and Installation

During project setup, Playhouse CLI installs the required Lighthouse command line utilities, user flow bridges, and headless Chrome drivers automatically:

```bash
# Global installation of Lighthouse CI CLI
npm install -g @lhci/cli@latest

# Optional global installation of standard Lighthouse CLI for ad-hoc inspection
npm install -g lighthouse

# Provision Playwright-Lighthouse flow bridge libraries
npm install --save-dev @thecollege/playwright-lighthouse-flow

# Verification of CLI availability
lhci --version
```

## Configuration and Performance Budgets

Playhouse CLI generates a comprehensive `lighthouserc.js` configuration file in the project root. This configuration establishes strict quality thresholds, defines user flow targets, and instructs the engine to execute multi-pass audits in headless browser mode:

```javascript
module.exports = {
  ci: {
    collect: {
      url: ['http://localhost:3000/', 'http://localhost:3000/checkout'],
      numberOfRuns: 3,
      settings: {
        chromeFlags: '--headless --no-sandbox --disable-gpu',
      },
    },
    assert: {
      assertions: {
        'categories:performance': ['error', { minScore: 0.90 }],
        'categories:accessibility': ['error', { minScore: 1.0 }],
        'categories:best-practices': ['error', { minScore: 0.95 }],
        'categories:seo': ['error', { minScore: 1.0 }],
        'first-contentful-paint': ['warn', { maxNumericValue: 1800 }],
        'largest-contentful-paint': ['error', { maxNumericValue: 2500 }],
        'cumulative-layout-shift': ['error', { maxNumericValue: 0.1 }],
        'total-blocking-time': ['error', { maxNumericValue: 300 }],
        'resource-summary:script:size': ['error', { maxNumericValue: 350000 }],
      },
    },
    upload: {
      target: 'temporary-public-storage',
    },
  },
};
```

## Core Audit Workflows

Playhouse CLI orchestrates Lighthouse verification through simple, standardized command wrappers:

### 1. Automated Regression Auditing
Agents execute full multi-run verification suites against local development servers or staging environments:

```bash
# Run Lighthouse CI audit suite against configured targets
npx lhci autorun --config=./lighthouserc.js
```

### 2. Interactive User Flow Auditing
Agents execute a Playwright script that triggers Lighthouse flow measurements during live DOM interactions:

```typescript
import { test, chromium } from '@playwright/test';
import { playhouseFlow } from '@playwright-playhouse/flow';

test('audit checkout user journey performance', async () => {
  const browser = await chromium.launch({ args: ['--remote-debugging-port=9222'] });
  const page = await browser.newPage();
  const flow = await playhouseFlow(page, { name: 'Checkout Journey' });

  // Navigation Audit: Initial load
  await flow.navigate('http://localhost:3000/');

  // Timespan Audit: Measure performance during cart interaction
  await flow.startTimespan({ stepName: 'Add to Cart and Open Drawer' });
  await page.getByRole('button', { name: 'Add to Cart' }).click();
  await page.getByTestId('cart-drawer').waitFor();
  await flow.endTimespan();

  // Snapshot Audit: Verify accessibility of the open drawer
  await flow.snapshot({ stepName: 'Cart Drawer State' });

  await flow.generateReport('.playhouse/reports/lighthouse/checkout-flow.json');
  await browser.close();
});
```

### 3. Ad-Hoc URL Profiling
When an agent creates a new page or optimizes a specific UI component, it can generate a targeted JSON scorecard:

```bash
# Execute single headless audit and output structured JSON report
lighthouse http://localhost:3000/new-feature --output=json --output-path=./.playhouse/reports/audit.json --chrome-flags="--headless"
```

## Automated Self-Healing Loops for AI Agents

When Lighthouse CI detects a budget violation or accessibility failure, returning raw HTML reports or verbose logs to an AI agent is inefficient. Playhouse CLI parses the output artifacts and extracts specific diagnostic rule violations into structured remediation prompts.

### Example Diagnostic Parsing
If an audit fails due to poor Largest Contentful Paint (LCP) and accessibility errors, Playhouse CLI extracts the exact failing audit identifiers and feeds actionable instructions back to the agent:

```json
{
  "status": "failed",
  "scorecard": {
    "performance": 0.82,
    "accessibility": 0.91
  },
  "violations": [
    {
      "auditId": "offscreen-images",
      "title": "Defer offscreen images",
      "savingsMs": 650,
      "targets": ["/assets/hero-banner.png", "/assets/product-grid.jpg"],
      "remediation": "Implement native HTML loading=\"lazy\" attribute or convert images to modern WebP/AVIF formats."
    },
    {
      "auditId": "color-contrast",
      "title": "Background and foreground colors do not have a sufficient contrast ratio",
      "targets": ["button.secondary-cta"],
      "remediation": "Adjust button text color to meet WCAG AA minimum contrast ratio of 4.5:1."
    },
    {
      "auditId": "unused-javascript",
      "title": "Reduce unused JavaScript",
      "savingsBytes": 145000,
      "targets": ["/static/js/bundle.main.js"],
      "remediation": "Implement dynamic import() code splitting for analytics and heavy charting widgets."
    }
  ]
}
```

## Best Practices for Agentic Performance Engineering

To maintain high scores without causing continuous build failures, Playhouse CLI guides agents using the following engineering principles:

1. Eliminate Flakiness via Multi-Run Averaging: Always configure `numberOfRuns` to at least 3. Network jitter and background operating system processes can cause variance of 5 to 10 points on single runs.
2. Target Core Web Vitals First: Focus agent optimization efforts on Largest Contentful Paint (LCP), Cumulative Layout Shift (CLS), and Total Blocking Time (TBT). These metrics directly impact SEO rankings and real-world user experience.
3. Zero Tolerance for Accessibility Violations: While performance scores can fluctuate slightly, accessibility rules (`aria-labels`, color contrast, image alt text, keyboard navigation) are deterministic. Playhouse CLI enforces a strict 100 percent passing threshold for accessibility audits across all user flow snapshots.
4. Bundle Size Management: Configure explicit JavaScript bundle size assertions in `lighthouserc.js` to prevent agents from importing massive third-party libraries for simple utility tasks.
