import { useEffect } from 'react';
import { useAppStore } from './store';
import { PadGrid } from './components/PadGrid';
import { BankSelector } from './components/BankSelector';
import { RecordingBar } from './components/RecordingBar';
import { LevelMeter } from './components/LevelMeter';
import { MixerPanel } from './components/MixerPanel';
import { PadEditor } from './components/PadEditor';
import { DevicesPanel } from './components/DevicesPanel';
import { ProjectPanel } from './components/ProjectPanel';
import './App.css';

export default function App() {
  const { loadProject, refreshLevels, refreshRecordingDuration, activePanel, setActivePanel, selectedPad } = useAppStore();

  useEffect(() => {
    loadProject();
  }, []);

  // Poll levels every 80ms
  useEffect(() => {
    const id = setInterval(refreshLevels, 80);
    return () => clearInterval(id);
  }, []);

  // Poll recording duration every second
  useEffect(() => {
    const id = setInterval(refreshRecordingDuration, 1000);
    return () => clearInterval(id);
  }, []);

  return (
    <div className="app">
      {/* Top bar */}
      <header className="top-bar">
        <div className="app-logo">🎙 Podcast Console</div>
        <nav className="nav-tabs">
          {(['pads', 'mixer', 'devices', 'project'] as const).map((p) => (
            <button
              key={p}
              className={`nav-tab ${activePanel === p ? 'active' : ''}`}
              onClick={() => setActivePanel(p)}
            >
              {p.charAt(0).toUpperCase() + p.slice(1)}
            </button>
          ))}
        </nav>
        <RecordingBar />
      </header>

      {/* Main content */}
      <main className="main-content">
        {activePanel === 'pads' && (
          <div className="pads-view">
            <div className="pads-left">
              <BankSelector />
              <PadGrid />
            </div>
            {selectedPad !== null && (
              <div className="pads-right">
                <PadEditor />
              </div>
            )}
          </div>
        )}
        {activePanel === 'mixer' && <MixerPanel />}
        {activePanel === 'devices' && <DevicesPanel />}
        {activePanel === 'project' && <ProjectPanel />}
      </main>

      {/* Bottom level meters */}
      <footer className="bottom-bar">
        <LevelMeter />
      </footer>
    </div>
  );
}
