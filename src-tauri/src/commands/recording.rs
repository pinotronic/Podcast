use tauri::State;
use crate::app_core::AppState;
use crate::audio_engine::engine::EngineCommand;
use crate::domain::session::RecordingState;
use crate::project_store::app_data_dir;

#[tauri::command]
pub async fn start_recording(state: State<'_, AppState>) -> Result<String, String> {
    {
        let session = state.session.lock();
        if session.recording_state == RecordingState::Recording {
            return Err("Already recording".to_string());
        }
    }

    let project = state.project.lock();
    let project_name = project.name.clone();
    drop(project);

    let engine = state.engine.lock();
    let sample_rate = engine.output_sample_rate();
    let channels = engine.output_channels();
    drop(engine);

    let sessions_dir = app_data_dir()
        .join("projects")
        .join(&project_name)
        .join("sessions");
    std::fs::create_dir_all(&sessions_dir).map_err(|e| e.to_string())?;

    let session_id = {
        let session = state.session.lock();
        session.id.clone()
    };

    let path = sessions_dir
        .join(format!("{}.wav", session_id))
        .to_string_lossy()
        .to_string();

    {
        let mut session = state.session.lock();
        session.recording_state = RecordingState::Recording;
        session.output_path = Some(path.clone());
    }

    let engine = state.engine.lock();
    engine.send(EngineCommand::StartRecording(path.clone(), sample_rate, channels));

    {
        let mut session = state.session.lock();
        session.sample_rate = sample_rate;
        session.duration_frames = 0;
    }

    Ok(path)
}

#[tauri::command]
pub async fn stop_recording(state: State<'_, AppState>) -> Result<Option<String>, String> {
    {
        let mut session = state.session.lock();
        session.recording_state = RecordingState::Idle;
    }

    let engine = state.engine.lock();
    engine.send(EngineCommand::StopRecording);

    let path = {
        let session = state.session.lock();
        session.output_path.clone()
    };

    Ok(path)
}

#[tauri::command]
pub async fn get_recording_duration(state: State<'_, AppState>) -> Result<f64, String> {
    let engine = state.engine.lock();
    Ok(engine.recording_duration_secs())
}

#[tauri::command]
pub async fn get_recording_state(state: State<'_, AppState>) -> Result<String, String> {
    let session = state.session.lock();
    let s = match session.recording_state {
        RecordingState::Idle => "idle",
        RecordingState::Armed => "armed",
        RecordingState::Recording => "recording",
        RecordingState::Paused => "paused",
    };
    Ok(s.to_string())
}
