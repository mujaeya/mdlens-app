# CLAUDE.md — MD Lens

가벼운 로컬 문서 뷰어 (Tauri v2 + 단일 HTML 프론트).

- **소스는 `mdlens.html` 하나** — 네이티브 전용 기능은 전부 `window.__TAURI__` 가드(브라우저에서도 열림). `dist/index.html`은 `npm run sync` 산출물이라 직접 수정 금지.
- 유지보수·릴리즈 절차: `MAINTAINING.md` (버전 3곳 갱신, 태그/수동 릴리즈, 번역 API 교체 위치).
- 현재 플랜·QA 이력: `docs/plan-v0.2.md`.
- 재빌드: `npx tauri build` (beforeBuildCommand가 sync 자동 실행). 폴더 이동/개명 후 빌드 실패 시 `cargo clean`.
- 상위 사업자 규약: `F:\pharus\CLAUDE.md` 상속. 모델 배정 `F:\pharus\model-matrix.md`, G-레인 `F:\pharus\g-lane.md`(T0~T1, `[g-lane]` 커밋은 게이트 후 인정).
- 시각 변경 시 마감 전 **전 영역 스크린샷 훑기**(탭줄·독·스크롤바·설정 포함) — v0.2 QA 재발 방지 규칙.
