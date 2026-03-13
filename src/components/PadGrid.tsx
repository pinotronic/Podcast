import React, { useCallback } from 'react';
import { useAppStore } from '../store';
import { commands, pickAudioFile } from '../lib/tauri';
import type { Pad } from '../types';

interface PadCellProps {
  pad: Pad;
  bankIndex: number;
  isActive: boolean;
  isSelected: boolean;
  onTrigger: () => void;
  onRightClick: (e: React.MouseEvent) => void;
}

function PadCell({ pad, isActive, isSelected, onTrigger, onRightClick }: PadCellProps) {
  const hasAsset = pad.asset !== null;

  return (
    <button
      className={`pad-cell ${isActive ? 'active' : ''} ${isSelected ? 'selected' : ''} ${!hasAsset ? 'empty' : ''}`}
      style={{ '--pad-color': pad.color } as React.CSSProperties}
      onClick={onTrigger}
      onContextMenu={onRightClick}
      title={hasAsset ? `${pad.name}\n${pad.asset?.name}\n${pad.asset?.duration_secs.toFixed(1)}s` : pad.name}
    >
      <span className="pad-name">{pad.name}</span>
      {hasAsset && (
        <span className="pad-asset-name">{pad.asset?.name}</span>
      )}
      {!hasAsset && <span className="pad-empty-hint">Drop audio</span>}
      {isActive && <div className="pad-ripple" />}
    </button>
  );
}

export function PadGrid() {
  const { project, activePads, selectedPad, triggerPad, stopPad, loadProject, setSelectedPad, setNotice } = useAppStore();

  const handleTrigger = useCallback(
    async (bank: number, slot: number, pad: Pad) => {
      try {
        if (!pad.asset) {
          const path = await pickAudioFile();
          if (!path) return;
          const result = await commands.importAsset(path);
          await commands.assignAssetToPad(bank, slot, result.asset_id);
          await commands.updatePadName(bank, slot, result.name);
          await loadProject();
          setNotice('success', `Asset assigned to ${result.name}.`);
          return;
        }
        const key = `${bank}-${slot}`;
        if (activePads.has(key) && pad.playback_mode === 'Toggle') {
          await stopPad(bank, slot);
        } else {
          await triggerPad(bank, slot);
        }
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Unable to use pad';
        setNotice('error', message);
      }
    },
    [activePads, triggerPad, stopPad, loadProject, setNotice]
  );

  const handleRightClick = useCallback(
    (e: React.MouseEvent, bank: number, slot: number) => {
      e.preventDefault();
      setSelectedPad(bank, slot);
    },
    [setSelectedPad]
  );

  if (!project) return <div className="pad-grid-empty">Loading...</div>;

  const bank = project.banks[project.active_bank];
  if (!bank) return null;

  return (
    <div className="pad-grid">
      {bank.pads.map((pad, i) => (
        <PadCell
          key={pad.id['0'] ?? i}
          pad={pad}
          bankIndex={project.active_bank}
          isActive={activePads.has(`${project.active_bank}-${i}`)}
          isSelected={selectedPad?.bank === project.active_bank && selectedPad?.slot === i}
          onTrigger={() => handleTrigger(project.active_bank, i, pad)}
          onRightClick={(e) => handleRightClick(e, project.active_bank, i)}
        />
      ))}
    </div>
  );
}
