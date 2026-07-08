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

/// 내장 탐색기: 폴더 하위의 하위폴더 + 문서(md/txt/pdf/docx)를 반환. 폴더 먼저, 이름순.
#[tauri::command]
fn read_dir(path: String) -> Result<Vec<DirEntry>, String> {
    let rd = std::fs::read_dir(&path).map_err(|e| e.to_string())?;
    let mut dirs = vec![];
    let mut files = vec![];
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name.starts_with('$') {
            continue;
        }
        let p = entry.path();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_dir {
            if name == "node_modules" || name == "target" {
                continue;
            }
            dirs.push(DirEntry { name, path: p.to_string_lossy().to_string(), is_dir: true });
        } else {
            let lower = name.to_lowercase();
            if [".md", ".markdown", ".txt", ".pdf", ".docx", ".doc"]
                .iter()
                .any(|e| lower.ends_with(e))
            {
                files.push(DirEntry { name, path: p.to_string_lossy().to_string(), is_dir: false });
            }
        }
    }
    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    dirs.extend(files);
    Ok(dirs)
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
            set_clip_watch
        ])
        .run(tauri::generate_context!())
        .expect("error while running MD Lens");
}
