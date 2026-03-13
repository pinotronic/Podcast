import { useState, useEffect } from 'react';
import { commands } from '../lib/tauri';
import { useAppStore } from '../store';

interface FaderProps {
  label: string;
  value: number;
  onChange: (v: number) => void;
  isMuted: boolean;
  onToggleMute: () => void;
}

function Fader({ label, value, onChange, isMuted, onToggleMute }: FaderProps) {
  return (
    <div className={`fader-strip ${isMuted ? 'muted' : ''}`}>
      <span className="fader-label">{label}</span>
      <input
        type="range"
        min={0}
        max={2}
        step={0.01}
        value={value}
        onChange={(e) => onChange(parseFloat(e.target.value))}
        className="fader-slider"
        aria-label={`${label} gain`}
      />
      <span className="fader-value">{Math.round(value * 100)}%</span>
      <button
        className={`btn-mute ${isMuted ? 'active' : ''}`}
        onClick={onToggleMute}
      >
        M
      </button>
    </div>
  );
}

export function MixerPanel() {
  const { project, loadProject } = useAppStore();
  const [masterGain, setMasterGainState] = useState(1.0);
  const [padsGain, setPadsGainState] = useState(1.0);
  const [micGain, setMicGainState] = useState(1.0);

  useEffect(() => {
    if (project) {
      setMasterGainState(project.mixer.master_gain);
      setPadsGainState(project.mixer.pads_strip.gain);
      setMicGainState(project.mixer.mic_strip.gain);
    }
  }, [project]);

  const handleMaster = async (v: number) => {
    setMasterGainState(v);
    await commands.setMasterGain(v);
  };

  const handlePads = async (v: number) => {
    setPadsGainState(v);
    await commands.setPadsGain(v);
  };

  const handleMic = async (v: number) => {
    setMicGainState(v);
    await commands.setMicGain(v);
  };

  const handleMute = async (strip: string) => {
    await commands.toggleMuteStrip(strip);
    await loadProject();
  };

  const mixer = project?.mixer;

  return (
    <div className="mixer-panel">
      <h3 className="panel-title">Mixer</h3>
      <div className="fader-row">
        <Fader
          label="MIC"
          value={micGain}
          onChange={handleMic}
          isMuted={mixer?.mic_strip.is_muted ?? false}
          onToggleMute={() => handleMute('mic')}
        />
        <Fader
          label="PADS"
          value={padsGain}
          onChange={handlePads}
          isMuted={mixer?.pads_strip.is_muted ?? false}
          onToggleMute={() => handleMute('pads')}
        />
        <div className="fader-separator" />
        <Fader
          label="MASTER"
          value={masterGain}
          onChange={handleMaster}
          isMuted={mixer?.master_muted ?? false}
          onToggleMute={() => handleMute('master')}
        />
      </div>
    </div>
  );
}
