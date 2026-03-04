use engine::project::Project;
use tauri::command;

#[command]
fn load_project(path: String) -> Result<serde_json::Value, String> {
    let project = Project::load(&path).map_err(|e| e.to_string())?;
    serde_json::to_value(&project).map_err(|e| e.to_string())
}

#[command]
fn get_project_path() -> String {
    // Returns the default project path for now
    // Later this will open a file dialog
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
        .invoke_handler(tauri::generate_handler![load_project, get_project_path])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}