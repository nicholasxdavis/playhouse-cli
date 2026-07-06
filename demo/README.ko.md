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
   <a href="README.ru.md">Русский</a> •
   <a href="README.zh.md">中文</a> •
   <a href="README.ja.md">日本語</a> •
   <b>한국어</b>
</p>
<p align="center">
   <img src="https://img.shields.io/github/actions/workflow/status/nicholasxdavis/playhouse-cli/ci.yml?branch=main&style=for-the-badge&label=CI&color=2bba68" alt="CI 상태">
   <img src="https://img.shields.io/npm/v/@nicholasxdavis/playhouse-cli?style=for-the-badge&label=Version&color=14949c" alt="npm 버전">
   <img src="https://img.shields.io/github/v/release/nicholasxdavis/playhouse-cli?style=for-the-badge&label=Release&color=14949c" alt="GitHub 릴리즈">
   <img src="https://img.shields.io/badge/License-MIT-14949c.svg?style=for-the-badge" alt="MIT 라이선스">
</p>

<p align="center">
  보안, 기능 테스트, 성능 감사 및 에이전트 인계(agent handoff)를 위한 QA CLI 도구.<br>
  쉘, CI 또는 에이전트 도구에서 헤드리스(headless)로 실행됩니다. 사람을 위한 옵션 TUI 제공.
</p>

<p align="center">
  <img src="icon/tui-7-6.png?raw=true" alt="Playhouse CLI 데모" width="75%">
</p>

## 설치

**권장 사항:**

```bash
npm install -g @nicholasxdavis/playhouse-cli
playhouse install --full   # Trivy, Arkenar, Playwright, Lighthouse, chromium
playhouse init             # .playhouse/ 및 에이전트 스킬 설정
```

개발 의존성:

```bash
pnpm add -D @nicholasxdavis/playhouse-cli
npx playhouse doctor
```

| 프로필 | 명령어 | 설치 항목 |
|--------|---------|----------|
| 전체 (기본값) | `playhouse install` 또는 `--full` | Trivy, Arkenar, Playwright, Lighthouse, chromium |
| 최소 | `playhouse install --minimal` | Trivy + Arkenar만 설치 |

옵션 프로젝트 훅:

```json
"scripts": {
  "postinstall": "playhouse-install-tools"
}
```

도구 설치 실패 시 `npm install` 단계를 실패로 처리하려면 `PLAYHOUSE_INSTALL_STRICT=1`을 설정하십시오.

| 방법 | 명령어 |
|--------|---------|
| npm (주요) | `npm i -g @nicholasxdavis/playhouse-cli` |
| npx | `npx @nicholasxdavis/playhouse-cli init` |
| GitHub 릴리즈 | [최신 바이너리](https://github.com/nicholasxdavis/playhouse-cli/releases/latest) |
| cargo (개발자용) | `cargo install --path . --force` |
| 수동 바이너리 | `PLAYHOUSE_BIN` 또는 `PLAYHOUSE_SKIP_DOWNLOAD=1` 설정 |

npm 패키지는 `postinstall` 시점에 OS 및 CPU 사양에 맞는 네이티브 바이너리(~12 MB)를 GitHub Releases에서 다운로드합니다. 현재 릴리즈: **v0.1.2**.

**소스 코드로부터 설치:**

```bash
git clone https://github.com/nicholasxdavis/playhouse-cli.git
cd playhouse-cli
cargo build --release
cargo install --path . --force
```

로컬 npm 개발 루프:

```bash
cargo build --release
cd packages/playhouse && npm run link-local   # bash
# PowerShell 5.x의 경우: cd packages/playhouse; npm run link-local
```

**CI:** [GitHub Actions](https://github.com/nicholasxdavis/playhouse-cli/actions)는 Ubuntu, macOS, Windows 환경에서 `cargo test`, npm 스모크 테스트 및 `playhouse verify`를 실행합니다.

**릴리즈 배포:** `Cargo.toml` 및 `packages/playhouse/package.json` 버전과 동일한 태그 `v0.1.2`를 push하여 npm postinstall용 바이너리를 배포합니다.

## 빠른 시작

```bash
playhouse                    # TUI (사람용)
playhouse doctor             # 도구 상태 확인
playhouse verify --json      # 전체 QA 제품군 실행 + 0-100 별점 평가
playhouse agent --json       # 에이전트 매니페스트 (가장 먼저 읽어보기)
playhouse upgrade --json     # GitHub 및 npm의 업데이트 확인
```

**브라우저 감사 (Lighthouse, Arkenar)를 수행하려면 URL이 필요합니다:**

```bash
playhouse config set default_url http://localhost:3000
```

Playhouse는 `package.json`, Vite 및 Wrangler 설정에서 포트 힌트를 읽고 일반적인 개발 포트를 스캔합니다.

**모노레포:**

```bash
playhouse config set scan_root apps/web
playhouse config set test_root apps/web
```

## 에이전트용 정보

먼저 **[AGENTS.md](../AGENTS.md)**를 읽어보십시오. `playhouse init` 명령은 **`.playhouse/SKILL.md`** 및 기타 워크스페이스 파일을 설치합니다.

```bash
playhouse skill install
playhouse skill status --json
```

## 명령어 목록

| 명령어 | 설명 |
|---------|-------------|
| `playhouse agent [--json]` | 전체 에이전트 매니페스트 |
| `playhouse agent status` | 빠른 상태 확인 및 추천 다음 조치 |
| `playhouse agent plan` | 단계별 워크플로우 제안 |
| `playhouse agent handoff` | 검증 완료 후 인계 번들 내보내기 |
| `playhouse verify` | Trivy + 기능 테스트 + Arkenar + Lighthouse |
| `playhouse test list\|init\|run` | 테스트 베이스플레이트 관리 |
| `playhouse score` | Playhouse 별점 (0-100) |
| `playhouse doctor` | 도구 상태 확인 |
| `playhouse install` | 기본 제공 도구 자동 설치 |
| `playhouse config` | 설정 조회/설정/스키마 (get/set/schema) |
| `playhouse skill` | `.playhouse/SKILL.md` 관리 |
| `playhouse upgrade` | GitHub 및 npm에서 최신 릴리즈 확인 |

참고 문서: [stars.md](../stars.md), [playwright.md](../playwright.md), [lighthouse.md](../lighthouse.md), [trivy.md](../trivy.md), [arkenar.md](../arkenar.md), [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md).

## Playhouse 별점

검증 완료 후 0-100 사이의 종합 감사 점수입니다. 보고서는 `.playhouse/reports/score.json`에 저장됩니다. 기본 통과 기준 점수: 75.

## 프로젝트 구성

```
대상 프로젝트/
  .playhouse/
    SKILL.md
    BRIEF.md
    AGENT.json
    config.json
    reports/score.json
```

## 패키지 매니저

Playwright와 Lighthouse는 npm, pnpm, yarn 또는 bun을 사용합니다:

```bash
playhouse config set package_manager pnpm
```

## 종료 코드

| 코드 | 의미 |
|------|---------|
| 0 | 통과 |
| 1 | 실패 |
| 2 | Lighthouse 점수가 기준치 미만 |
| 3 | Arkenar 진단 항목 발견 |
| 4 | Trivy 진단 항목 발견 |
| 5 | 필요한 도구 누락 |
