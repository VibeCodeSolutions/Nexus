// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::sync::Mutex;

use tauri::{Manager, RunEvent, WindowEvent};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;

/// Holds the handle to the running `nexus-core` sidecar process.
struct SidecarHandle(Mutex<Option<CommandChild>>);

#[tauri::command]
fn get_core_token() -> Result<String, String> {
    let home = dirs::home_dir().ok_or_else(|| "no home dir".to_string())?;
    let path = home.join(".nexus_token");
    fs::read_to_string(&path)
        .map(|s| s.trim().to_string())
        .map_err(|e| format!("token read failed ({}): {}", path.display(), e))
}

#[tauri::command]
fn get_core_url() -> String {
    "http://127.0.0.1:7777".to_string()
}

#[tauri::command]
fn restart_core(
    app: tauri::AppHandle,
    state: tauri::State<'_, SidecarHandle>,
) -> Result<(), String> {
    if let Some(child) = state.0.lock().unwrap().take() {
        let _ = child.kill();
    }
    spawn_sidecar(&app).map_err(|e| format!("respawn failed: {}", e))
}

/// Spawn the bundled `nexus-core` sidecar and store its handle in the managed state.
fn spawn_sidecar(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let (mut rx, child) = app
        .shell()
        .sidecar("nexus-core")?
        .args(["serve"])
        .spawn()?;

    // Stash the child so we can kill it on shutdown.
    {
        let state = app.state::<SidecarHandle>();
        *state.0.lock().unwrap() = Some(child);
    }

    // Pipe the sidecar's stdout/stderr into our own stderr so `tauri dev`
    // shows core logs inline.
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) | CommandEvent::Stderr(line) => {
                    eprintln!("[nexus-core] {}", String::from_utf8_lossy(&line));
                }
                CommandEvent::Terminated(payload) => {
                    eprintln!("[nexus-core] terminated: {:?}", payload);
                }
                _ => {}
            }
        }
    });

    Ok(())
}

fn kill_sidecar(app: &tauri::AppHandle) {
    let state = app.state::<SidecarHandle>();
    let maybe_child = state.0.lock().unwrap().take();
    if let Some(child) = maybe_child {
        let _ = child.kill();
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(SidecarHandle(Mutex::new(None)))
        .setup(|app| {
            let handle = app.handle().clone();
            if let Err(e) = spawn_sidecar(&handle) {
                eprintln!("Sidecar spawn failed: {}", e);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { .. } = event {
                kill_sidecar(&window.app_handle().clone());
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_core_token,
            get_core_url,
            restart_core
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let RunEvent::ExitRequested { .. } | RunEvent::Exit = event {
                kill_sidecar(app_handle);
            }
        });
}
