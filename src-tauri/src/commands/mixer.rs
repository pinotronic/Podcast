use tauri::State;
use crate::app_core::AppState;
use crate::audio_engine::engine::EngineCommand;

#[tauri::command]
pub async fn set_master_gain(gain: f32, state: State<'_, AppState>) -> Result<(), String> {
    let g = gain.clamp(0.0, 2.0);
    let engine = state.engine.lock();
    engine.send(EngineCommand::SetMasterGain(g));
    let mut project = state.project.lock();
    project.mixer.master_gain = g;
    Ok(())
}

#[tauri::command]
pub async fn set_pads_gain(gain: f32, state: State<'_, AppState>) -> Result<(), String> {
    let g = gain.clamp(0.0, 2.0);
    let engine = state.engine.lock();
    engine.send(EngineCommand::SetPadsGain(g));
    let mut project = state.project.lock();
    project.mixer.pads_strip.gain = g;
    Ok(())
}

#[tauri::command]
pub async fn set_mic_gain(gain: f32, state: State<'_, AppState>) -> Result<(), String> {
    let g = gain.clamp(0.0, 2.0);
    let engine = state.engine.lock();
    engine.send(EngineCommand::SetMicGain(g));
    let mut project = state.project.lock();
    project.mixer.mic_strip.gain = g;
    Ok(())
}

#[tauri::command]
pub async fn get_mixer_state(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let project = state.project.lock();
    Ok(serde_json::to_value(&project.mixer).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn get_levels(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let engine = state.engine.lock();
    let level_rx = engine.level_rx.lock();
    // Drain to get latest
    let mut latest = serde_json::json!({ "master_l": 0.0, "master_r": 0.0, "mic": 0.0 });
    while let Ok(snap) = level_rx.try_recv() {
        latest = serde_json::json!({
            "master_l": snap.master_l,
            "master_r": snap.master_r,
            "mic": snap.mic,
        });
    }
    Ok(latest)
}

#[tauri::command]
pub async fn toggle_mute_strip(
    strip: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let mut project = state.project.lock();
    let engine = state.engine.lock();
    let muted = match strip.as_str() {
        "mic" => {
            project.mixer.mic_strip.is_muted = !project.mixer.mic_strip.is_muted;
            engine.send(EngineCommand::SetMicMuted(project.mixer.mic_strip.is_muted));
            project.mixer.mic_strip.is_muted
        }
        "pads" => {
            project.mixer.pads_strip.is_muted = !project.mixer.pads_strip.is_muted;
            engine.send(EngineCommand::SetPadsMuted(project.mixer.pads_strip.is_muted));
            project.mixer.pads_strip.is_muted
        }
        "master" => {
            project.mixer.master_muted = !project.mixer.master_muted;
            engine.send(EngineCommand::SetMasterMuted(project.mixer.master_muted));
            if project.mixer.master_muted {
                project.mixer.master_peak_l = 0.0;
                project.mixer.master_peak_r = 0.0;
            }
            project.mixer.master_muted
        }
        _ => return Err("Unknown strip".to_string()),
    };
    Ok(muted)
}
