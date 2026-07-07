use civil::parser::parse_dxf_file;
use engine::bcf::export_clashes_to_bcf;
use engine::clash::ClashDetector;
use engine::object::ConstructionObject;
use engine::project::Project;
use ifc::parser::parse_ifc_file;
use tauri::command;
use std::path::{Component, Path};
use std::sync::{Mutex, MutexGuard, PoisonError};

pub struct AppState {
    pub project: Mutex<Option<Project>>,
}

/// A panic elsewhere while holding this lock must not brick every future
/// command — the guarded state itself (Option<Project>) is never left
/// invalid mid-mutation, so recovering the poisoned value is safe.
fn lock_project(state: &AppState) -> MutexGuard<'_, Option<Project>> {
    state.project.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Wraps a freshly imported (IFC/DXF) object list in a new Project named
/// after the source file, since neither format carries an OCM project id.
fn wrap_imported_objects(path: &str, objects: Vec<ConstructionObject>) -> Project {
    let mut project = Project::new(
        Path::new(path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    );
    for obj in objects {
        project.add_object(obj);
    }
    project
}

#[command]
fn load_project(path: String, state: tauri::State<AppState>) -> Result<serde_json::Value, String> {
    let project = if path.ends_with(".ifc") {
        let objects = parse_ifc_file(&path).map_err(|e| e.to_string())?;
        wrap_imported_objects(&path, objects)
    } else if path.ends_with(".dxf") {
        let objects = parse_dxf_file(&path).map_err(|e| e.to_string())?;
        wrap_imported_objects(&path, objects)
    } else {
        // load project from .ocm file
        Project::load(&path).map_err(|e| e.to_string())?
    };

    *lock_project(&state) = Some(project.clone());
    serde_json::to_value(&project).map_err(|e| e.to_string())
}

#[command]
fn run_clash(state: tauri::State<AppState>) -> Result<serde_json::Value, String> {
    let guard = lock_project(&state);
    let project = guard.as_ref().ok_or("No project loaded")?;
    let refs: Vec<&engine::object::ConstructionObject> = project.objects.values().collect();
    let results = ClashDetector::run(&refs);
    serde_json::to_value(&results).map_err(|e| e.to_string())
}

#[command]
fn export_bcf(path: String, state: tauri::State<AppState>) -> Result<(), String> {
    if Path::new(&path).components().any(|c| c == Component::ParentDir) {
        return Err(format!("Invalid output path: {path}"));
    }

    let guard = lock_project(&state);
    let project = guard.as_ref().ok_or("No project loaded")?;
    let refs: Vec<&engine::object::ConstructionObject> = project.objects.values().collect();
    let results = ClashDetector::run(&refs);

    let bytes = export_clashes_to_bcf(project, &results).map_err(|e| e.to_string())?;
    std::fs::write(&path, bytes).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())  
        .manage(AppState {
            project: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![load_project, run_clash, export_bcf])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}