import { useAppStore } from '../store';

function formatTime(secs: number) {
  const m = Math.floor(secs / 60).toString().padStart(2, '0');
  const s = Math.floor(secs % 60).toString().padStart(2, '0');
  return `${m}:${s}`;
}

export function RecordingBar() {
  const { recordingState, recordingDuration, startRecording, stopRecording, stopAllPads } = useAppStore();

  const isRecording = recordingState === 'recording';

  return (
    <div className={`recording-bar ${isRecording ? 'recording' : ''}`}>
      <div className="rec-indicator">
        <div className={`rec-dot ${isRecording ? 'blink' : ''}`} />
        <span>{isRecording ? 'REC' : 'READY'}</span>
      </div>

      {isRecording && (
        <div className="rec-timer">{formatTime(recordingDuration)}</div>
      )}

      <div className="rec-controls">
        <button
          className={`btn-record ${isRecording ? 'active' : ''}`}
          onClick={isRecording ? stopRecording : startRecording}
          title={isRecording ? 'Stop Recording' : 'Start Recording'}
        >
          {isRecording ? '⏹ STOP' : '⏺ REC'}
        </button>

        <button
          className="btn-stop-all"
          onClick={stopAllPads}
          title="Stop all pads"
        >
          ⏹ ALL
        </button>
      </div>
    </div>
  );
}
