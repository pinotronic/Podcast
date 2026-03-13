use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use std::sync::Arc;

use crate::audio_engine::decoder;
use crate::audio_engine::engine::EngineCommand;
use crate::audio_engine::scheduler::PadVoice;
use crate::app_core::AppState;
use crate::domain::pad::{AudioAsset, AssetId, PlaybackMode};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportAssetResult {
    pub asset_id: String,
    pub name: String,
    pub duration_secs: f64,
    pub sample_rate: u32,
    pub channels: u16,
}

#[tauri::command]
pub async fn import_asset(
    path: String,
    state: State<'_, AppState>,
) -> Result<ImportAssetResult, String> {
    let decoded = decoder::decode_file(&path).map_err(|e| e.to_string())?;

    let name = std::path::Path::new(&path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("asset")
        .to_string();

    let asset = AudioAsset {
        id: AssetId::new(),
        name: name.clone(),
        path: path.clone(),
        duration_secs: decoded.duration_secs,
        sample_rate: decoded.sample_rate,
        channels: decoded.channels,
    };

    let asset_id = asset.id.0.clone();
    let result = ImportAssetResult {
        asset_id: asset_id.clone(),
        name,
        duration_secs: decoded.duration_secs,
        sample_rate: decoded.sample_rate,
        channels: decoded.channels,
    };

    {
        let mut lib = state.asset_library.lock();
        lib.insert(asset_id, (asset, Arc::new(decoded.samples)));
    }

    Ok(result)
}

#[tauri::command]
pub async fn assign_asset_to_pad(
    bank_index: usize,
    slot_index: usize,
    asset_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut project = state.project.lock();
    let bank = project.banks.get_mut(bank_index).ok_or("Invalid bank")?;
    let pad = bank.pads.get_mut(slot_index).ok_or("Invalid slot")?;

    let lib = state.asset_library.lock();
    let (asset, _) = lib.get(&asset_id).ok_or("Asset not found")?;
    pad.asset = Some(asset.clone());
    Ok(())
}

#[tauri::command]
pub async fn trigger_pad(
    bank_index: usize,
    slot_index: usize,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let pad_id;
    let asset_id;
    let channels;
    let gain;
    let looping;

    {
        let project = state.project.lock();
        let bank = project.banks.get(bank_index).ok_or("Invalid bank")?;
        let pad = bank.pads.get(slot_index).ok_or("Invalid slot")?;
        if !pad.is_ready() {
            return Err("Pad not ready".to_string());
        }
        pad_id = pad.id.0.clone();
        gain = pad.gain;
        looping = pad.playback_mode == PlaybackMode::Loop;
        let asset = pad.asset.as_ref().unwrap();
        asset_id = asset.id.0.clone();
        channels = asset.channels;
    }

    let samples = {
        let lib = state.asset_library.lock();
        let (asset, samples) = lib.get(&asset_id).ok_or("Asset not loaded")?;
        let _ = asset;
        samples.clone()
    };

    let playback_id = Uuid::new_v4().to_string();
    let voice = PadVoice {
        pad_id: pad_id.clone(),
        playback_id: playback_id.clone(),
        samples,
        channels,
        gain,
        cursor: 0,
        looping,
        finished: false,
    };

    let engine = state.engine.lock();
    engine.send(EngineCommand::TriggerPad(voice));
    Ok(playback_id)
}

#[tauri::command]
pub async fn stop_pad(
    bank_index: usize,
    slot_index: usize,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pad_id = {
        let project = state.project.lock();
        let bank = project.banks.get(bank_index).ok_or("Invalid bank")?;
        let pad = bank.pads.get(slot_index).ok_or("Invalid slot")?;
        pad.id.0.clone()
    };
    let engine = state.engine.lock();
    engine.send(EngineCommand::StopPad(pad_id));
    Ok(())
}

#[tauri::command]
pub async fn stop_all_pads(state: State<'_, AppState>) -> Result<(), String> {
    let engine = state.engine.lock();
    engine.send(EngineCommand::StopAll);
    Ok(())
}

#[tauri::command]
pub async fn update_pad_name(
    bank_index: usize,
    slot_index: usize,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut project = state.project.lock();
    let bank = project.banks.get_mut(bank_index).ok_or("Invalid bank")?;
    let pad = bank.pads.get_mut(slot_index).ok_or("Invalid slot")?;
    pad.name = name;
    Ok(())
}

#[tauri::command]
pub async fn update_pad_color(
    bank_index: usize,
    slot_index: usize,
    color: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut project = state.project.lock();
    let bank = project.banks.get_mut(bank_index).ok_or("Invalid bank")?;
    let pad = bank.pads.get_mut(slot_index).ok_or("Invalid slot")?;
    pad.color = color;
    Ok(())
}

#[tauri::command]
pub async fn update_pad_gain(
    bank_index: usize,
    slot_index: usize,
    gain: f32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut project = state.project.lock();
    let bank = project.banks.get_mut(bank_index).ok_or("Invalid bank")?;
    let pad = bank.pads.get_mut(slot_index).ok_or("Invalid slot")?;
    pad.gain = gain.clamp(0.0, 2.0);
    Ok(())
}

#[tauri::command]
pub async fn set_pad_playback_mode(
    bank_index: usize,
    slot_index: usize,
    mode: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut project = state.project.lock();
    let bank = project.banks.get_mut(bank_index).ok_or("Invalid bank")?;
    let pad = bank.pads.get_mut(slot_index).ok_or("Invalid slot")?;
    pad.playback_mode = match mode.as_str() {
        "loop" => PlaybackMode::Loop,
        "toggle" => PlaybackMode::Toggle,
        _ => PlaybackMode::OneShot,
    };
    Ok(())
}

#[tauri::command]
pub async fn switch_bank(
    bank_index: usize,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut project = state.project.lock();
    if bank_index >= project.banks.len() {
        return Err("Invalid bank index".to_string());
    }
    project.active_bank = bank_index;
    Ok(())
}
