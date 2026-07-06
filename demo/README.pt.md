<p align="center">
  <img src="icon/playhouse-logo.png" alt="Playhouse" width="360">
</p>
<p align="center">
   <a href="../README.md">English</a> •
   <a href="README.es.md">Español</a> •
   <a href="README.fr.md">Français</a> •
   <a href="README.de.md">Deutsch</a> •
   <a href="README.it.md">Italiano</a> •
   <b>Português</b> •
   <a href="README.ru.md">Русский</a> •
   <a href="README.zh.md">中文</a> •
   <a href="README.ja.md">日本語</a> •
   <a href="README.ko.md">한국어</a>
</p>
<p align="center">
   <img src="https://img.shields.io/github/actions/workflow/status/nicholasxdavis/playhouse-cli/ci.yml?branch=main&style=for-the-badge&label=CI&color=2bba68" alt="Status do CI">
   <img src="https://img.shields.io/npm/v/@nicholasxdavis/playhouse-cli?style=for-the-badge&label=Versão&color=14949c" alt="versão do npm">
   <img src="https://img.shields.io/github/v/release/nicholasxdavis/playhouse-cli?style=for-the-badge&label=Release&color=14949c" alt="Release do GitHub">
   <img src="https://img.shields.io/badge/Licença-MIT-14949c.svg?style=for-the-badge" alt="Licença MIT">
</p>

<p align="center">
  CLI de garantia de qualidade (QA) para segurança, testes funcionais, auditorias de desempenho e transferência de agentes.<br>
  Executado de forma headless a partir do shell, CI ou ferramentas de agentes. TUI opcional para humanos.
</p>

<p align="center">
  <img src="icon/tui-7-6.png?raw=true" alt="Demonstração do Playhouse CLI" width="75%">
</p>

## Instalação

**Recomendado:**

```bash
npm install -g @nicholasxdavis/playhouse-cli
playhouse install --full   # Trivy, Arkenar, Playwright, Lighthouse, chromium
playhouse init             # configura .playhouse/ e a habilidade do agente
```

Dependência de desenvolvimento:

```bash
pnpm add -D @nicholasxdavis/playhouse-cli
npx playhouse doctor
```

| Perfil | Comando | Instala |
|--------|---------|---------|
| Completo (padrão) | `playhouse install` ou `--full` | Trivy, Arkenar, Playwright, Lighthouse, chromium |
| Mínimo | `playhouse install --minimal` | Apenas Trivy + Arkenar |

Hook de projeto opcional:

```json
"scripts": {
  "postinstall": "playhouse-install-tools"
}
```

Defina `PLAYHOUSE_INSTALL_STRICT=1` para falhar a execução de `npm install` caso a instalação das ferramentas falhe.

| Método | Comando |
|--------|---------|
| npm (primário) | `npm i -g @nicholasxdavis/playhouse-cli` |
| npx | `npx @nicholasxdavis/playhouse-cli init` |
| GitHub Release | [Últimos binários](https://github.com/nicholasxdavis/playhouse-cli/releases/latest) |
| cargo (desenvolvedores) | `cargo install --path . --force` |
| binário manual | defina `PLAYHOUSE_BIN` ou `PLAYHOUSE_SKIP_DOWNLOAD=1` |

O pacote npm baixa o binário nativo (~12 MB) dos Releases do GitHub no `postinstall`. Release atual: **v0.1.2**.

**A partir do código-fonte:**

```bash
git clone https://github.com/nicholasxdavis/playhouse-cli.git
cd playhouse-cli
cargo build --release
cargo install --path . --force
```

Ciclo de desenvolvimento npm local:

```bash
cargo build --release
cd packages/playhouse && npm run link-local   # bash
# No PowerShell 5.x: cd packages/playhouse; npm run link-local
```

**CI:** [GitHub Actions](https://github.com/nicholasxdavis/playhouse-cli/actions) executa `cargo test`, testes de fumaça npm e `playhouse verify` no Ubuntu, macOS e Windows.

**Releases:** envie a tag `v0.1.2` (deve corresponder ao `Cargo.toml` e `packages/playhouse/package.json`) para publicar binários para o postinstall do npm.

## Início rápido

```bash
playhouse                    # TUI (humanos)
playhouse doctor             # verificar ferramentas
playhouse verify --json      # conjunto completo de QA + classificação de 0-100 estrelas
playhouse agent --json       # manifesto do agente (ler primeiro)
playhouse upgrade --json     # verificar atualizações no GitHub e npm
```

**Auditorias de navegador (Lighthouse, Arkenar) precisam de uma URL:**

```bash
playhouse config set default_url http://localhost:3000
```

O Playhouse também lê dicas de porta de `package.json`, Vite e configuração do Wrangler, e depois sonda as portas de desenvolvimento comuns.

**Monorepos:**

```bash
playhouse config set scan_root apps/web
playhouse config set test_root apps/web
```

## Para agentes

Leia **[AGENTS.md](../AGENTS.md)** primeiro. O `playhouse init` instala o **`.playhouse/SKILL.md`** e outros arquivos de espaço de trabalho.

```bash
playhouse skill install
playhouse skill status --json
```

## Comandos

| Comando | Descrição |
|---------|-------------|
| `playhouse agent [--json]` | Manifesto de agente completo |
| `playhouse agent status` | Saúde rápida + próximas ações |
| `playhouse agent plan` | Fluxo de trabalho em fases |
| `playhouse agent handoff` | Verificar + exportar pacote de transferência |
| `playhouse verify` | Trivy + funcional + Arkenar + Lighthouse |
| `playhouse test list\|init\|run` | Modelos de teste |
| `playhouse score` | Estrelas Playhouse (0-100) |
| `playhouse doctor` | Saúde das ferramentas |
| `playhouse install` | Instalação automática de ferramentas empacotadas |
| `playhouse config` | Configurações (get/set/schema) |
| `playhouse skill` | Gerenciamento de `.playhouse/SKILL.md` |
| `playhouse upgrade` | Verificar GitHub / npm para novos releases |

Consulte também: [stars.md](../stars.md), [playwright.md](../playwright.md), [lighthouse.md](../lighthouse.md), [trivy.md](../trivy.md), [arkenar.md](../arkenar.md), [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md).

## Estrellas Playhouse

Pontuação de auditoria combinada de 0-100 após a verificação. Relatório em `.playhouse/reports/score.json`. Limite padrão de aprovação: 75.

## Estrutura do projeto

```
seu-projeto/
  .playhouse/
    SKILL.md
    BRIEF.md
    AGENT.json
    config.json
    reports/score.json
```

## Gerenciadores de pacotes

Playwright e Lighthouse usam npm, pnpm, yarn ou bun:

```bash
playhouse config set package_manager pnpm
```

## Códigos de saída

| Código | Significado |
|------|---------|
| 0 | Aprovado |
| 1 | Falha |
| 2 | Lighthouse abaixo do limite |
| 3 | Descobertas do Arkenar |
| 4 | Descobertas do Trivy |
| 5 | Ferramenta ausente |
