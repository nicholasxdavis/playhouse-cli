<p align="center">
  <img src="icon/playhouse-logo.png" alt="Playhouse" width="360">
</p>
<p align="center">
   <a href="../README.md">English</a> •
   <a href="README.es.md">Español</a> •
   <a href="README.fr.md">Français</a> •
   <a href="README.de.md">Deutsch</a> •
   <b>Italiano</b> •
   <a href="README.pt.md">Português</a> •
   <a href="README.ru.md">Русский</a> •
   <a href="README.zh.md">中文</a> •
   <a href="README.ja.md">日本語</a> •
   <a href="README.ko.md">한국어</a>
</p>
<p align="center">
   <img src="https://img.shields.io/github/actions/workflow/status/nicholasxdavis/playhouse-cli/ci.yml?branch=main&style=for-the-badge&label=CI&color=2bba68" alt="Stato CI">
   <img src="https://img.shields.io/npm/v/@nicholasxdavis/playhouse-cli?style=for-the-badge&label=Versione&color=14949c" alt="versione npm">
   <img src="https://img.shields.io/github/v/release/nicholasxdavis/playhouse-cli?style=for-the-badge&label=Release&color=14949c" alt="Release GitHub">
   <img src="https://img.shields.io/badge/Licenza-MIT-14949c.svg?style=for-the-badge" alt="Licenza MIT">
</p>

<p align="center">
  Interfaccia a riga di comando (CLI) per il controllo qualità (QA) per sicurezza, test funzionali, audit delle prestazioni e passaggio di consegne all'agente.<br>
  Funziona in modalità headless da terminale, CI o strumenti dell'agente. TUI opzionale per gli esseri umani.
</p>

<p align="center">
  <img src="icon/tui-7-6.png?raw=true" alt="Dimostrazione di Playhouse CLI" width="75%">
</p>

## Installazione

**Consigliato:**

```bash
npm install -g @nicholasxdavis/playhouse-cli
playhouse install --full   # Trivy, Arkenar, Playwright, Lighthouse, chromium
playhouse init             # configura .playhouse/ e l'abilità dell'agente
```

Dipendenza di sviluppo:

```bash
pnpm add -D @nicholasxdavis/playhouse-cli
npx playhouse doctor
```

| Profilo | Comando | Installa |
|--------|---------|----------|
| Completo (predefinito) | `playhouse install` o `--full` | Trivy, Arkenar, Playwright, Lighthouse, chromium |
| Minimo | `playhouse install --minimal` | Solo Trivy + Arkenar |

Hook di progetto opzionale:

```json
"scripts": {
  "postinstall": "playhouse-install-tools"
}
```

Imposta `PLAYHOUSE_INSTALL_STRICT=1` per far fallire `npm install` se l'installazione dello strumento non va a buon fine.

| Metodo | Comando |
|--------|---------|
| npm (principale) | `npm i -g @nicholasxdavis/playhouse-cli` |
| npx | `npx @nicholasxdavis/playhouse-cli init` |
| GitHub Release | [Ultimi binari](https://github.com/nicholasxdavis/playhouse-cli/releases/latest) |
| cargo (sviluppatori) | `cargo install --path . --force` |
| binario manual | imposta `PLAYHOUSE_BIN` o `PLAYHOUSE_SKIP_DOWNLOAD=1` |

Il pacchetto npm scarica il binario nativo (~12 MB) dalle Release di GitHub durante `postinstall`. Release corrente: **v0.1.2**.

**Sorgente:**

```bash
git clone https://github.com/nicholasxdavis/playhouse-cli.git
cd playhouse-cli
cargo build --release
cargo install --path . --force
```

Ciclo di sviluppo npm locale:

```bash
cargo build --release
cd packages/playhouse && npm run link-local   # bash
# In PowerShell 5.x: cd packages/playhouse; npm run link-local
```

**CI:** [GitHub Actions](https://github.com/nicholasxdavis/playhouse-cli/actions) esegue `cargo test`, test di fumo npm e `playhouse verify` su Ubuntu, macOS e Windows.

**Release:** invia il tag `v0.1.2` (deve corrispondere a `Cargo.toml` e `packages/playhouse/package.json`) per pubblicare i binari per il postinstall npm.

## Avvio rapido

```bash
playhouse                    # TUI (esseri umani)
playhouse doctor             # controllare gli strumenti
playhouse verify --json      # suite QA completa + valutazione da 0 a 100 stelle
playhouse agent --json       # manifesto dell'agente (leggere prima)
playhouse upgrade --json     # controlla gli aggiornamenti su GitHub e npm
```

**Gli audit del browser (Lighthouse, Arkenar) richiedono un URL:**

```bash
playhouse config set default_url http://localhost:3000
```

Playhouse legge anche i suggerimenti sulle porte da `package.json`, Vite e Wrangler config, e poi esegue la scansione delle porte di sviluppo comuni.

**Monorepo:**

```bash
playhouse config set scan_root apps/web
playhouse config set test_root apps/web
```

## Per gli agenti

Leggi prima **[AGENTS.md](../AGENTS.md)**. `playhouse init` installa **`.playhouse/SKILL.md`** e altri file dello spazio di lavoro.

```bash
playhouse skill install
playhouse skill status --json
```

## Comandi

| Comando | Descrizione |
|---------|-------------|
| `playhouse agent [--json]` | Manifesto completo dell'agente |
| `playhouse agent status` | Stato di salute rapido + prossime azioni |
| `playhouse agent plan` | Flusso di lavoro a fasi |
| `playhouse agent handoff` | Verifica + esporta pacchetto di consegna |
| `playhouse verify` | Trivy + funzionale + Arkenar + Lighthouse |
| `playhouse test list\|init\|run` | Modelli di test |
| `playhouse score` | Stelle Playhouse (0-100) |
| `playhouse doctor` | Stato di salute degli strumenti |
| `playhouse install` | Installazione automatica degli strumenti inclusi |
| `playhouse config` | Impostazioni (get/set/schema) |
| `playhouse skill` | Gestione di `.playhouse/SKILL.md` |
| `playhouse upgrade` | Controlla GitHub / npm per nuove release |

Vedi anche: [stars.md](../stars.md), [playwright.md](../playwright.md), [lighthouse.md](../lighthouse.md), [trivy.md](../trivy.md), [arkenar.md](../arkenar.md), [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md).

## Stelle Playhouse

Punteggio di audit combinato da 0 a 100 dopo la verifica. Rapporto su `.playhouse/reports/score.json`. Soglia di superamento predefinita: 75.

## Struttura del progetto

```
tuo-progetto/
  .playhouse/
    SKILL.md
    BRIEF.md
    AGENT.json
    config.json
    reports/score.json
```

## Gestori di pacchetti

Playwright e Lighthouse utilizzano npm, pnpm, yarn o bun:

```bash
playhouse config set package_manager pnpm
```

## Codici di uscita

| Codice | Significato |
|------|---------|
| 0 | Superato |
| 1 | Errore |
| 2 | Lighthouse sotto la soglia |
| 3 | Risultati di Arkenar |
| 4 | Risultati di Trivy |
| 5 | Strumento mancante |
