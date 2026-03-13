use tauri::State;
use crate::app_core::AppState;
use crate::audio_engine::engine::AudioEngine;
use crate::domain::session::RecordingState;

#[tauri::command]
pub async fn list_output_devices(_state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(AudioEngine::list_output_devices())
}

#[tauri::command]
pub async fn list_input_devices(_state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(AudioEngine::list_input_devices())
}

#[tauri::command]
pub async fn get_selected_devices(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let project = state.project.lock();
    Ok(serde_json::json!({
        "input": project.input_device_name,
        "output": project.output_device_name,
    }))
}

#[tauri::command]
pub async fn set_selected_devices(
    input: Option<String>,
    output: Option<String>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    {
        let session = state.session.lock();
        if session.recording_state == RecordingState::Recording {
            return Err("Stop recording before changing devices".to_string());
        }
    }

    state.reload_engine(input.clone(), output.clone()).map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "input": input.filter(|value| !value.is_empty()),
        "output": output.filter(|value| !value.is_empty()),
    }))
}
