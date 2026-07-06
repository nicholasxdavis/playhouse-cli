<p align="center">
  <img src="icon/playhouse-logo.png" alt="Playhouse" width="360">
</p>
<p align="center">
   <a href="../README.md">English</a> •
   <b>Español</b> •
   <a href="README.fr.md">Français</a> •
   <a href="README.de.md">Deutsch</a> •
   <a href="README.it.md">Italiano</a> •
   <a href="README.pt.md">Português</a> •
   <a href="README.ru.md">Русский</a> •
   <a href="README.zh.md">中文</a> •
   <a href="README.ja.md">日本語</a> •
   <a href="README.ko.md">한국어</a>
</p>
<p align="center">
   <img src="https://img.shields.io/github/actions/workflow/status/nicholasxdavis/playhouse-cli/ci.yml?branch=main&style=for-the-badge&label=CI&color=2bba68" alt="Estado de CI">
   <img src="https://img.shields.io/npm/v/@nicholasxdavis/playhouse-cli?style=for-the-badge&label=Versión&color=14949c" alt="versión npm">
   <img src="https://img.shields.io/github/v/release/nicholasxdavis/playhouse-cli?style=for-the-badge&label=Publicación&color=14949c" alt="Publicación de GitHub">
   <img src="https://img.shields.io/badge/Licencia-MIT-14949c.svg?style=for-the-badge" alt="Licencia MIT">
</p>

<p align="center">
  CLI de control de calidad (QA) para seguridad, pruebas funcionales, auditorías de rendimiento y transferencia a agentes.<br>
  Se ejecuta de forma headless desde la terminal, CI o herramientas de agentes. TUI opcional para humanos.
</p>

<p align="center">
  <img src="icon/tui-7-6.png?raw=true" alt="Demostración de Playhouse CLI" width="75%">
</p>

## Instalación

**Recomendado:**

```bash
npm install -g @nicholasxdavis/playhouse-cli
playhouse install --full   # Trivy, Arkenar, Playwright, Lighthouse, chromium
playhouse init             # configura .playhouse/ y la habilidad de agente
```

Dependencia de desarrollo:

```bash
pnpm add -D @nicholasxdavis/playhouse-cli
npx playhouse doctor
```

| Perfil | Comando | Instala |
|--------|---------|---------|
| Completo (por defecto) | `playhouse install` o `--full` | Trivy, Arkenar, Playwright, Lighthouse, chromium |
| Mínimo | `playhouse install --minimal` | Trivy + Arkenar únicamente |

Hook opcional de proyecto:

```json
"scripts": {
  "postinstall": "playhouse-install-tools"
}
```

Establezca `PLAYHOUSE_INSTALL_STRICT=1` para fallar `npm install` si falla la instalación de la herramienta.

| Método | Comando |
|--------|---------|
| npm (primario) | `npm i -g @nicholasxdavis/playhouse-cli` |
| npx | `npx @nicholasxdavis/playhouse-cli init` |
| GitHub Release | [Últimos binarios](https://github.com/nicholasxdavis/playhouse-cli/releases/latest) |
| cargo (desarrolladores) | `cargo install --path . --force` |
| binario manual | establezca `PLAYHOUSE_BIN` o `PLAYHOUSE_SKIP_DOWNLOAD=1` |

El paquete npm descarga el binario nativo (~12 MB) desde las publicaciones de GitHub en `postinstall`. Publicación actual: **v0.1.2**.

**Desde el código fuente:**

```bash
git clone https://github.com/nicholasxdavis/playhouse-cli.git
cd playhouse-cli
cargo build --release
cargo install --path . --force
```

Bucle de desarrollo local de npm:

```bash
cargo build --release
cd packages/playhouse && npm run link-local   # bash
# En PowerShell 5.x: cd packages/playhouse; npm run link-local
```

**CI:** [GitHub Actions](https://github.com/nicholasxdavis/playhouse-cli/actions) ejecuta `cargo test`, pruebas de humo npm y `playhouse verify` en Ubuntu, macOS y Windows.

**Publicaciones:** envíe la etiqueta `v0.1.2` (debe coincidir con `Cargo.toml` y `packages/playhouse/package.json`) para publicar binarios para el postinstall de npm.

## Inicio rápido

```bash
playhouse                    # TUI (humanos)
playhouse doctor             # comprobar herramientas
playhouse verify --json      # conjunto completo de QA + calificación de 0 a 100 estrellas
playhouse agent --json       # manifiesto de agente (leer primero)
playhouse upgrade --json     # comprobar actualizaciones en GitHub y npm
```

**Las auditorías de navegador (Lighthouse, Arkenar) necesitan una URL:**

```bash
playhouse config set default_url http://localhost:3000
```

Playhouse también lee pistas de puertos de `package.json`, Vite y la configuración de Wrangler, y luego sondea puertos de desarrollo comunes.

**Monorepositorios:**

```bash
playhouse config set scan_root apps/web
playhouse config set test_root apps/web
```

## Para agentes

Lea **[AGENTS.md](../AGENTS.md)** primero. `playhouse init` instala **`.playhouse/SKILL.md`** y otros archivos del espacio de trabajo.

```bash
playhouse skill install
playhouse skill status --json
```

## Comandos

| Comando | Descripción |
|---------|-------------|
| `playhouse agent [--json]` | Manifiesto de agente completo |
| `playhouse agent status` | Estado de salud rápido + próximas acciones |
| `playhouse agent plan` | Flujo de trabajo en fases |
| `playhouse agent handoff` | Verificar + exportar el paquete de transferencia |
| `playhouse verify` | Trivy + funcional + Arkenar + Lighthouse |
| `playhouse test list\|init\|run` | Plantillas de prueba |
| `playhouse score` | Estrellas Playhouse (0-100) |
| `playhouse doctor` | Estado de salud de las herramientas |
| `playhouse install` | Instalación automática de herramientas empaquetadas |
| `playhouse config` | Configuración (get/set/schema) |
| `playhouse skill` | Gestión de `.playhouse/SKILL.md` |
| `playhouse upgrade` | Comprobar GitHub / npm para nuevas publicaciones |

Consulte también: [stars.md](../stars.md), [playwright.md](../playwright.md), [lighthouse.md](../lighthouse.md), [trivy.md](../trivy.md), [arkenar.md](../arkenar.md), [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md).

## Estrellas Playhouse

Puntuación de auditoría combinada de 0 a 100 después de la verificación. Informe en `.playhouse/reports/score.json`. Umbral de aprobación por defecto: 75.

## Diseño del proyecto

```
tu-proyecto/
  .playhouse/
    SKILL.md
    BRIEF.md
    AGENT.json
    config.json
    reports/score.json
```

## Gestores de paquetes

Playwright y Lighthouse usan npm, pnpm, yarn o bun:

```bash
playhouse config set package_manager pnpm
```

## Códigos de salida

| Código | Significado |
|------|---------|
| 0 | Aprobado |
| 1 | Fallo |
| 2 | Lighthouse por debajo del umbral |
| 3 | Hallazgos de Arkenar |
| 4 | Hallazgos de Trivy |
| 5 | Falta una herramienta |
