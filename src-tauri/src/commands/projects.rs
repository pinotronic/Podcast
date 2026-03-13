use tauri::State;
use crate::app_core::AppState;
use crate::domain::project::Project;
use crate::project_store;

#[tauri::command]
pub async fn get_project(state: State<'_, AppState>) -> Result<Project, String> {
    let project = state.project.lock();
    Ok(project.clone())
}

#[tauri::command]
pub async fn save_project(state: State<'_, AppState>) -> Result<(), String> {
    let project = state.project.lock();
    let path = project_store::default_project_path(&project.name);
    project_store::save_project(&project, &path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_project(name: String, state: State<'_, AppState>) -> Result<Project, String> {
    let new_project = Project::new(&name);
    let path = project_store::default_project_path(&name);
    project_store::save_project(&new_project, &path).map_err(|e| e.to_string())?;
    state.activate_project(new_project).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_projects(_state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(project_store::list_projects())
}

#[tauri::command]
pub async fn open_project(name: String, state: State<'_, AppState>) -> Result<Project, String> {
    let path = project_store::default_project_path(&name);
    let loaded = project_store::load_project(&path).map_err(|e| e.to_string())?;
    state.activate_project(loaded).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_active_bank(index: usize, state: State<'_, AppState>) -> Result<(), String> {
    let mut project = state.project.lock();
    if index >= project.banks.len() {
        return Err("Invalid bank".to_string());
    }
    project.active_bank = index;
    Ok(())
}
