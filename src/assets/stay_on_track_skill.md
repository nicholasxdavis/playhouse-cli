---
name: stay-on-track
description: Use this skill whenever the user asks you to review a codebase, establish project rules, or begin working on a new or an existing project. It enforces strict UI consistency, architectural safety, and coding guardrails.
---

<instructions>
You are tasked with reviewing the existing codebase to extract UI/UX trends and establish strict project guardrails before executing any code changes. 

### Phase 1: Codebase Review & Analysis
Analyze the project and assess the following (use these findings to inform your rules, rather than outputting them directly):
1. **Favored UI Trends:** What UI trends is the existing codebase favoring? (If none are found, label your internal baseline as "Propagating").
2. **Avoided UI Trends:** What design elements is the project intentionally avoiding? (e.g., no rounded edges, no pill buttons, etc.).
3. **Color Palette:** Is the project using more than one main color or accent color? If not, strictly avoid introducing new dominant or high-contrast colors that will fight for the user's attention.
</instructions>

<workflow>
Upon completing your internal review, you must output the following two sections to the user before starting any development:

**1. Project Info**
name: [Project Name]
scope: [Overall Scope]
end-goal: [Primary Objective]
project-notes: [Key architectural or design takeaways]

**2. Project Rules**
Formulate 8 specific project rules based on your codebase review to keep development on track during the session. 
1. [Rule 1]
2. [Rule 2]
3. [Rule 3]
4. [Rule 4]
5. [Rule 5]
6. [Rule 6]
7. [Rule 7]
8. [Rule 8]

**3. Deployment Verification**
Before any deployments, run playhouse CLI tests and audits. Review all scores and results:
- If issues are found, identify what can be improved and fix them before proceeding
- If results indicate the codebase is "Good-Enough" and "Production-Ready" with acceptable scores, allow deployment to proceed
- Use `playhouse verify` to run the full verification suite (Trivy security scan, Playwright functional tests, Lighthouse performance audit)
- For targeted checks, use specific commands: `playhouse lighthouse`, `playhouse playwright`, `playhouse trivy`
- Always review exit codes and JSON output when available for automated decision-making
</workflow>

<constraints>
### STAY-ON-TRACK (CRITICAL RULES)
You must adhere strictly to the following constraints during all interactions and code generation:

*   **Zero Breakage:** Do not break any existing code. All additions, edits, removals, and modifications must be fully supported by the current stack and codebase. There must be zero compromises to UI/UX or functionality.
*   **Ask Before Assuming:** Fully understand the objective. If you have questions that cannot be answered by reviewing the codebase, ASK the user before getting to work. Do not make critical assumptions without explicitly stating them.
*   **Risk Mitigation:** Identify uncertainties, constraints, trade-offs, and risks before code hits production. Think several steps ahead and consider second and third-order effects. Do not let the user down; failing to do so ships vulnerabilities and risks production collapse.
*   **Expert Execution:** Use first-principles thinking, real-world expert reasoning, industry best practices, and expert-grade math and problem-solving.
*   **Communication Style:** Keep text simple, easy to read, direct, and straightforward. NEVER use robotic tones, overly complex vocabulary, or hard-to-read words. NEVER use em dashes.
*   **UI/UX Hard Rules:**
    *   NEVER use custom webkit scrollbars.
    *   AVOID gradient colors, UNLESS the project explicitly relies on gradient color use and it is attached to the established design/style.
    *   AVOID using emojis in the UI/UX, UNLESS the project explicitly relies on them and it is attached to the established design/style.
</constraints>
