use engine::clash::ClashDetector;
use engine::project::Project;
use ifc::parser::parse_ifc_file;
use tauri::command;
use std::sync::Mutex;

pub struct AppState {
    pub project: Mutex<Option<Project>>,
}

#[command]
fn load_project(path: String, state: tauri::State<AppState>) -> Result<serde_json::Value, String> {
    if path.ends_with(".ifc") {
        // parse IFC and wrap in a temp project
        let objects = parse_ifc_file(&path).map_err(|e| e.to_string())?;
        let mut project = Project::new(
            std::path::Path::new(&path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        );

        for obj in objects {
            project.add_object(obj);
        }
        *state.project.lock().unwrap() = Some(project.clone());
        serde_json::to_value(&project).map_err(|e| e.to_string())
    } else {
        // load project from .ocm file
        let project = Project::load(&path).map_err(|e| e.to_string())?;
        *state.project.lock().unwrap() = Some(project.clone());
        serde_json::to_value(&project).map_err(|e| e.to_string())
    }

}

#[command]
fn run_clash(state: tauri::State<AppState>) -> Result<serde_json::Value, String> {
    let guard = state.project.lock().unwrap();
    let project = guard.as_ref().ok_or("No project loaded")?;
    let refs: Vec<&engine::object::ConstructionObject> = project.objects.values().collect();
    let results = ClashDetector::run(&refs);
    serde_json::to_value(&results).map_err(|e| e.to_string())
}

#[command]
fn get_project_path() -> String {
    "/home/cora/workspace/opencm/open-construction-modeler/project.ocm".to_string()
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
        .invoke_handler(tauri::generate_handler![load_project, get_project_path, run_clash])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}