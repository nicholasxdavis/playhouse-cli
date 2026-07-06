<p align="center">
  <img src="icon/playhouse-logo.png" alt="Playhouse" width="360">
</p>
<p align="center">
   <a href="../README.md">English</a> •
   <a href="README.es.md">Español</a> •
   <a href="README.fr.md">Français</a> •
   <b>Deutsch</b> •
   <a href="README.it.md">Italiano</a> •
   <a href="README.pt.md">Português</a> •
   <a href="README.ru.md">Русский</a> •
   <a href="README.zh.md">中文</a> •
   <a href="README.ja.md">日本語</a> •
   <a href="README.ko.md">한국어</a>
</p>
<p align="center">
   <img src="https://img.shields.io/github/actions/workflow/status/nicholasxdavis/playhouse-cli/ci.yml?branch=main&style=for-the-badge&label=CI&color=2bba68" alt="CI-Status">
   <img src="https://img.shields.io/npm/v/@nicholasxdavis/playhouse-cli?style=for-the-badge&label=Version&color=14949c" alt="npm-Version">
   <img src="https://img.shields.io/github/v/release/nicholasxdavis/playhouse-cli?style=for-the-badge&label=Release&color=14949c" alt="GitHub-Release">
   <img src="https://img.shields.io/badge/Lizenz-MIT-14949c.svg?style=for-the-badge" alt="MIT-Lizenz">
</p>

<p align="center">
  QA-CLI für Sicherheit, funktionale Tests, Performance-Audits und Agenten-Übergabe.<br>
  Läuft headless über die Shell, CI oder Agenten-Tools. Optionale TUI für Menschen.
</p>

<p align="center">
  <img src="icon/tui-7-6.png?raw=true" alt="Playhouse CLI Demo" width="75%">
</p>

## Installation

**Empfohlen:**

```bash
npm install -g @nicholasxdavis/playhouse-cli
playhouse install --full   # Trivy, Arkenar, Playwright, Lighthouse, Chromium
playhouse init             # Richtet .playhouse/ und den Agenten-Skill ein
```

Entwicklungs-Abhängigkeit:

```bash
pnpm add -D @nicholasxdavis/playhouse-cli
npx playhouse doctor
```

| Profil | Befehl | Installiert |
|--------|--------|-------------|
| Vollständig (Standard) | `playhouse install` oder `--full` | Trivy, Arkenar, Playwright, Lighthouse, Chromium |
| Minimal | `playhouse install --minimal` | Nur Trivy + Arkenar |

Optionaler Projekt-Hook:

```json
"scripts": {
  "postinstall": "playhouse-install-tools"
}
```

Setzen Sie `PLAYHOUSE_INSTALL_STRICT=1`, damit `npm install` fehlschlägt, wenn die Tool-Installation fehlschlägt.

| Methode | Befehl |
|--------|---------|
| npm (primär) | `npm i -g @nicholasxdavis/playhouse-cli` |
| npx | `npx @nicholasxdavis/playhouse-cli init` |
| GitHub-Release | [Neueste Binaries](https://github.com/nicholasxdavis/playhouse-cli/releases/latest) |
| cargo (Entwickler) | `cargo install --path . --force` |
| manuelles Binary | Setzen Sie `PLAYHOUSE_BIN` oder `PLAYHOUSE_SKIP_DOWNLOAD=1` |

Das npm-Paket lädt das native Binary (~12 MB) von GitHub-Releases während `postinstall` herunter. Aktuelles Release: **v0.1.2**.

**Aus der Quelle:**

```bash
git clone https://github.com/nicholasxdavis/playhouse-cli.git
cd playhouse-cli
cargo build --release
cargo install --path . --force
```

Lokale npm-Entwicklungs-Schleife:

```bash
cargo build --release
cd packages/playhouse && npm run link-local   # bash
# Unter PowerShell 5.x: cd packages/playhouse; npm run link-local
```

**CI:** [GitHub Actions](https://github.com/nicholasxdavis/playhouse-cli/actions) führt `cargo test`, npm-Smoke-Tests und `playhouse verify` auf Ubuntu, macOS und Windows aus.

**Releases:** Pushen Sie das Tag `v0.1.2` (muss mit `Cargo.toml` und `packages/playhouse/package.json` übereinstimmen), um Binaries für npm postinstall freizugeben.

## Schnellstart

```bash
playhouse                    # TUI (Menschen)
playhouse doctor             # Tools überprüfen
playhouse verify --json      # Vollständige QA-Suite + 0-100 Sterne-Bewertung
playhouse agent --json       # Agenten-Manifest (zuerst lesen)
playhouse upgrade --json     # Auf Updates bei GitHub und npm prüfen
```

**Browser-Audits (Lighthouse, Arkenar) benötigen eine URL:**

```bash
playhouse config set default_url http://localhost:3000
```

Playhouse liest auch Port-Hinweise aus `package.json`, Vite und der Wrangler-Konfiguration und prüft dann gängige Entwicklungs-Ports.

**Monorepos:**

```bash
playhouse config set scan_root apps/web
playhouse config set test_root apps/web
```

## Für Agenten

Lesen Sie zuerst **[AGENTS.md](../AGENTS.md)**. `playhouse init` installiert **`.playhouse/SKILL.md`** und andere Workspace-Dateien.

```bash
playhouse skill install
playhouse skill status --json
```

## Befehle

| Befehl | Beschreibung |
|---------|-------------|
| `playhouse agent [--json]` | Vollständiges Agenten-Manifest |
| `playhouse agent status` | Schneller Gesundheitszustand + nächste Aktionen |
| `playhouse agent plan` | Phasenbasierter Workflow |
| `playhouse agent handoff` | Überprüfen + Handoff-Bundle exportieren |
| `playhouse verify` | Trivy + Funktional + Arkenar + Lighthouse |
| `playhouse test list\|init\|run` | Testvorlagen |
| `playhouse score` | Playhouse Stars (0-100) |
| `playhouse doctor` | Tool-Gesundheitszustand |
| `playhouse install` | Automatische Installation der gebündelten Tools |
| `playhouse config` | Einstellungen (get/set/schema) |
| `playhouse skill` | Verwaltung von `.playhouse/SKILL.md` |
| `playhouse upgrade` | Auf neuere Releases bei GitHub / npm prüfen |

Siehe auch: [stars.md](../stars.md), [playwright.md](../playwright.md), [lighthouse.md](../lighthouse.md), [trivy.md](../trivy.md), [arkenar.md](../arkenar.md), [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md).

## Playhouse Stars

Kombinierter 0-100 Audit-Score nach Überprüfung. Bericht unter `.playhouse/reports/score.json`. Standard-Erfolgsgrenze: 75.

## Projektstruktur

```
dein-projekt/
  .playhouse/
    SKILL.md
    BRIEF.md
    AGENT.json
    config.json
    reports/score.json
```

## Paketmanager

Playwright und Lighthouse verwenden npm, pnpm, yarn oder bun:

```bash
playhouse config set package_manager pnpm
```

## Exit-Codes

| Code | Bedeutung |
|------|---------|
| 0 | Bestanden |
| 1 | Fehler |
| 2 | Lighthouse unter dem Schwellenwert |
| 3 | Arkenar-Befunde |
| 4 | Trivy-Befunde |
| 5 | Tool fehlt |
