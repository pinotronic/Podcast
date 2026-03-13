mod domain;
mod audio_engine;
mod app_core;
mod project_store;
mod commands;

use app_core::AppState;
use commands::{pads, recording, devices, projects, mixer as mixer_cmd};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "podcast_console_lib=debug".parse().unwrap()),
        )
        .init();

    let state = AppState::new().expect("Failed to initialize audio engine");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            // Pads
            pads::import_asset,
            pads::assign_asset_to_pad,
            pads::trigger_pad,
            pads::stop_pad,
            pads::stop_all_pads,
            pads::update_pad_name,
            pads::update_pad_color,
            pads::update_pad_gain,
            pads::set_pad_playback_mode,
            pads::switch_bank,
            // Recording
            recording::start_recording,
            recording::stop_recording,
            recording::get_recording_duration,
            recording::get_recording_state,
            // Devices
            devices::list_output_devices,
            devices::list_input_devices,
            devices::get_selected_devices,
            devices::set_selected_devices,
            // Projects
            projects::get_project,
            projects::save_project,
            projects::create_project,
            projects::list_projects,
            projects::open_project,
            projects::set_active_bank,
            // Mixer
            mixer_cmd::set_master_gain,
            mixer_cmd::set_pads_gain,
            mixer_cmd::set_mic_gain,
            mixer_cmd::get_mixer_state,
            mixer_cmd::get_levels,
            mixer_cmd::toggle_mute_strip,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
