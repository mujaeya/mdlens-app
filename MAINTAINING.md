# 유지보수 · 배포 메모 (maintainer notes)

> 저장소 소유자(홍찬영)가 직접 손봐야 하는 "수동 조작" 부분 정리.
> 일반 사용자 안내는 `README.md`.
> 소규모 반복 수정은 G-레인(Gemini CLI — 규약 `F:\pharus\GEMINI.md`, 큐 `F:\pharus\g-lane.md`)으로 처리 가능. `[g-lane]` 커밋은 Claude/Codex 게이트 후에만 push.

---

## 1. 설치 파일(.exe)은 저장소에 커밋되지 않는다

- `src-tauri/target/` 은 `.gitignore`로 제외 — 빌드 산출물은 커밋하지 않는 게 정상이다.
- 따라서 배포하려면 **GitHub Releases**에 따로 올려야 한다. 두 가지 방법:

### 방법 A — 자동 빌드 (추천)

태그를 만들어 push하면 `.github/workflows/build.yml`이 GitHub 서버에서 빌드하고 **Release 초안**을 만든다.

- **GitHub Desktop**: `History` 탭 → 최신 커밋 우클릭 → **Create Tag** → `v0.1.0` 입력 → `Repository → Push`(태그도 함께 push)
- **명령**:
  ```bash
  git tag v0.1.0
  git push origin v0.1.0
  ```
- 몇 분 뒤 GitHub 웹 → **Releases**에 `Draft`가 생김 → 내용 확인 후 **Publish release** 클릭
- Windows 설치파일이 자동 첨부된다. (mac/linux는 §4 참고)

### 방법 B — 수동 업로드

- 로컬 빌드 산출물 위치:
  ```
  src-tauri\target\release\bundle\nsis\MD Lens_0.1.0_x64-setup.exe
  ```
- GitHub 웹 → **Releases → Draft a new release** → 태그 입력(`v0.1.0`) → 위 `.exe`를 드래그 → **Publish**

---

## 2. 새 버전 낼 때

버전 숫자를 **3곳** 맞춘 뒤 커밋 → 새 태그 push:

- `src-tauri/tauri.conf.json` → `"version"`
- `src-tauri/Cargo.toml` → `version`
- `package.json` → `"version"`

---

## 3. 번역 API (수동 교체 가능)

- **현재**: 무료 비공식 Google 엔드포인트(키 없음). 가끔 한도에 걸리거나 느리고, 정확도는 "대충 알아보는" 수준.
- **코드 위치**: `src-tauri/src/main.rs` 의 `translate_text` 함수.
- **정식으로 바꾸려면**: DeepL 무료 API(월 50만 자, 키 필요) 또는 Google Cloud Translation로 교체.
  - 키가 필요한 방식으로 바꾸면 **설정 화면에 키 입력란을 추가**해야 한다(현재 없음).
- 번역은 **네이티브 앱 전용**이다(브라우저판은 CORS로 막힘 — 의도된 동작).

---

## 4. 코드 서명 (SmartScreen 경고)

- 현재 **미서명** → 받는 사람에게 "알 수 없는 게시자 / Windows가 PC를 보호했습니다" 경고. 사용자는 **추가 정보 → 실행**으로 넘어가야 한다(README에 안내됨).
- 없애려면 코드 서명 인증서 필요(대략 연 20~40만 원대) → `tauri.conf.json`에 서명 설정 추가.
- 초기 배포에는 미서명으로 두는 경우가 많다.

---

## 5. 크로스 플랫폼 상태

- **Windows 우선** 개발·검증됨.
- mac/linux: 워크플로우가 빌드를 시도하지만 **미검증**. 내장 탐색기의 드라이브 목록(`list_drives`, A:~Z:)이 Windows 전용이라 다른 OS에선 비어 보인다. 필요해지면 OS 분기 추가.
- **Windows만 배포**할 거면 `.github/workflows/build.yml`의 `matrix`에서 `macos-latest`, `ubuntu-latest` 줄을 지우면 된다.

---

## 6. 재빌드 / 프론트 수정

```bash
cd F:\pharus\tools\mdlens-app
npx tauri build          # beforeBuildCommand가 npm run sync 자동 실행
```

- 프론트엔드는 **`mdlens.html` 한 파일**만 고치면 된다. `dist/index.html`은 빌드 때 자동 복사된다.
- 네이티브 전용 기능(탐색기·클립보드·번역·항상 위)은 전부 `window.__TAURI__` 가드 안에 있어, 같은 파일이 브라우저에서도 열린다.
