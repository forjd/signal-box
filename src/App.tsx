import { useEffect, useMemo, useState } from "react";
import {
  getAppMetadata,
  getDatabaseHealth,
  type AppMetadata,
  type DatabaseHealth,
} from "./lib/tauri";
import "./App.css";

type RouteId = "inbox" | "projects" | "memory" | "artefacts" | "search" | "settings";

type LoadState<T> =
  | { status: "loading" }
  | { status: "ready"; data: T }
  | { status: "error"; message: string };

const routes: Array<{ id: RouteId; label: string; eyebrow: string; title: string; body: string }> = [
  {
    id: "inbox",
    label: "Inbox",
    eyebrow: "Capture",
    title: "No captures yet",
    body: "The inbox will hold unprocessed developer context while preserving each raw capture.",
  },
  {
    id: "projects",
    label: "Projects",
    eyebrow: "Organize",
    title: "No projects yet",
    body: "Projects will become the home for captures, decisions, questions, sources, and generated artefacts.",
  },
  {
    id: "memory",
    label: "Project Memory",
    eyebrow: "Recall",
    title: "Project memory is ready for structure",
    body: "Later phases will attach distilled context to editable project memory.",
  },
  {
    id: "artefacts",
    label: "Artefacts",
    eyebrow: "Generate",
    title: "No artefacts yet",
    body: "Generated plans, ADRs, prompts, briefs, and drafts will appear here.",
  },
  {
    id: "search",
    label: "Search / Ask",
    eyebrow: "Find",
    title: "Search is reserved",
    body: "Semantic search and ask flows are out of scope for this foundation phase.",
  },
  {
    id: "settings",
    label: "Settings",
    eyebrow: "Local setup",
    title: "Storage foundation",
    body: "Signal Box stores local data in the app data directory and runs SQLite migrations on startup.",
  },
];

function App() {
  const [activeRoute, setActiveRoute] = useState<RouteId>("inbox");
  const [databaseHealth, setDatabaseHealth] = useState<LoadState<DatabaseHealth>>({ status: "loading" });
  const [metadata, setMetadata] = useState<LoadState<AppMetadata>>({ status: "loading" });

  useEffect(() => {
    let isMounted = true;

    async function loadFoundationState() {
      try {
        const [health, appMetadata] = await Promise.all([getDatabaseHealth(), getAppMetadata()]);

        if (!isMounted) {
          return;
        }

        setDatabaseHealth({ status: "ready", data: health });
        setMetadata({ status: "ready", data: appMetadata });
      } catch (error) {
        if (!isMounted) {
          return;
        }

        const message = error instanceof Error ? error.message : String(error);
        setDatabaseHealth({ status: "error", message });
        setMetadata({ status: "error", message });
      }
    }

    loadFoundationState();

    return () => {
      isMounted = false;
    };
  }, []);

  const currentRoute = useMemo(
    () => routes.find((route) => route.id === activeRoute) ?? routes[0],
    [activeRoute],
  );

  return (
    <main className="app-shell">
      <aside className="sidebar" aria-label="Primary navigation">
        <div className="brand-block">
          <span className="brand-mark" aria-hidden="true">
            SB
          </span>
          <div>
            <p className="brand-name">Signal Box</p>
            <p className="brand-subtitle">Local developer context</p>
          </div>
        </div>

        <nav className="nav-list">
          {routes.map((route) => (
            <button
              className="nav-item"
              data-active={route.id === activeRoute}
              key={route.id}
              onClick={() => setActiveRoute(route.id)}
              type="button"
            >
              {route.label}
            </button>
          ))}
        </nav>

        <DatabasePill state={databaseHealth} />
      </aside>

      <section className="workspace" aria-labelledby="view-title">
        <header className="workspace-header">
          <div>
            <p className="eyebrow">{currentRoute.eyebrow}</p>
            <h1 id="view-title">{currentRoute.label}</h1>
          </div>
          <MetadataSummary state={metadata} />
        </header>

        {activeRoute === "settings" ? (
          <SettingsView health={databaseHealth} metadata={metadata} />
        ) : (
          <EmptyState title={currentRoute.title} body={currentRoute.body} />
        )}
      </section>
    </main>
  );
}

