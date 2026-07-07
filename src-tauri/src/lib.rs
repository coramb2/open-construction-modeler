use civil::parser::parse_dxf_file;
use engine::bcf::export_clashes_to_bcf;
use engine::clash::{ClashCheckResult, ClashDetector};
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

/// Pure command logic, kept separate from the #[command] wrapper so it can be
/// unit tested against a plain AppState without spinning up a Tauri runtime.
fn load_project_impl(path: &str, state: &AppState) -> Result<Project, String> {
    let project = if path.ends_with(".ifc") {
        let objects = parse_ifc_file(path).map_err(|e| e.to_string())?;
        wrap_imported_objects(path, objects)
    } else if path.ends_with(".dxf") {
        let objects = parse_dxf_file(path).map_err(|e| e.to_string())?;
        wrap_imported_objects(path, objects)
    } else {
        // load project from .ocm file
        Project::load(path).map_err(|e| e.to_string())?
    };

    // Only reached on success — a failed load must not clobber whatever
    // project was already loaded.
    *lock_project(state) = Some(project.clone());
    Ok(project)
}

#[command]
fn load_project(path: String, state: tauri::State<AppState>) -> Result<serde_json::Value, String> {
    let project = load_project_impl(&path, &state)?;
    serde_json::to_value(&project).map_err(|e| e.to_string())
}

fn run_clash_impl(state: &AppState) -> Result<Vec<ClashCheckResult>, String> {
    let guard = lock_project(state);
    let project = guard.as_ref().ok_or("No project loaded")?;
    let refs: Vec<&ConstructionObject> = project.objects.values().collect();
    Ok(ClashDetector::run(&refs))
}

#[command]
fn run_clash(state: tauri::State<AppState>) -> Result<serde_json::Value, String> {
    let results = run_clash_impl(&state)?;
    serde_json::to_value(&results).map_err(|e| e.to_string())
}

fn export_bcf_impl(path: &str, state: &AppState) -> Result<Vec<u8>, String> {
    if Path::new(path).components().any(|c| c == Component::ParentDir) {
        return Err(format!("Invalid output path: {path}"));
    }

    let guard = lock_project(state);
    let project = guard.as_ref().ok_or("No project loaded")?;
    let refs: Vec<&ConstructionObject> = project.objects.values().collect();
    let results = ClashDetector::run(&refs);

    export_clashes_to_bcf(project, &results).map_err(|e| e.to_string())
}

