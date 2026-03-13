import { useState, useEffect } from 'react';
import { useAppStore } from '../store';
import { commands } from '../lib/tauri';

export function ProjectPanel() {
  const { project, loadProject, saveProject } = useAppStore();
  const [projects, setProjects] = useState<string[]>([]);
  const [newName, setNewName] = useState('');

  useEffect(() => {
    commands.listProjects().then(setProjects);
  }, [project]);

  const handleCreate = async () => {
    if (!newName.trim()) return;
    await commands.createProject(newName.trim());
    await loadProject();
    setNewName('');
    const list = await commands.listProjects();
    setProjects(list);
  };

  const handleOpen = async (name: string) => {
    await commands.openProject(name);
    await loadProject();
  };

  return (
    <div className="project-panel">
      <h3 className="panel-title">Project</h3>

      {project && (
        <div className="current-project">
          <div className="project-name">{project.name}</div>
          <div className="project-meta">v{project.app_version} · {project.sample_rate}Hz</div>
          <button className="btn-save-project" onClick={saveProject}>💾 Save</button>
        </div>
      )}

      <div className="new-project">
        <h4>New Project</h4>
        <div className="new-project-row">
          <input
            type="text"
            placeholder="Project name"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
            className="editor-input"
          />
          <button className="btn-create" onClick={handleCreate}>Create</button>
        </div>
      </div>

      {projects.length > 0 && (
        <div className="project-list">
          <h4>Open Project</h4>
          {projects.map((p) => (
            <button
              key={p}
              className={`project-item ${project?.name === p ? 'active' : ''}`}
              onClick={() => handleOpen(p)}
            >
              {p}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
