import { useAppStore } from '../store';

interface MeterBarProps {
  level: number;
  label: string;
}

function MeterBar({ level, label }: MeterBarProps) {
  const pct = Math.min(level * 100, 100);
  const color = pct > 90 ? '#ff3333' : pct > 75 ? '#ffaa00' : '#33cc66';

  return (
    <div className="meter-bar-wrapper">
      <span className="meter-label">{label}</span>
      <div className="meter-track">
        <div
          className="meter-fill"
          style={{ width: `${pct}%`, background: color }}
        />
      </div>
      <span className="meter-value">{pct.toFixed(0)}</span>
    </div>
  );
}

export function LevelMeter() {
  const { levels } = useAppStore();

  return (
    <div className="level-meter">
      <MeterBar level={levels.master_l} label="L" />
      <MeterBar level={levels.master_r} label="R" />
      <MeterBar level={levels.mic} label="MIC" />
    </div>
  );
}