#[command]
fn export_bcf(path: String, state: tauri::State<AppState>) -> Result<(), String> {
    let bytes = export_bcf_impl(&path, &state)?;
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
#[cfg(test)]
mod tests {
    use super::*;
    use engine::clash::ClashCheckResult;
    use engine::metadata::{LodLevel, Trade};
    use std::sync::Arc;

    fn empty_state() -> AppState {
        AppState { project: Mutex::new(None) }
    }

    fn tmp_path(ext: &str) -> String {
        format!("/tmp/ocm_tauri_test_{}.{ext}", uuid::Uuid::new_v4())
    }

    fn make_object(name: &str, pos: [f64; 3], dims: [f64; 3]) -> ConstructionObject {
        let mut obj = ConstructionObject::new(
            name.to_string(),
            Trade::Structural,
            None,
            LodLevel::Lod200,
            String::new(),
            String::new(),
        );
        obj.position = Some(pos);
        obj.dimensions = Some(dims);
        obj
    }

    #[test]
    fn test_load_project_ocm() {
        let path = tmp_path("ocm");
        let mut project = Project::new("Saved Project".to_string());
        project.add_object(make_object("A", [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]));
        project.save(&path).unwrap();

        let state = empty_state();
        let loaded = load_project_impl(&path, &state).expect("load should succeed");
        assert_eq!(loaded.name, "Saved Project");
        assert_eq!(loaded.objects.len(), 1);

        let guard = lock_project(&state);
        assert_eq!(guard.as_ref().unwrap().name, "Saved Project");
    }

    #[test]
    fn test_load_project_ifc() {
        let path = tmp_path("ifc");
        std::fs::write(&path, "#42= IFCWALLSTANDARDCASE('guid123',#1,'Wall A',$,$);").unwrap();

        let state = empty_state();
        let loaded = load_project_impl(&path, &state).expect("load should succeed");
        assert_eq!(loaded.objects.len(), 1);
        assert_eq!(loaded.objects.values().next().unwrap().name, "Wall A");
        // Project name should come from the file, not the full path
        assert!(loaded.name.ends_with(".ifc"));
        assert!(!loaded.name.contains('/'));
    }

    #[test]
    fn test_load_project_dxf() {
        let path = tmp_path("dxf");
        let content = "0\nSECTION\n2\nENTITIES\n0\nPOINT\n8\nC-SURVEY\n10\n1.0\n20\n2.0\n30\n0.0\n0\nENDSEC\n0\nEOF\n";
        std::fs::write(&path, content).unwrap();

        let state = empty_state();
        let loaded = load_project_impl(&path, &state).expect("load should succeed");
        assert_eq!(loaded.objects.len(), 1);
        assert!(matches!(loaded.objects.values().next().unwrap().trade, Trade::Civil));
    }

    #[test]
    fn test_load_project_failure_preserves_existing_state() {
        let state = empty_state();
        load_project_impl("/tmp/ocm_tauri_test_does_not_exist_at_all.ocm", &state)
            .expect_err("nonexistent file should fail");
        // Nothing was ever loaded, so state should remain empty (not panic, not
        // spuriously populated)
        assert!(lock_project(&state).is_none());

        // Now load something real, then try (and fail) to load garbage —
        // the original project must survive the failed attempt.
        let good_path = tmp_path("ocm");
        Project::new("Original".to_string()).save(&good_path).unwrap();
        load_project_impl(&good_path, &state).unwrap();

        load_project_impl("/tmp/ocm_tauri_test_still_does_not_exist.ocm", &state)
            .expect_err("second load should also fail");
        assert_eq!(lock_project(&state).as_ref().unwrap().name, "Original");
    }

    #[test]
    fn test_load_project_rejects_path_traversal() {
        let state = empty_state();
        let result = load_project_impl("../../etc/passwd.ifc", &state);
        assert!(result.is_err());
        assert!(lock_project(&state).is_none());
    }

    #[test]
    fn test_run_clash_no_project_loaded() {
        let state = empty_state();
        let result = run_clash_impl(&state);
        assert_eq!(result.unwrap_err(), "No project loaded");
    }

    #[test]
    fn test_run_clash_empty_project() {
        let state = empty_state();
        *lock_project(&state) = Some(Project::new("Empty".to_string()));
        let results = run_clash_impl(&state).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_run_clash_with_overlapping_objects() {
        let mut project = Project::new("Clashy".to_string());
        project.add_object(make_object("A", [0.0, 0.0, 0.0], [2.0, 2.0, 2.0]));
        project.add_object(make_object("B", [1.0, 1.0, 1.0], [2.0, 2.0, 2.0]));

        let state = empty_state();
        *lock_project(&state) = Some(project);

        let results = run_clash_impl(&state).unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], ClashCheckResult::Clash(_)));
    }

    #[test]
    fn test_export_bcf_rejects_path_traversal() {
        let state = empty_state();
        let result = export_bcf_impl("../../tmp/evil.bcfzip", &state);
        assert!(result.is_err());
    }

    #[test]
    fn test_export_bcf_no_project_loaded() {
        let state = empty_state();
        let result = export_bcf_impl(&tmp_path("bcfzip"), &state);
        assert_eq!(result.unwrap_err(), "No project loaded");
    }

    #[test]
    fn test_export_bcf_produces_valid_zip() {
        let mut project = Project::new("Clashy".to_string());
        project.add_object(make_object("A", [0.0, 0.0, 0.0], [2.0, 2.0, 2.0]));
        project.add_object(make_object("B", [1.0, 1.0, 1.0], [2.0, 2.0, 2.0]));

        let state = empty_state();
        *lock_project(&state) = Some(project);

        let bytes = export_bcf_impl("clashes.bcfzip", &state).unwrap();
        let archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        // bcf.version + one topic for the one clash
        assert_eq!(archive.len(), 2);
    }

    #[test]
    fn test_wrap_imported_objects_names_project_after_filename_only() {
        let objects = vec![make_object("A", [0.0, 0.0, 0.0], [1.0, 1.0, 1.0])];
        let project = wrap_imported_objects("/some/deep/path/MyModel.ifc", objects);
        assert_eq!(project.name, "MyModel.ifc");
        assert_eq!(project.objects.len(), 1);
    }

    #[test]
    fn test_lock_project_recovers_from_poisoned_mutex() {
        let state = Arc::new(empty_state());
        *lock_project(&state) = Some(Project::new("Before poison".to_string()));

        let poison_state = Arc::clone(&state);
        let result = std::thread::spawn(move || {
            let _guard = lock_project(&poison_state);
            panic!("simulated panic while holding the lock");
        })
        .join();
        assert!(result.is_err(), "the spawned thread should have panicked");

        // The mutex is now poisoned. A naive .lock().unwrap() would panic
        // here too — lock_project must recover instead.
        let guard = lock_project(&state);
        assert_eq!(guard.as_ref().unwrap().name, "Before poison");
    }
}
