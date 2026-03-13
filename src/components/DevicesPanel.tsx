import { useState, useEffect } from 'react';
import { commands } from '../lib/tauri';
import { useAppStore } from '../store';

export function DevicesPanel() {
  const { setNotice } = useAppStore();
  const [inputs, setInputs] = useState<string[]>([]);
  const [outputs, setOutputs] = useState<string[]>([]);
  const [selected, setSelected] = useState<{ input: string | null; output: string | null }>({ input: null, output: null });
  const [isApplying, setIsApplying] = useState(false);

  useEffect(() => {
    const loadDevices = async () => {
      try {
        const [availableInputs, availableOutputs, currentSelection] = await Promise.all([
          commands.listInputDevices(),
          commands.listOutputDevices(),
          commands.getSelectedDevices(),
        ]);
        setInputs(availableInputs);
        setOutputs(availableOutputs);
        setSelected({ input: currentSelection.input, output: currentSelection.output });
      } catch (err) {
        setNotice('error', err instanceof Error ? err.message : 'Unable to query the available audio devices.');
      }
    };

    loadDevices();
  }, [setNotice]);

  const applySelection = async () => {
    setIsApplying(true);

    try {
      const applied = await commands.setSelectedDevices(selected.input, selected.output);
      setSelected(applied);
      setNotice('success', 'Devices updated. The audio engine was reloaded.');
    } catch (err) {
      setNotice('error', err instanceof Error ? err.message : 'Unable to update devices');
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

      <p className="devices-note">Changing devices while recording is blocked to avoid corrupting the active session.</p>
    </div>
  );
}