function DatabasePill({ state }: { state: LoadState<DatabaseHealth> }) {
  if (state.status === "loading") {
    return <span className="status-pill status-neutral">Checking database</span>;
  }

  if (state.status === "error" || !state.data.ok) {
    return <span className="status-pill status-error">Database attention needed</span>;
  }

  return <span className="status-pill status-ok">Database ready</span>;
}

function MetadataSummary({ state }: { state: LoadState<AppMetadata> }) {
  if (state.status === "loading") {
    return <LoadingState label="Loading app metadata" />;
  }

  if (state.status === "error") {
    return <ErrorState title="Metadata unavailable" message={state.message} compact />;
  }

  return (
    <div className="metadata-summary" aria-label="Application metadata">
      <span>{state.data.productName}</span>
      <span>v{state.data.version}</span>
    </div>
  );
}

function SettingsView({
  health,
  metadata,
}: {
  health: LoadState<DatabaseHealth>;
  metadata: LoadState<AppMetadata>;
}) {
  return (
    <div className="settings-grid">
      <section className="panel" aria-labelledby="database-heading">
        <h2 id="database-heading">Database</h2>
        {health.status === "loading" && <LoadingState label="Checking local database" />}
        {health.status === "error" && <ErrorState title="Database command failed" message={health.message} />}
        {health.status === "ready" &&
          (health.data.ok ? (
            <dl className="detail-list">
              <div>
                <dt>App data directory</dt>
                <dd>{health.data.appDataDir}</dd>
              </div>
              <div>
                <dt>Database file</dt>
                <dd>{health.data.databasePath}</dd>
              </div>
              <div>
                <dt>Latest migration</dt>
                <dd>{health.data.latestMigration ?? "None"}</dd>
              </div>
              <div>
                <dt>Applied migrations</dt>
                <dd>{health.data.appliedMigrations}</dd>
              </div>
            </dl>
          ) : (
            <ErrorState title="Database startup failed" message={health.data.startupError ?? "Unknown startup error"} />
          ))}
      </section>

      <section className="panel" aria-labelledby="app-heading">
        <h2 id="app-heading">Application</h2>
        {metadata.status === "loading" && <LoadingState label="Loading application metadata" />}
        {metadata.status === "error" && <ErrorState title="Metadata unavailable" message={metadata.message} />}
        {metadata.status === "ready" && (
          <dl className="detail-list">
            <div>
              <dt>Product</dt>
              <dd>{metadata.data.productName}</dd>
            </div>
            <div>
              <dt>Package</dt>
              <dd>{metadata.data.packageName}</dd>
            </div>
            <div>
              <dt>Version</dt>
              <dd>{metadata.data.version}</dd>
            </div>
            <div>
              <dt>Database filename</dt>
              <dd>{metadata.data.databaseFileName}</dd>
            </div>
          </dl>
        )}
      </section>
    </div>
  );
}

function EmptyState({ title, body }: { title: string; body: string }) {
  return (
    <section className="empty-state" aria-live="polite">
      <p className="empty-kicker">Foundation placeholder</p>
      <h2>{title}</h2>
      <p>{body}</p>
    </section>
  );
}

function LoadingState({ label }: { label: string }) {
  return (
    <div className="loading-state" role="status">
      <span className="spinner" aria-hidden="true" />
      <span>{label}</span>
    </div>
  );
}

function ErrorState({
  title,
  message,
  compact = false,
}: {
  title: string;
  message: string;
  compact?: boolean;
}) {
  return (
    <div className={compact ? "error-state compact" : "error-state"} role="alert">
      <strong>{title}</strong>
      <span>{message}</span>
    </div>
  );
}

export default App;
