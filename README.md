# MD Lens

A lightweight, local‑first document viewer for Markdown, text, PDF and Word files.
No account, no cloud, no telemetry — it just opens your files fast.

> 가벼운 로컬 문서 뷰어입니다. md · txt · pdf · docx를 빠르게 열어보고, 탭·분할·내장 탐색기·클립보드 기록까지. 설치는 아래 **Download**.

---

## Features

- **Formats** — Markdown & text (render + edit), PDF (built‑in viewer), DOCX (rendered)
- **Code & config files** — `.json` · `.yml` · `.toml` · `.env` · `.gitignore` · `.js` · `.py` · `.rs` and many more open as editable plain text — a fast pop‑open for quick edits without launching an IDE
- **Tabs** — each split pane has its own tabs
- **Split view** — 1 / 2 / 3 panes, independent of tabs
- **Built‑in file explorer** *(app)* — browse drives and folders inside the app, no OS dialog
- **Clipboard auto‑record** *(app, off by default)* — capture copied text into a list you can re‑copy
- **Quick translate** *(app, optional/online)* — select text → a small button translates it into your UI language via a free service (local‑first, so it stays a lightweight extra)
- **Always‑on‑top** *(app)* — pin the window like a sticky note
- **File association** *(app)* — double‑click `.md` / `.txt` to open in MD Lens
- **7 languages** — English · 한국어 · 日本語 · 中文 · Français · Deutsch · Español (⚙ top‑right)
- **Light / dark** — follows your system theme

There are two ways to run it:

| | Browser version | App (recommended) |
|---|---|---|
| File | `mdlens.html` | installer / `mdlens.exe` |
| Install | none — open in Chrome/Edge | one‑click installer |
| Explorer · clipboard · always‑on‑top · file association | ✗ (browser sandbox) | ✓ |

## Download

Grab the latest installer from the [Releases](../../releases) page:

- **Windows** — `MD Lens_x.y.z_x64-setup.exe`

> ⚠️ The app is **not code‑signed** yet, so Windows SmartScreen may warn "unknown publisher".
> Click **More info → Run anyway**. (Signing is planned.)

## Keyboard shortcuts

| Action | Keys |
|---|---|
| New document | `Ctrl+N` |
| Open | `Ctrl+O` (or drag & drop onto a pane) |
| Edit ⇄ View | `Ctrl+E` |
| Save | `Ctrl+S` |
| Split | click **1 / 2 / 3** at the top |

## Build from source

Requires [Node.js](https://nodejs.org) and the [Rust toolchain](https://rustup.rs) (+ the
[Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS).

```bash
npm install
npm run tauri build      # or: npx tauri build
```

Output:
- standalone exe → `src-tauri/target/release/mdlens.exe`
- installer → `src-tauri/target/release/bundle/`

**Frontend source is a single file:** `mdlens.html`. `npm run sync` copies it to `dist/index.html`
(what Tauri bundles). All native‑only features are guarded by `window.__TAURI__`, so the same file
runs as the browser version too.

## Tech

Tauri v2 · Rust · a single dependency‑free HTML/JS frontend. Native commands: file open/save,
directory listing, clipboard watch. Word rendering via [mammoth.js](https://github.com/mwilliamson/mammoth.js).

## License

[MIT](LICENSE) © 2026 HonCC-9
