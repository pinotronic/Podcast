import { useState, useEffect } from 'react';
import { useAppStore } from '../store';
import { commands, pickAudioFile } from '../lib/tauri';

const COLORS = [
  '#4a9eff', '#ff6b6b', '#51cf66', '#ffd43b',
  '#cc5de8', '#ff922b', '#20c997', '#f06595',
  '#74c0fc', '#a9e34b', '#e599f7', '#ffa94d',
];

export function PadEditor() {
  const { project, selectedPad, loadProject, setSelectedPad } = useAppStore();
  const [name, setName] = useState('');
  const [color, setColor] = useState('#4a9eff');
  const [gain, setGain] = useState(1.0);
  const [mode, setMode] = useState('OneShot');

  const pad = selectedPad && project
    ? project.banks[selectedPad.bank]?.pads[selectedPad.slot]
    : null;

  useEffect(() => {
    if (pad) {
      setName(pad.name);
      setColor(pad.color);
      setGain(pad.gain);
      setMode(pad.playback_mode);
    }
  }, [pad]);

  if (!pad || !selectedPad) {
    return (
      <div className="pad-editor empty">
        <p>Right-click a pad to edit it</p>
      </div>
    );
  }

  const save = async () => {
    await commands.updatePadName(selectedPad.bank, selectedPad.slot, name);
    await commands.updatePadColor(selectedPad.bank, selectedPad.slot, color);
    await commands.updatePadGain(selectedPad.bank, selectedPad.slot, gain);
    await commands.setPadPlaybackMode(selectedPad.bank, selectedPad.slot, mode.toLowerCase());
    await loadProject();
    setSelectedPad(null, null);
  };

  const assignAudio = async () => {
    const path = await pickAudioFile();
    if (!path) return;
    const result = await commands.importAsset(path);
    await commands.assignAssetToPad(selectedPad.bank, selectedPad.slot, result.asset_id);
    setName(result.name);
    await loadProject();
  };

  return (
    <div className="pad-editor">
      <div className="editor-header">
        <h3>Edit Pad</h3>
        <button className="btn-close" onClick={() => setSelectedPad(null, null)}>✕</button>
      </div>

      <div className="editor-field">
        <label>Name</label>
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          className="editor-input"
        />
      </div>

      <div className="editor-field">
        <label>Color</label>
        <div className="color-picker">
          {COLORS.map((c) => (
            <button
              key={c}
              className={`color-swatch ${color === c ? 'selected' : ''}`}
              style={{ background: c }}
              onClick={() => setColor(c)}
            />
          ))}
        </div>
      </div>

      <div className="editor-field">
        <label>Mode</label>
        <select
          value={mode}
          onChange={(e) => setMode(e.target.value)}
          className="editor-select"
        >
          <option value="OneShot">One Shot</option>
          <option value="Loop">Loop</option>
          <option value="Toggle">Toggle</option>
        </select>
      </div>

      <div className="editor-field">
        <label>Gain: {Math.round(gain * 100)}%</label>
        <input
          type="range"
          min={0}
          max={2}
          step={0.01}
          value={gain}
          onChange={(e) => setGain(parseFloat(e.target.value))}
          className="editor-slider"
        />
      </div>

      {pad.asset && (
        <div className="editor-asset-info">
          <span className="asset-icon">🎵</span>
          <div>
            <div className="asset-name">{pad.asset.name}</div>
            <div className="asset-meta">{pad.asset.duration_secs.toFixed(1)}s · {pad.asset.sample_rate}Hz</div>
          </div>
        </div>
      )}

      <div className="editor-actions">
        <button className="btn-assign" onClick={assignAudio}>
          {pad.asset ? '↻ Replace Audio' : '+ Assign Audio'}
        </button>
        <button className="btn-save" onClick={save}>Save</button>
      </div>
    </div>
  );
}
