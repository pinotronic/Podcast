import { useAppStore } from '../store';

export function BankSelector() {
  const { project, setActiveBank } = useAppStore();
  if (!project) return null;

  return (
    <div className="bank-selector">
      {project.banks.map((bank, i) => (
        <button
          key={i}
          className={`bank-btn ${project.active_bank === i ? 'active' : ''}`}
          onClick={() => setActiveBank(i)}
        >
          {bank.name}
        </button>
      ))}
    </div>
  );
}
