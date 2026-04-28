import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Archive,
  CheckCircle2,
  ExternalLink,
  Inbox,
  PanelTopOpen,
  RefreshCw,
  Save,
  Settings2,
  Sparkles,
} from "lucide-react";
import { type ReactNode, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { toast, Toaster } from "sonner";

import { EmptyState, ErrorState, LoadingState } from "@/components/common/state-views";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { Textarea } from "@/components/ui/textarea";
import {
  createCapture,
  getCaptureDistillation,
  getAppMetadata,
  getDatabaseHealth,
  getProviderSettings,
  listCaptures,
  listProjects,
  processCapture,
  saveProviderSettings,
  showQuickCapture,
  testProviderSettings,
  updateCaptureProject,
  updateCaptureStatus,
  type AppMetadata,
  type Capture,
  type CaptureDistillation,
  type CaptureStatus,
  type CaptureType,
  type DatabaseHealth,
  type DistilledDecision,
  type DistilledQuestion,
  type DistilledSource,
  type DistilledTask,
  type ProviderSettings,
  type ProviderSettingsInput,
  type ProviderType,
  type ProjectOption,
  type SourceKind,
} from "./lib/tauri";
import "./App.css";

type RouteId = "inbox" | "projects" | "memory" | "artefacts" | "search" | "settings";

type LoadState<T> =
  | { status: "loading" }
  | { status: "ready"; data: T }
  | { status: "error"; message: string };

const currentWindow = getCurrentWindow();
const isQuickCaptureWindow = currentWindow.label === "quick-capture";

const routes: Array<{ id: RouteId; label: string; eyebrow: string; title: string; body: string }> =
  [
    {
      id: "inbox",
      label: "Inbox",
      eyebrow: "Capture",
      title: "No unprocessed captures",
      body: "New raw developer context will land here until you archive it or later distil it.",
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
      body: "Semantic search and ask flows are out of scope for this phase.",
    },
    {
      id: "settings",
      label: "Settings",
      eyebrow: "Local setup",
      title: "Storage foundation",
      body: "Signal Box stores local data in the app data directory and runs SQLite migrations on startup.",
    },
  ];

const captureTypeLabels: Record<CaptureType, string> = {
  note: "Note",
  url: "URL",
  code: "Code",
  terminal: "Terminal",
  ai_chat: "AI chat",
  github_issue: "GitHub issue",
};

const sourceKindLabels: Record<SourceKind, string> = {
  typed: "Typed",
  pasted_url: "Pasted URL",
  code: "Code",
  terminal: "Terminal",
  ai_chat: "AI chat",
  github: "GitHub",
};

const providerTypeLabels: Record<ProviderType, string> = {
  openai: "OpenAI",
  openrouter: "OpenRouter",
  ollama: "Ollama",
};

function App() {
  if (isQuickCaptureWindow) {
    return <QuickCaptureWindow />;
  }

  return <MainWindow />;
}

function MainWindow() {
  const [activeRoute, setActiveRoute] = useState<RouteId>("inbox");
  const [databaseHealth, setDatabaseHealth] = useState<LoadState<DatabaseHealth>>({
    status: "loading",
  });
  const [metadata, setMetadata] = useState<LoadState<AppMetadata>>({ status: "loading" });
  const [captures, setCaptures] = useState<LoadState<Capture[]>>({ status: "loading" });
  const [projects, setProjects] = useState<LoadState<ProjectOption[]>>({ status: "loading" });
  const [selectedCaptureId, setSelectedCaptureId] = useState<string | null>(null);

  async function loadFoundationState() {
    try {
      const [health, appMetadata, unprocessedCaptures, projectOptions] = await Promise.all([
        getDatabaseHealth(),
        getAppMetadata(),
        listCaptures("unprocessed"),
        listProjects(),
      ]);

      setDatabaseHealth({ status: "ready", data: health });
      setMetadata({ status: "ready", data: appMetadata });
      setCaptures({ status: "ready", data: unprocessedCaptures });
      setProjects({ status: "ready", data: projectOptions });
      setSelectedCaptureId((current) => current ?? unprocessedCaptures[0]?.id ?? null);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setDatabaseHealth({ status: "error", message });
      setMetadata({ status: "error", message });
      setCaptures({ status: "error", message });
      setProjects({ status: "error", message });
    }
  }

  useEffect(() => {
    loadFoundationState();
  }, []);

  const currentRoute = useMemo(
    () => routes.find((route) => route.id === activeRoute) ?? routes[0],
    [activeRoute],
  );

  const selectedCapture =
    captures.status === "ready"
      ? (captures.data.find((capture) => capture.id === selectedCaptureId) ?? captures.data[0])
      : undefined;

  async function refreshInbox() {
    const [unprocessedCaptures, projectOptions] = await Promise.all([
      listCaptures("unprocessed"),
      listProjects(),
    ]);
    setCaptures({ status: "ready", data: unprocessedCaptures });
    setProjects({ status: "ready", data: projectOptions });
    setSelectedCaptureId((current) => {
      if (current && unprocessedCaptures.some((capture) => capture.id === current)) {
        return current;
      }

      return unprocessedCaptures[0]?.id ?? null;
    });
  }

  async function archiveCapture(id: string) {
    try {
      await updateCaptureStatus(id, "archived");
      await refreshInbox();
      toast.success("Capture archived");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function changeCaptureStatus(id: string, status: CaptureStatus) {
    try {
      await updateCaptureStatus(id, status);
      await refreshInbox();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function changeCaptureProject(id: string, projectId: string | null) {
    try {
      const updated = await updateCaptureProject(id, projectId);
      setCaptures((current) =>
        current.status === "ready"
          ? {
              status: "ready",
              data: current.data.map((capture) => (capture.id === id ? updated : capture)),
            }
          : current,
      );
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  const replaceCapture = useCallback((updated: Capture) => {
    setCaptures((current) =>
      current.status === "ready"
        ? {
            status: "ready",
            data: current.data.map((capture) => (capture.id === updated.id ? updated : capture)),
          }
        : current,
    );
  }, []);

  return (
    <main className="app-shell">
      <Toaster richColors position="bottom-right" />
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
            <Button
              className="nav-item"
              data-active={route.id === activeRoute}
              key={route.id}
              onClick={() => setActiveRoute(route.id)}
              type="button"
              variant="ghost"
            >
              {route.label}
            </Button>
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
          <div className="header-actions">
            {activeRoute === "inbox" && (
              <Button type="button" onClick={() => showQuickCapture()} size="sm">
                <PanelTopOpen aria-hidden="true" />
                Quick capture
              </Button>
            )}
            <MetadataSummary state={metadata} />
          </div>
        </header>

        {activeRoute === "settings" && <SettingsView health={databaseHealth} metadata={metadata} />}
        {activeRoute === "inbox" && (
          <InboxView
            captures={captures}
            projects={projects}
            selectedCapture={selectedCapture}
            selectedCaptureId={selectedCaptureId}
            onArchive={archiveCapture}
            onRefresh={refreshInbox}
            onSelect={setSelectedCaptureId}
            onStatusChange={changeCaptureStatus}
            onProjectChange={changeCaptureProject}
            onCaptureChange={replaceCapture}
          />
        )}
        {activeRoute !== "settings" && activeRoute !== "inbox" && (
          <EmptyState title={currentRoute.title} body={currentRoute.body} />
        )}
      </section>
    </main>
  );
}

function QuickCaptureWindow() {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [rawText, setRawText] = useState("");
  const [projects, setProjects] = useState<ProjectOption[]>([]);
  const [projectId, setProjectId] = useState<string | null>(null);
  const [captureType, setCaptureType] = useState<CaptureType>("note");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    textareaRef.current?.focus();
    listProjects()
      .then(setProjects)
      .catch((loadError) =>
        setError(loadError instanceof Error ? loadError.message : String(loadError)),
      );
  }, []);

  useEffect(() => {
    if (!rawText.trim()) {
      setCaptureType("note");
      return;
    }

    setCaptureType(detectCapture(rawText).captureType);
  }, [rawText]);

  async function save(autoClose: boolean, processAfter = false) {
    setError(null);
    setSaving(true);

    try {
      const detection = detectCapture(rawText);
      const saved = await createCapture({
        rawText,
        captureType,
        sourceKind: detection.sourceKind,
        source: detection.source,
        projectId,
      });
      if (processAfter) {
        await processCapture(saved.id);
      }
      setRawText("");
      toast.success(processAfter ? "Capture saved and distilled" : "Capture saved");

      if (autoClose) {
        await currentWindow.hide();
      } else {
        textareaRef.current?.focus();
      }
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : String(saveError));
    } finally {
      setSaving(false);
    }
  }

  return (
    <main className="quick-capture-shell">
      <Toaster richColors position="bottom-center" />
      <header className="quick-capture-header">
        <div>
          <p className="eyebrow">Capture</p>
          <h1>Quick Capture</h1>
        </div>
        <Button
          type="button"
          variant="ghost"
          onClick={() => currentWindow.hide()}
          aria-label="Close quick capture"
        >
          Esc
        </Button>
      </header>

      <Textarea
        ref={textareaRef}
        className="quick-capture-textarea"
        value={rawText}
        onChange={(event) => setRawText(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            currentWindow.hide();
          }
          if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
            save(true);
          }
        }}
        placeholder="Paste a thought, URL, snippet, terminal output, AI chat excerpt, or issue text."
      />

      <div className="quick-capture-controls">
        <div className="form-field">
          <span>Project</span>
          <ProjectSelect projects={projects} value={projectId} onChange={setProjectId} />
        </div>
        <div className="form-field">
          <span>Type</span>
          <CaptureTypeSelect value={captureType} onChange={setCaptureType} />
        </div>
        <Badge variant="secondary">{sourceKindLabels[detectCapture(rawText).sourceKind]}</Badge>
      </div>

      {error && <ErrorState title="Could not save capture" message={error} compact />}

      <footer className="quick-capture-actions">
        <Button type="button" variant="outline" onClick={() => currentWindow.hide()}>
          Cancel
        </Button>
        <Button
          type="button"
          variant="secondary"
          disabled={!rawText.trim() || saving}
          onClick={() => {
            save(false, true);
          }}
        >
          <Sparkles aria-hidden="true" />
          Process now
        </Button>
        <Button type="button" disabled={!rawText.trim() || saving} onClick={() => save(true)}>
          <Save aria-hidden="true" />
          Save raw
        </Button>
      </footer>
    </main>
  );
}

function InboxView({
  captures,
  projects,
  selectedCapture,
  selectedCaptureId,
  onArchive,
  onRefresh,
  onSelect,
  onStatusChange,
  onProjectChange,
  onCaptureChange,
}: {
  captures: LoadState<Capture[]>;
  projects: LoadState<ProjectOption[]>;
  selectedCapture: Capture | undefined;
  selectedCaptureId: string | null;
  onArchive: (id: string) => void;
  onRefresh: () => void;
  onSelect: (id: string) => void;
  onStatusChange: (id: string, status: CaptureStatus) => void;
  onProjectChange: (id: string, projectId: string | null) => void;
  onCaptureChange: (capture: Capture) => void;
}) {
  if (captures.status === "loading") {
    return <LoadingState label="Loading capture inbox" />;
  }

  if (captures.status === "error") {
    return <ErrorState title="Inbox unavailable" message={captures.message} />;
  }

  if (captures.data.length === 0) {
    return (
      <section className="inbox-empty">
        <EmptyState
          kicker="Inbox"
          title="No unprocessed captures"
          body="Use the global hotkey or the quick capture button to save raw context locally."
        />
        <Button type="button" onClick={() => showQuickCapture()}>
          <PanelTopOpen aria-hidden="true" />
          Quick capture
        </Button>
      </section>
    );
  }

  return (
    <div className="inbox-layout">
      <section className="capture-list" aria-label="Unprocessed captures">
        <div className="list-toolbar">
          <div>
            <p className="eyebrow">Unprocessed</p>
            <h2>{captures.data.length} captures</h2>
          </div>
          <Button type="button" variant="outline" size="sm" onClick={onRefresh}>
            <RefreshCw aria-hidden="true" />
            Refresh
          </Button>
        </div>
        <div className="capture-list-items">
          {captures.data.map((capture) => (
            <button
              className="capture-row"
              data-active={capture.id === selectedCaptureId}
              key={capture.id}
              onClick={() => onSelect(capture.id)}
              type="button"
            >
              <span className="capture-row-icon">
                <Inbox aria-hidden="true" />
              </span>
              <span className="capture-row-main">
                <strong>{capture.title || fallbackTitle(capture.rawText)}</strong>
                <span>{previewText(capture.rawText)}</span>
                <span className="capture-row-meta">
                  <Badge variant="secondary">{captureTypeLabels[capture.captureType]}</Badge>
                  <Badge variant="outline">{capture.projectId ? "Assigned" : "Unassigned"}</Badge>
                  <span>{formatAge(capture.createdAt)}</span>
                </span>
              </span>
            </button>
          ))}
        </div>
      </section>

      {selectedCapture && (
        <CaptureDetail
          capture={selectedCapture}
          projects={projects}
          onArchive={onArchive}
          onStatusChange={onStatusChange}
          onProjectChange={onProjectChange}
          onCaptureChange={onCaptureChange}
        />
      )}
    </div>
  );
}

function CaptureDetail({
  capture,
  projects,
  onArchive,
  onStatusChange,
  onProjectChange,
  onCaptureChange,
}: {
  capture: Capture;
  projects: LoadState<ProjectOption[]>;
  onArchive: (id: string) => void;
  onStatusChange: (id: string, status: CaptureStatus) => void;
  onProjectChange: (id: string, projectId: string | null) => void;
  onCaptureChange: (capture: Capture) => void;
}) {
  const projectOptions = projects.status === "ready" ? projects.data : [];
  const [distillation, setDistillation] = useState<LoadState<CaptureDistillation>>({
    status: "loading",
  });
  const [processing, setProcessing] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setDistillation({ status: "loading" });

    getCaptureDistillation(capture.id)
      .then((result) => {
        if (!cancelled) {
          setDistillation({ status: "ready", data: result });
          onCaptureChange(result.capture);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setDistillation({
            status: "error",
            message: error instanceof Error ? error.message : String(error),
          });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [capture.id, onCaptureChange]);

  async function runProcessing() {
    setProcessing(true);
    setDistillation({ status: "loading" });

    try {
      const result = await processCapture(capture.id);
      setDistillation({ status: "ready", data: result });
      onCaptureChange(result.capture);
      toast.success("Capture distilled");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setDistillation({ status: "error", message });
      toast.error(message);
    } finally {
      setProcessing(false);
    }
  }

  return (
    <section className="capture-detail" aria-label="Capture detail">
      <header className="detail-header">
        <div>
          <p className="eyebrow">Raw capture</p>
          <h2>{capture.title || fallbackTitle(capture.rawText)}</h2>
        </div>
        <div className="detail-actions">
          <Button type="button" onClick={runProcessing} disabled={processing}>
            <Sparkles aria-hidden="true" />
            {processing ? "Processing" : "Process"}
          </Button>
          <Button type="button" variant="outline" onClick={() => onArchive(capture.id)}>
            <Archive aria-hidden="true" />
            Archive
          </Button>
        </div>
      </header>

      <div className="detail-controls">
        <div className="form-field">
          <span>Status</span>
          <Select
            value={capture.status}
            onValueChange={(value) => onStatusChange(capture.id, value as CaptureStatus)}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="unprocessed">Unprocessed</SelectItem>
              <SelectItem value="processed">Processed</SelectItem>
              <SelectItem value="archived">Archived</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="form-field">
          <span>Project</span>
          <ProjectSelect
            projects={projectOptions}
            value={capture.projectId}
            onChange={(projectId) => onProjectChange(capture.id, projectId)}
          />
        </div>
      </div>

      <div className="metadata-strip">
        <Badge variant="secondary">{captureTypeLabels[capture.captureType]}</Badge>
        <Badge variant="outline">{sourceKindLabels[capture.sourceKind]}</Badge>
        <Badge variant="outline">{capture.suggestedProjectName ?? "No project suggestion"}</Badge>
        <Badge variant={capture.processingStatus === "failed" ? "destructive" : "secondary"}>
          {capture.processingStatus}
        </Badge>
        {capture.source && (
          <span className="source-link">
            <ExternalLink aria-hidden="true" />
            {capture.source}
          </span>
        )}
      </div>

      {capture.processingError && (
        <div className="detail-alert">
          <ErrorState title="Processing failed" message={capture.processingError} compact />
        </div>
      )}

      <pre className="raw-capture">{capture.rawText}</pre>

      <DistillationReview state={distillation} />

      <dl className="detail-list compact-list">
        <div>
          <dt>Created</dt>
          <dd>{formatDateTime(capture.createdAt)}</dd>
        </div>
        <div>
          <dt>Updated</dt>
          <dd>{formatDateTime(capture.updatedAt)}</dd>
        </div>
        <div>
          <dt>Processed</dt>
          <dd>{capture.processedAt ? formatDateTime(capture.processedAt) : "Not processed"}</dd>
        </div>
        <div>
          <dt>Archived</dt>
          <dd>{capture.archivedAt ? formatDateTime(capture.archivedAt) : "Not archived"}</dd>
        </div>
      </dl>
    </section>
  );
}

function DistillationReview({ state }: { state: LoadState<CaptureDistillation> }) {
  if (state.status === "loading") {
    return (
      <section className="distillation-panel">
        <LoadingState label="Loading extracted objects" compact />
      </section>
    );
  }

  if (state.status === "error") {
    return (
      <section className="distillation-panel">
        <ErrorState title="Extraction unavailable" message={state.message} compact />
      </section>
    );
  }

  const { capture, tasks, decisions, questions, sources } = state.data;
  const hasObjects =
    tasks.length > 0 || decisions.length > 0 || questions.length > 0 || sources.length > 0;

  return (
    <section className="distillation-panel" aria-label="Extracted objects">
      <div className="distillation-header">
        <div>
          <p className="eyebrow">Distilled memory</p>
          <h3>{capture.summary ? "Structured extraction" : "No extraction yet"}</h3>
        </div>
        {capture.processedAt && (
          <Badge variant="secondary">
            <CheckCircle2 aria-hidden="true" />
            Processed
          </Badge>
        )}
      </div>

      {capture.summary && <p className="distillation-summary">{capture.summary}</p>}

      {!hasObjects && (
        <p className="muted-copy">
          Process this capture to save tasks, decisions, questions, and sources beside the raw text.
        </p>
      )}

      <div className="distillation-grid">
        <ObjectGroup title="Tasks" items={tasks} renderItem={(task) => <TaskItem task={task} />} />
        <ObjectGroup
          title="Decisions"
          items={decisions}
          renderItem={(decision) => <DecisionItem decision={decision} />}
        />
        <ObjectGroup
          title="Questions"
          items={questions}
          renderItem={(question) => <QuestionItem question={question} />}
        />
        <ObjectGroup
          title="Sources"
          items={sources}
          renderItem={(source) => <SourceItem source={source} />}
        />
      </div>
    </section>
  );
}

function ObjectGroup<T>({
  title,
  items,
  renderItem,
}: {
  title: string;
  items: T[];
  renderItem: (item: T) => ReactNode;
}) {
  if (items.length === 0) {
    return null;
  }

  return (
    <div className="object-group">
      <h4>{title}</h4>
      <div className="object-list">{items.map(renderItem)}</div>
    </div>
  );
}

function TaskItem({ task }: { task: DistilledTask }) {
  return (
    <article className="object-item" key={task.id}>
      <strong>{task.title}</strong>
      {task.description && <p>{task.description}</p>}
    </article>
  );
}

function DecisionItem({ decision }: { decision: DistilledDecision }) {
  return (
    <article className="object-item" key={decision.id}>
      <strong>{decision.title}</strong>
      <p>{decision.decision}</p>
      {decision.rationale && <small>{decision.rationale}</small>}
    </article>
  );
}

function QuestionItem({ question }: { question: DistilledQuestion }) {
  return (
    <article className="object-item" key={question.id}>
      <strong>{question.question}</strong>
      {question.answer && <p>{question.answer}</p>}
    </article>
  );
}

function SourceItem({ source }: { source: DistilledSource }) {
  return (
    <article className="object-item" key={source.id}>
      <strong>{source.title}</strong>
      <Badge variant="outline">{source.sourceType}</Badge>
      {source.url && <p>{source.url}</p>}
      {source.rawReference && <small>{source.rawReference}</small>}
    </article>
  );
}

function ProjectSelect({
  projects,
  value,
  onChange,
}: {
  projects: ProjectOption[];
  value: string | null;
  onChange: (value: string | null) => void;
}) {
  return (
    <Select
      value={value ? packProjectId(value) : "unassigned"}
      onValueChange={(next) => onChange(unpackProjectId(next))}
    >
      <SelectTrigger>
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="unassigned">Unassigned</SelectItem>
        {projects.map((project) => (
          <SelectItem key={project.id} value={packProjectId(project.id)}>
            {project.name}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}

function CaptureTypeSelect({
  value,
  onChange,
}: {
  value: CaptureType;
  onChange: (value: CaptureType) => void;
}) {
  return (
    <Select value={value} onValueChange={(next) => onChange(next as CaptureType)}>
      <SelectTrigger>
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {Object.entries(captureTypeLabels).map(([type, label]) => (
          <SelectItem key={type} value={type}>
            {label}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}

function DatabasePill({ state }: { state: LoadState<DatabaseHealth> }) {
  if (state.status === "loading") {
    return (
      <Badge className="status-pill" variant="secondary">
        Checking database
      </Badge>
    );
  }

  if (state.status === "error" || !state.data.ok) {
    return (
      <Badge className="status-pill" variant="destructive">
        Database attention needed
      </Badge>
    );
  }

  return (
    <Badge className="status-pill" variant="secondary">
      Database ready
    </Badge>
  );
}

function MetadataSummary({ state }: { state: LoadState<AppMetadata> }) {
  if (state.status === "loading") {
    return <LoadingState label="Loading app metadata" compact />;
  }

  if (state.status === "error") {
    return <ErrorState title="Metadata unavailable" message={state.message} compact />;
  }

  return (
    <div className="metadata-summary" aria-label="Application metadata">
      <span>{state.data.productName}</span>
      <span>v{state.data.version}</span>
      <span>{state.data.quickCaptureHotkey}</span>
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
      <ProviderSettingsCard />

      <Card aria-labelledby="database-heading">
        <CardHeader>
          <CardTitle id="database-heading">Database</CardTitle>
        </CardHeader>
        <Separator />
        <CardContent>
          {health.status === "loading" && <LoadingState label="Checking local database" />}
          {health.status === "error" && (
            <ErrorState title="Database command failed" message={health.message} />
          )}
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
              <ErrorState
                title="Database startup failed"
                message={health.data.startupError ?? "Unknown startup error"}
              />
            ))}
        </CardContent>
      </Card>

      <Card aria-labelledby="app-heading">
        <CardHeader>
          <CardTitle id="app-heading">Application</CardTitle>
        </CardHeader>
        <Separator />
        <CardContent>
          {metadata.status === "loading" && <LoadingState label="Loading application metadata" />}
          {metadata.status === "error" && (
            <ErrorState title="Metadata unavailable" message={metadata.message} />
          )}
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
              <div>
                <dt>Quick capture hotkey</dt>
                <dd>{metadata.data.quickCaptureHotkey}</dd>
              </div>
            </dl>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function ProviderSettingsCard() {
  const [state, setState] = useState<LoadState<ProviderSettings>>({ status: "loading" });
  const [form, setForm] = useState<ProviderSettingsInput>({
    providerType: "openai",
    baseUrl: "https://api.openai.com/v1",
    apiKey: "",
    chatModel: "gpt-4.1-mini",
    embeddingModel: "",
    ollamaBaseUrl: "http://localhost:11434",
  });
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testMessage, setTestMessage] = useState<string | null>(null);

  useEffect(() => {
    getProviderSettings()
      .then((settings) => {
        setState({ status: "ready", data: settings });
        setForm(providerSettingsToInput(settings));
      })
      .catch((error) =>
        setState({
          status: "error",
          message: error instanceof Error ? error.message : String(error),
        }),
      );
  }, []);

  async function saveSettings() {
    setSaving(true);
    setTestMessage(null);

    try {
      const settings = await saveProviderSettings(form);
      setState({ status: "ready", data: settings });
      setForm(providerSettingsToInput(settings));
      toast.success("Provider settings saved");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  async function testSettings() {
    setTesting(true);
    setTestMessage(null);

    try {
      const result = await testProviderSettings(form);
      setTestMessage(result.message);
      toast.success(result.message);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setTestMessage(message);
      toast.error(message);
    } finally {
      setTesting(false);
    }
  }

  return (
    <Card className="provider-card" aria-labelledby="provider-heading">
      <CardHeader>
        <CardTitle id="provider-heading">AI Provider</CardTitle>
      </CardHeader>
      <Separator />
      <CardContent>
        {state.status === "loading" && <LoadingState label="Loading provider settings" />}
        {state.status === "error" && (
          <ErrorState title="Provider settings unavailable" message={state.message} />
        )}
        {state.status === "ready" && (
          <div className="provider-settings">
            <div className="provider-status">
              <Badge variant="secondary">
                <Settings2 aria-hidden="true" />
                {providerTypeLabels[form.providerType]}
              </Badge>
              {state.data.hasApiKey && form.providerType !== "ollama" && (
                <Badge variant="outline">API key saved</Badge>
              )}
            </div>

            <div className="settings-form-grid">
              <div className="form-field">
                <span>Provider</span>
                <Select
                  value={form.providerType}
                  onValueChange={(providerType) =>
                    setForm((current) => providerDefaults(providerType as ProviderType, current))
                  }
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {Object.entries(providerTypeLabels).map(([type, label]) => (
                      <SelectItem key={type} value={type}>
                        {label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <label className="form-field" htmlFor="provider-chat-model">
                <span>Chat model</span>
                <Input
                  id="provider-chat-model"
                  value={form.chatModel}
                  onChange={(event) =>
                    setForm((current) => ({ ...current, chatModel: event.target.value }))
                  }
                  placeholder="gpt-4.1-mini"
                />
              </label>

              {form.providerType !== "ollama" && (
                <>
                  <label className="form-field" htmlFor="provider-base-url">
                    <span>Base URL</span>
                    <Input
                      id="provider-base-url"
                      value={form.baseUrl ?? ""}
                      onChange={(event) =>
                        setForm((current) => ({ ...current, baseUrl: event.target.value }))
                      }
                      placeholder="https://api.openai.com/v1"
                    />
                  </label>
                  <label className="form-field" htmlFor="provider-api-key">
                    <span>API key</span>
                    <Input
                      id="provider-api-key"
                      value={form.apiKey}
                      type="password"
                      onChange={(event) =>
                        setForm((current) => ({ ...current, apiKey: event.target.value }))
                      }
                      placeholder={state.data.hasApiKey ? "Saved key preserved" : "sk-..."}
                    />
                  </label>
                </>
              )}

              {form.providerType === "ollama" && (
                <label className="form-field" htmlFor="provider-ollama-base-url">
                  <span>Ollama base URL</span>
                  <Input
                    id="provider-ollama-base-url"
                    value={form.ollamaBaseUrl ?? ""}
                    onChange={(event) =>
                      setForm((current) => ({ ...current, ollamaBaseUrl: event.target.value }))
                    }
                    placeholder="http://localhost:11434"
                  />
                </label>
              )}

              <label className="form-field" htmlFor="provider-embedding-model">
                <span>Embedding model</span>
                <Input
                  id="provider-embedding-model"
                  value={form.embeddingModel ?? ""}
                  onChange={(event) =>
                    setForm((current) => ({ ...current, embeddingModel: event.target.value }))
                  }
                  placeholder="Reserved for search"
                />
              </label>
            </div>

            {testMessage && <p className="provider-test-message">{testMessage}</p>}

            <div className="provider-actions">
              <Button type="button" onClick={saveSettings} disabled={saving}>
                <Save aria-hidden="true" />
                Save settings
              </Button>
              <Button type="button" variant="outline" onClick={testSettings} disabled={testing}>
                <Sparkles aria-hidden="true" />
                Test provider
              </Button>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function detectCapture(rawText: string): {
  captureType: CaptureType;
  sourceKind: SourceKind;
  source: string | null;
} {
  const trimmed = rawText.trim();
  const firstUrl = trimmed.match(/https?:\/\/[^\s)]+/)?.[0] ?? null;

  if (/github\.com\/.+\/.+\/issues\/\d+/i.test(trimmed)) {
    return { captureType: "github_issue", sourceKind: "github", source: firstUrl };
  }

  if (firstUrl && trimmed === firstUrl) {
    return { captureType: "url", sourceKind: "pasted_url", source: firstUrl };
  }

  if (/^(npm|bun|pnpm|yarn|cargo|git|error:|warning:|\$ )/im.test(trimmed)) {
    return { captureType: "terminal", sourceKind: "terminal", source: null };
  }

  if (/```|function\s+\w+|const\s+\w+\s*=|class\s+\w+|import\s+.+from/.test(trimmed)) {
    return { captureType: "code", sourceKind: "code", source: null };
  }

  if (/^(user|assistant|system):/im.test(trimmed)) {
    return { captureType: "ai_chat", sourceKind: "ai_chat", source: null };
  }

  return { captureType: "note", sourceKind: "typed", source: firstUrl };
}

function providerSettingsToInput(settings: ProviderSettings): ProviderSettingsInput {
  return {
    providerType: settings.providerType,
    baseUrl: settings.baseUrl ?? "",
    apiKey: "",
    chatModel: settings.chatModel,
    embeddingModel: settings.embeddingModel ?? "",
    ollamaBaseUrl: settings.ollamaBaseUrl ?? "http://localhost:11434",
  };
}

function providerDefaults(
  providerType: ProviderType,
  current: ProviderSettingsInput,
): ProviderSettingsInput {
  if (providerType === "openrouter") {
    return {
      ...current,
      providerType,
      baseUrl: "https://openrouter.ai/api/v1",
      chatModel: current.chatModel || "openai/gpt-4.1-mini",
    };
  }

  if (providerType === "ollama") {
    return {
      ...current,
      providerType,
      chatModel: current.chatModel || "llama3.1",
      ollamaBaseUrl: current.ollamaBaseUrl || "http://localhost:11434",
    };
  }

  return {
    ...current,
    providerType,
    baseUrl: "https://api.openai.com/v1",
    chatModel: current.chatModel || "gpt-4.1-mini",
  };
}

function fallbackTitle(rawText: string) {
  const condensed = rawText.split(/\s+/).filter(Boolean).join(" ");
  return condensed.slice(0, 80) || "Untitled capture";
}

function previewText(rawText: string) {
  const condensed = rawText.split(/\s+/).filter(Boolean).join(" ");
  return condensed.slice(0, 150) || "No preview";
}

function formatAge(value: string) {
  const created = new Date(value).getTime();
  const minutes = Math.max(0, Math.round((Date.now() - created) / 60000));

  if (minutes < 1) {
    return "Just now";
  }

  if (minutes < 60) {
    return `${minutes}m ago`;
  }

  const hours = Math.round(minutes / 60);
  if (hours < 24) {
    return `${hours}h ago`;
  }

  return `${Math.round(hours / 24)}d ago`;
}

function formatDateTime(value: string) {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

function packProjectId(value: string) {
  return `project:${value}`;
}

function unpackProjectId(value: string) {
  return value === "unassigned" ? null : value.replace(/^project:/, "");
}

export default App;
