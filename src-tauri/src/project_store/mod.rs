use anyhow::Result;
use std::path::PathBuf;
use crate::domain::project::Project;

pub fn app_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("PodcastConsole")
}

pub fn default_project_path(name: &str) -> PathBuf {
    app_data_dir()
        .join("projects")
        .join(name)
        .join("project.json")
}

pub fn save_project(project: &Project, path: &PathBuf) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(project)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn load_project(path: &PathBuf) -> Result<Project> {
    let json = std::fs::read_to_string(path)?;
    let project: Project = serde_json::from_str(&json)?;
    Ok(project)
}

pub fn list_projects() -> Vec<String> {
    let dir = app_data_dir().join("projects");
    std::fs::read_dir(&dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{load_project, save_project};
    use crate::domain::project::Project;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_project_path() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir()
            .join(format!("podcast-console-test-{}", unique))
            .join("project.json")
    }

    #[test]
    fn save_and_load_preserves_selected_devices() {
        let path = temp_project_path();
        let mut project = Project::new("device-persistence");
        project.input_device_name = Some("Mic USB".to_string());
        project.output_device_name = Some("Monitors USB".to_string());

        save_project(&project, &path).expect("project saved");
        let loaded = load_project(&path).expect("project loaded");

        assert_eq!(loaded.input_device_name.as_deref(), Some("Mic USB"));
        assert_eq!(loaded.output_device_name.as_deref(), Some("Monitors USB"));

        let _ = std::fs::remove_file(&path);
        if let Some(parent) = path.parent() {
            let _ = std::fs::remove_dir_all(parent);
        }
    }
}
