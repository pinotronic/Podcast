import { useState, useEffect } from 'react';
import { commands } from '../lib/tauri';

export function DevicesPanel() {
  const [inputs, setInputs] = useState<string[]>([]);
  const [outputs, setOutputs] = useState<string[]>([]);
  const [selected, setSelected] = useState<{ input: string | null; output: string | null }>({ input: null, output: null });
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isApplying, setIsApplying] = useState(false);

  useEffect(() => {
    commands.listInputDevices().then(setInputs);
    commands.listOutputDevices().then(setOutputs);
    commands.getSelectedDevices().then((d) => setSelected({ input: d.input, output: d.output }));
  }, []);

  const applySelection = async () => {
    setIsApplying(true);
    setStatus(null);
    setError(null);

    try {
      const applied = await commands.setSelectedDevices(selected.input, selected.output);
      setSelected(applied);
      setStatus('Devices updated. The audio engine was reloaded.');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unable to update devices');
    } finally {
      setIsApplying(false);
    }
  };

  return (
    <div className="devices-panel">
      <h3 className="panel-title">Devices</h3>

      <div className="device-field">
        <label htmlFor="device-input">Input (Microphone)</label>
        <select id="device-input" className="device-select" value={selected.input ?? ''} onChange={(e) => setSelected(s => ({ ...s, input: e.target.value || null }))}>
          <option value="">Default</option>
          {inputs.map((d) => <option key={d} value={d}>{d}</option>)}
        </select>
      </div>

      <div className="device-field">
        <label htmlFor="device-output">Output</label>
        <select id="device-output" className="device-select" value={selected.output ?? ''} onChange={(e) => setSelected(s => ({ ...s, output: e.target.value || null }))}>
          <option value="">Default</option>
          {outputs.map((d) => <option key={d} value={d}>{d}</option>)}
        </select>
      </div>

      <div className="devices-actions">
        <button className="btn-save" onClick={applySelection} disabled={isApplying}>
          {isApplying ? 'Applying...' : 'Apply Devices'}
        </button>
      </div>

      {status && <p className="devices-status">{status}</p>}
      {error && <p className="devices-error">{error}</p>}
      <p className="devices-note">Changing devices while recording is blocked to avoid corrupting the active session.</p>
    </div>
  );
}
