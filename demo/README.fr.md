<p align="center">
  <img src="icon/playhouse-logo.png" alt="Playhouse" width="360">
</p>
<p align="center">
   <a href="../README.md">English</a> •
   <a href="README.es.md">Español</a> •
   <b>Français</b> •
   <a href="README.de.md">Deutsch</a> •
   <a href="README.it.md">Italiano</a> •
   <a href="README.pt.md">Português</a> •
   <a href="README.ru.md">Русский</a> •
   <a href="README.zh.md">中文</a> •
   <a href="README.ja.md">日本語</a> •
   <a href="README.ko.md">한국어</a>
</p>
<p align="center">
   <img src="https://img.shields.io/github/actions/workflow/status/nicholasxdavis/playhouse-cli/ci.yml?branch=main&style=for-the-badge&label=CI&color=2bba68" alt="Statut CI">
   <img src="https://img.shields.io/npm/v/@nicholasxdavis/playhouse-cli?style=for-the-badge&label=Version&color=14949c" alt="version npm">
   <img src="https://img.shields.io/github/v/release/nicholasxdavis/playhouse-cli?style=for-the-badge&label=Publication&color=14949c" alt="Publication GitHub">
   <img src="https://img.shields.io/badge/Licence-MIT-14949c.svg?style=for-the-badge" alt="Licence MIT">
</p>

<p align="center">
  CLI d'assurance qualité (QA) pour la sécurité, les tests fonctionnels, les audits de performance et le transfert d'agent.<br>
  S'exécute en mode headless depuis le terminal, la CI ou les outils d'agent. TUI optionnelle pour les humains.
</p>

<p align="center">
  <img src="icon/tui-7-6.png?raw=true" alt="Démonstration de Playhouse CLI" width="75%">
</p>

## Installation

**Recommandé :**

```bash
npm install -g @nicholasxdavis/playhouse-cli
playhouse install --full   # Trivy, Arkenar, Playwright, Lighthouse, chromium
playhouse init             # configure .playhouse/ et la compétence d'agent
```

Dépendance de développement :

```bash
pnpm add -D @nicholasxdavis/playhouse-cli
npx playhouse doctor
```

| Profil | Commande | Installe |
|--------|----------|----------|
| Complet (par défaut) | `playhouse install` ou `--full` | Trivy, Arkenar, Playwright, Lighthouse, chromium |
| Minimal | `playhouse install --minimal` | Trivy + Arkenar uniquement |

Hook de projet optionnel :

```json
"scripts": {
  "postinstall": "playhouse-install-tools"
}
```

Définissez `PLAYHOUSE_INSTALL_STRICT=1` pour faire échouer `npm install` si l'installation des outils échoue.

| Méthode | Commande |
|---------|----------|
| npm (principal) | `npm i -g @nicholasxdavis/playhouse-cli` |
| npx | `npx @nicholasxdavis/playhouse-cli init` |
| GitHub Release | [Derniers binaires](https://github.com/nicholasxdavis/playhouse-cli/releases/latest) |
| cargo (développeurs) | `cargo install --path . --force` |
| binaire manuel | définissez `PLAYHOUSE_BIN` ou `PLAYHOUSE_SKIP_DOWNLOAD=1` |

Le paquet npm télécharge le binaire natif (~12 Mo) depuis les publications de GitHub lors du `postinstall`. Publication actuelle : **v0.1.2**.

**À partir des sources :**

```bash
git clone https://github.com/nicholasxdavis/playhouse-cli.git
cd playhouse-cli
cargo build --release
cargo install --path . --force
```

Boucle de développement npm locale :

```bash
cargo build --release
cd packages/playhouse && npm run link-local   # bash
# En PowerShell 5.x : cd packages/playhouse; npm run link-local
```

**CI :** [GitHub Actions](https://github.com/nicholasxdavis/playhouse-cli/actions) exécute `cargo test`, des tests de fumée npm et `playhouse verify` sur Ubuntu, macOS et Windows.

**Publications :** poussez la balise `v0.1.2` (doit correspondre à `Cargo.toml` et `packages/playhouse/package.json`) pour publier des binaires pour le postinstall npm.

## Démarrage rapide

```bash
playhouse                    # TUI (humains)
playhouse doctor             # vérifier les outils
playhouse verify --json      # suite QA complète + évaluation de 0 à 100 étoiles
playhouse agent --json       # manifeste de l'agent (à lire en premier)
playhouse upgrade --json     # vérifier les mises à jour sur GitHub et npm
```

**Les audits de navigateur (Lighthouse, Arkenar) nécessitent une URL :**

```bash
playhouse config set default_url http://localhost:3000
```

Playhouse lit également les indices de ports de `package.json`, de Vite et de la configuration Wrangler, puis sonde les ports de développement courants.

**Monorepos :**

```bash
playhouse config set scan_root apps/web
playhouse config set test_root apps/web
```

## Pour les agents

Lisez **[AGENTS.md](../AGENTS.md)** en premier. `playhouse init` installe **`.playhouse/SKILL.md`** et d'autres fichiers de l'espace de travail.

```bash
playhouse skill install
playhouse skill status --json
```

## Commandes

| Commande | Description |
|----------|-------------|
| `playhouse agent [--json]` | Manifeste complet de l'agent |
| `playhouse agent status` | Santé rapide + actions suivantes |
| `playhouse agent plan` | Flux de travail par étapes |
| `playhouse agent handoff` | Vérifier + exporter le bundle de transfert |
| `playhouse verify` | Trivy + fonctionnel + Arkenar + Lighthouse |
| `playhouse test list\|init\|run` | Modèles de test |
| `playhouse score` | Étoiles Playhouse (0-100) |
| `playhouse doctor` | Santé des outils |
| `playhouse install` | Installation automatique des outils intégrés |
| `playhouse config` | Paramètres (get/set/schema) |
| `playhouse skill` | Gestion de `.playhouse/SKILL.md` |
| `playhouse upgrade` | Vérifier GitHub / npm pour de nouvelles publications |

Consultez également : [stars.md](../stars.md), [playwright.md](../playwright.md), [lighthouse.md](../lighthouse.md), [trivy.md](../trivy.md), [arkenar.md](../arkenar.md), [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md).

## Étoiles Playhouse

Score d'audit combiné de 0 à 100 après vérification. Rapport dans `.playhouse/reports/score.json`. Seuil de réussite par défaut : 75.

## Structure du projet

```
votre-projet/
  .playhouse/
    SKILL.md
    BRIEF.md
    AGENT.json
    config.json
    reports/score.json
```

## Gestionnaires de paquets

Playwright et Lighthouse utilisent npm, pnpm, yarn ou bun :

```bash
playhouse config set package_manager pnpm
```

## Codes de sortie

| Code | Signification |
|------|---------|
| 0 | Réussite |
| 1 | Échec |
| 2 | Lighthouse en dessous du seuil |
| 3 | Découvertes Arkenar |
| 4 | Découvertes Trivy |
| 5 | Outil manquant |
