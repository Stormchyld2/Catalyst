import { useState } from "react";
import "./app.css";
import catalystLogo from "./assets/catalyst-logo.png";

type ViewId =
  | "vault"
  | "manuscript"
  | "story"
  | "plot"
  | "timeline"
  | "graph"
  | "github"
  | "terminal";

const views: Array<{ id: ViewId; label: string; mark: string }> = [
  { id: "vault", label: "Vault", mark: "V" },
  { id: "manuscript", label: "Manuscript", mark: "M" },
  { id: "story", label: "Story Bible", mark: "S" },
  { id: "plot", label: "Plot", mark: "P" },
  { id: "timeline", label: "Timeline", mark: "T" },
  { id: "graph", label: "Graph", mark: "G" },
  { id: "github", label: "GitHub", mark: "⇄" },
  { id: "terminal", label: "Terminal", mark: ">" },
];

function App() {
  const [activeView, setActiveView] = useState<ViewId>("vault");

  const activeLabel =
    views.find((view) => view.id === activeView)?.label ?? "Vault";

  return (
    <div className="app-shell">
      <header className="titlebar">
        <div className="brand">
          <span className="brand-name"><span className="brand-gold">Litho</span><span className="brand-blue">Mark</span></span>
        </div>
        <div className="project-title">No project open</div>
      </header>

      <div className="workspace">
        <nav className="activity-bar" aria-label="Primary views">
          {views.map((view) => (
            <button
              key={view.id}
              className={"activity-button " + (activeView === view.id ? "active" : "")}
              onClick={() => setActiveView(view.id)}
              title={view.label}
              aria-label={view.label}
            >
              <span>{view.mark}</span>
            </button>
          ))}
        </nav>

        <aside className="sidebar">
          <div className="panel-heading">
            <span>{activeLabel}</span>
            <button className="ghost-button" aria-label="More options">
              •••
            </button>
          </div>

          {activeView === "vault" ? (
            <div className="empty-panel">
              <p className="empty-title">No vault open</p>
              <p className="empty-copy">
                Open a local Catalyst project to browse its folders and files.
              </p>
              <button className="primary-button">Open Project</button>
            </div>
          ) : (
            <div className="empty-panel">
              <p className="empty-title">{activeLabel}</p>
              <p className="empty-copy">
                This workspace will be connected to Catalyst&apos;s file-first
                project model.
              </p>
            </div>
          )}
        </aside>

        <main className="editor-area">
          <div className="editor-tabs">
            <div className="editor-tab active">Welcome</div>
          </div>

          <section className="welcome">
            <img className="welcome-logo" src={catalystLogo} alt="Catalyst" />
            <div className="welcome-actions">
              <button className="welcome-action">
                <span>New Project</span>
                <small>Create a local-first writing vault</small>
              </button>
              <button className="welcome-action">
                <span>Open Local Project</span>
                <small>Open an existing folder from this computer</small>
              </button>
              <button className="welcome-action">
                <span>Clone GitHub Repository</span>
                <small>Bring down an existing writing repository</small>
              </button>
            </div>

            <p className="principle">
              Your files are the source of truth. Catalyst is the workspace
              around them.
            </p>
          </section>
        </main>

      </div>

      <footer className="statusbar">
        <div className="status-left">
          <span>main</span>
          <span>↓0 ↑0</span>
        </div>
        <div className="status-right">
          <span>Offline-ready</span>
          <span>UTF-8</span>
          <span>Markdown</span>
        </div>
      </footer>
    </div>
  );
}

export default App;
