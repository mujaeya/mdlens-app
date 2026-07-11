#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};

struct LaunchPath(Mutex<Option<String>>);
struct ClipWatch(Arc<AtomicBool>);

/// 클립보드 자동 기록 on/off (기본 꺼짐 — 개인정보 보호)
#[tauri::command]
fn set_clip_watch(on: bool, state: State<ClipWatch>) {
    state.0.store(on, Ordering::Relaxed);
}

#[derive(serde::Serialize)]
struct LaunchFile {
    name: String,
    content: String,
    path: String,
}

/// 더블클릭/연결 프로그램으로 실행됐을 때 넘어온 파일을 1회 반환
#[tauri::command]
fn launch_file(state: State<LaunchPath>) -> Option<LaunchFile> {
    let path = state.0.lock().ok()?.take()?;
    let content = std::fs::read_to_string(&path).ok()?;
    let name = std::path::Path::new(&path)
        .file_name()?
        .to_string_lossy()
        .to_string();
    Some(LaunchFile { name, content, path })
}

/// 더블클릭으로 연 파일의 원본 경로 저장 (브라우저 FS 핸들 없이도 Ctrl+S 지원)
#[tauri::command]
fn save_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
struct DirEntry {
    name: String,
    path: String,
    is_dir: bool,
}

/// 내장 탐색기: 사용 가능한 드라이브 문자 목록 (C:\, F:\ ...)
#[tauri::command]
fn list_drives() -> Vec<String> {
    let mut out = vec![];
    for c in b'A'..=b'Z' {
        let d = format!("{}:\\", c as char);
        if std::path::Path::new(&d).exists() {
            out.push(d);
        }
    }
    out
}

/// 문서 + 코드/설정 텍스트 파일 판정. (dotfile .env/.gitignore 등 + 확장자 없는 흔한 텍스트 포함)
fn is_openable(name: &str) -> bool {
    let l = name.to_lowercase();
    const EXTS: &[&str] = &[
        ".md", ".markdown", ".txt", ".pdf", ".docx", ".doc", ".rtf",
        ".json", ".jsonc", ".yml", ".yaml", ".toml", ".ini", ".conf", ".config", ".env",
        ".xml", ".csv", ".tsv", ".log", ".properties", ".gradle",
        ".js", ".mjs", ".cjs", ".ts", ".tsx", ".jsx", ".vue", ".svelte",
        ".py", ".rs", ".go", ".rb", ".php", ".java", ".c", ".h", ".cpp", ".hpp", ".cc",
        ".cs", ".swift", ".kt", ".kts", ".lua", ".pl", ".r", ".dart", ".scala",
        ".html", ".htm", ".css", ".scss", ".sass", ".less",
        ".sh", ".bash", ".zsh", ".ps1", ".bat", ".cmd", ".sql",
        ".gitignore", ".gitattributes", ".dockerignore", ".editorconfig", ".npmrc",
        ".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg", ".bmp", ".ico",
    ];
    if EXTS.iter().any(|e| l.ends_with(e)) {
        return true;
    }
    const NOEXT: &[&str] = &[
        "dockerfile", "makefile", "license", "readme", "changelog", "notice",
        ".env", ".gitignore", ".prettierrc", ".eslintrc", ".babelrc",
    ];
    NOEXT.contains(&l.as_str())
}

/// 내장 탐색기: 폴더 하위의 하위폴더 + 열 수 있는 문서/코드 파일을 반환. 폴더 먼저, 이름순.
#[tauri::command]
fn read_dir(path: String) -> Result<Vec<DirEntry>, String> {
    let rd = std::fs::read_dir(&path).map_err(|e| e.to_string())?;
    let mut dirs = vec![];
    let mut files = vec![];
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('$') {
            continue;
        }
        let p = entry.path();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_dir {
            // 숨김/시스템 폴더는 감춤 (단 파일 dotfile은 아래에서 허용)
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
            dirs.push(DirEntry { name, path: p.to_string_lossy().to_string(), is_dir: true });
        } else if is_openable(&name) {
            files.push(DirEntry { name, path: p.to_string_lossy().to_string(), is_dir: false });
        }
    }
    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    dirs.extend(files);
    Ok(dirs)
}

/// 온라인 빠른 번역 (무료 엔드포인트, 정확도보다 가벼움 우선). 네이티브만 — 브라우저는 CORS로 불가.
#[tauri::command]
fn translate_text(text: String, target: String) -> Result<String, String> {
    let url = format!(
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl={}&dt=t&q={}",
        target,
        urlencoding::encode(&text)
    );
    let body = ureq::get(&url)
        .call()
        .map_err(|e| e.to_string())?
        .into_string()
        .map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    let mut out = String::new();
    if let Some(arr) = v.get(0).and_then(|x| x.as_array()) {
        for seg in arr {
            if let Some(s) = seg.get(0).and_then(|x| x.as_str()) {
                out.push_str(s);
            }
        }
    }
    if out.is_empty() {
        Err("no translation".into())
    } else {
        Ok(out)
    }
}

/// 내장 탐색기: 경로로 텍스트 파일(md/txt) 열기
#[tauri::command]
fn read_text_at(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

/// 내장 탐색기: 경로로 바이너리(pdf/docx)를 base64로 반환
#[tauri::command]
fn read_bytes_at(path: String) -> Result<String, String> {
    use std::io::Read;
    let mut f = std::fs::File::open(&path).map_err(|e| e.to_string())?;
    let mut buf = vec![];
    f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    Ok(B64.encode(&buf))
}

fn main() {
    let arg = std::env::args()
        .nth(1)
        .filter(|a| std::path::Path::new(a).is_file());

    let watch = Arc::new(AtomicBool::new(false));
    let watch_for_thread = watch.clone();

    tauri::Builder::default()
        // 단일 인스턴스: 이미 열려 있으면 두 번째 실행의 파일 인자를 기존 창으로 넘기고 종료 (새 창 금지)
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if let Some(p) = argv
                .get(1)
                .filter(|a| std::path::Path::new(a.as_str()).is_file())
            {
                let _ = app.emit("open-path", p.clone());
            }
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .manage(LaunchPath(Mutex::new(arg)))
        .manage(ClipWatch(watch))
        .setup(move |app| {
            // 클립보드 폴링 스레드: 켜져 있을 때만 새 텍스트를 프론트로 emit
            let handle = app.handle().clone();
            let w = watch_for_thread;
            std::thread::spawn(move || {
                let mut cb = match arboard::Clipboard::new() {
                    Ok(c) => c,
                    Err(_) => return,
                };
                let mut last = String::new();
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(1200));
                    if !w.load(Ordering::Relaxed) {
                        continue;
                    }
                    if let Ok(txt) = cb.get_text() {
                        if !txt.is_empty() && txt != last {
                            last = txt.clone();
                            let _ = handle.emit("clip-added", txt);
                        }
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            launch_file,
            save_file,
            list_drives,
            read_dir,
            read_text_at,
            read_bytes_at,
            set_clip_watch,
            translate_text
        ])
        .run(tauri::generate_context!())
        .expect("error while running MD Lens");
}
