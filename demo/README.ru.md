<p align="center">
  <img src="icon/playhouse-logo.png" alt="Playhouse" width="360">
</p>
<p align="center">
   <a href="../README.md">English</a> •
   <a href="README.es.md">Español</a> •
   <a href="README.fr.md">Français</a> •
   <a href="README.de.md">Deutsch</a> •
   <a href="README.it.md">Italiano</a> •
   <a href="README.pt.md">Português</a> •
   <b>Русский</b> •
   <a href="README.zh.md">中文</a> •
   <a href="README.ja.md">日本語</a> •
   <a href="README.ko.md">한국어</a>
</p>
<p align="center">
   <img src="https://img.shields.io/github/actions/workflow/status/nicholasxdavis/playhouse-cli/ci.yml?branch=main&style=for-the-badge&label=CI&color=2bba68" alt="Статус CI">
   <img src="https://img.shields.io/npm/v/@nicholasxdavis/playhouse-cli?style=for-the-badge&label=Версия&color=14949c" alt="версия npm">
   <img src="https://img.shields.io/github/v/release/nicholasxdavis/playhouse-cli?style=for-the-badge&label=Релиз&color=14949c" alt="Релиз GitHub">
   <img src="https://img.shields.io/badge/Лицензия-MIT-14949c.svg?style=for-the-badge" alt="Лицензия MIT">
</p>

<p align="center">
  QA CLI для обеспечения безопасности, функционального тестирования, аудита производительности и передачи управления агенту.<br>
  Запускается в фоновом режиме (headless) из терминала, CI или инструментов агента. Дополнительный TUI для людей.
</p>

<p align="center">
  <img src="icon/tui-7-6.png?raw=true" alt="Демонстрация Playhouse CLI" width="75%">
</p>

## Установка

**Рекомендуется:**

```bash
npm install -g @nicholasxdavis/playhouse-cli
playhouse install --full   # Trivy, Arkenar, Playwright, Lighthouse, chromium
playhouse init             # настраивает .playhouse/ и навык агента
```

Зависимость для разработки:

```bash
pnpm add -D @nicholasxdavis/playhouse-cli
npx playhouse doctor
```

| Профиль | Команда | Устанавливает |
|--------|---------|---------------|
| Полный (по умолчанию) | `playhouse install` или `--full` | Trivy, Arkenar, Playwright, Lighthouse, chromium |
| Минимальный | `playhouse install --minimal` | Только Trivy + Arkenar |

Необязательный хук проекта:

```json
"scripts": {
  "postinstall": "playhouse-install-tools"
}
```

Установите `PLAYHOUSE_INSTALL_STRICT=1`, чтобы `npm install` завершалась с ошибкой, если установка инструментов не удалась.

| Метод | Команда |
|--------|---------|
| npm (основной) | `npm i -g @nicholasxdavis/playhouse-cli` |
| npx | `npx @nicholasxdavis/playhouse-cli init` |
| GitHub Release | [Последние бинарники](https://github.com/nicholasxdavis/playhouse-cli/releases/latest) |
| cargo (разработчики) | `cargo install --path . --force` |
| вручную скачанный бинарник | установите `PLAYHOUSE_BIN` или `PLAYHOUSE_SKIP_DOWNLOAD=1` |

Пакет npm загружает нативный бинарный файл (~12 МБ) из релизов GitHub во время выполнения `postinstall`. Текущий релиз: **v0.1.2**.

**Из исходного кода:**

```bash
git clone https://github.com/nicholasxdavis/playhouse-cli.git
cd playhouse-cli
cargo build --release
cargo install --path . --force
```

Локальный цикл разработки npm:

```bash
cargo build --release
cd packages/playhouse && npm run link-local   # bash
# В PowerShell 5.x: cd packages/playhouse; npm run link-local
```

**CI:** [GitHub Actions](https://github.com/nicholasxdavis/playhouse-cli/actions) запускает `cargo test`, дымовые тесты npm и `playhouse verify` на Ubuntu, macOS и Windows.

**Релизы:** опубликуйте тег `v0.1.2` (должен совпадать с `Cargo.toml` и `packages/playhouse/package.json`), чтобы выложить бинарные файлы для автоустановки npm postinstall.

## Быстрый старт

```bash
playhouse                    # TUI (для людей)
playhouse doctor             # проверка инструментов
playhouse verify --json      # полный набор QA + оценка от 0 до 100 звезд
playhouse agent --json       # манифест агента (прочитать первым)
playhouse upgrade --json     # проверка обновлений на GitHub и npm
```

**Для аудита в браузере (Lighthouse, Arkenar) требуется URL-адрес:**

```bash
playhouse config set default_url http://localhost:3000
```

Playhouse также считывает подсказки о портах из `package.json`, Vite и Wrangler config, а затем проверяет стандартные порты разработки.

**Монорепозитории:**

```bash
playhouse config set scan_root apps/web
playhouse config set test_root apps/web
```

## Для агентов

Сначала прочитайте **[AGENTS.md](../AGENTS.md)**. Команда `playhouse init` устанавливает **`.playhouse/SKILL.md`** и другие файлы рабочего пространства.

```bash
playhouse skill install
playhouse skill status --json
```

## Команды

| Команда | Описание |
|---------|----------|
| `playhouse agent [--json]` | Полный манифест агента |
| `playhouse agent status` | Быстрая проверка состояния + следующие действия |
| `playhouse agent plan` | Поэтапный рабочий процесс |
| `playhouse agent handoff` | Проверка + экспорт пакета передачи |
| `playhouse verify` | Trivy + функциональные тесты + Arkenar + Lighthouse |
| `playhouse test list\|init\|run` | Шаблоны тестов |
| `playhouse score` | Оценки Playhouse (0-100 звезд) |
| `playhouse doctor` | Состояние инструментов |
| `playhouse install` | Автоматическая установка встроенных инструментов |
| `playhouse config` | Настройки (get/set/schema) |
| `playhouse skill` | Управление `.playhouse/SKILL.md` |
| `playhouse upgrade` | Проверка наличия новых релизов на GitHub / npm |

См. также: [stars.md](../stars.md), [playwright.md](../playwright.md), [lighthouse.md](../lighthouse.md), [trivy.md](../trivy.md), [arkenar.md](../arkenar.md), [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md).

## Оценки Playhouse

Комбинированная оценка аудита от 0 до 100 после проверки. Отчет в `.playhouse/reports/score.json`. Порог прохождения по умолчанию: 75.

## Структура проекта

```
ваш-проект/
  .playhouse/
    SKILL.md
    BRIEF.md
    AGENT.json
    config.json
    reports/score.json
```

## Пакетные менеджеры

Playwright и Lighthouse используют npm, pnpm, yarn или bun:

```bash
playhouse config set package_manager pnpm
```

## Коды завершения

| Код | Значение |
|------|---------|
| 0 | Успешно |
| 1 | Ошибка |
| 2 | Оценка Lighthouse ниже порога |
| 3 | Обнаружены уязвимости Arkenar |
| 4 | Обнаружены уязвимости Trivy |
| 5 | Отсутствует необходимый инструмент |
