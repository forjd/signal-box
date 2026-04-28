import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Archive,
  CheckCircle2,
  ExternalLink,
  Inbox,
  Link2,
  FolderPlus,
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
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import {
  acceptSuggestedProject,
  askContext,
  createCapture,
  createProject,
  createSource,
  generateArtefact,
  getCaptureDistillation,
  getAppMetadata,
  getDatabaseHealth,
  getProjectMemory,
  getProviderSettings,
  indexSearchContext,
  listArtefacts,
  listCaptures,
  listProjectRecords,
  listProjects,
  processCapture,
  projectRecall,
  saveArtefact,
  saveProviderSettings,
  searchContext,
  showQuickCapture,
  testProviderSettings,
  updateDecision,
  updateCaptureProject,
  updateCaptureStatus,
  updateProject,
  updateQuestion,
  updateSource,
  updateTask,
  type AppMetadata,
  type ArtefactContextSelection,
  type ArtefactDraft,
  type ArtefactType,
  type AskAnswer,
  type Capture,
  type CaptureDistillation,
  type CaptureStatus,
  type CaptureType,
  type DatabaseHealth,
  type DistilledDecision,
  type DistilledQuestion,
  type DistilledSource,
  type DistilledTask,
  type ProjectDecision,
  type ProjectMemory,
  type ProviderSettings,
  type ProviderSettingsInput,
  type ProviderType,
  type ProjectArtefact,
  type ProjectOption,
  type ProjectQuestion,
  type ProjectRecord,
  type ProjectSource,
  type ProjectTask,
  type SaveDecisionInput,
  type SaveArtefactInput,
  type SaveProjectInput,
  type SaveQuestionInput,
  type SaveSourceInput,
  type SaveTaskInput,
  type SearchResult,
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
      title: "No project selected",
      body: "Create a project or choose one from the project memory list to review its captured context.",
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
      title: "Search local memory",
      body: "Index project context, run semantic or keyword search, and ask grounded questions.",
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

const artefactTypeLabels: Record<ArtefactType, string> = {
  product_brief: "Product brief",
  implementation_plan: "Implementation plan",
  adr: "ADR",
  coding_agent_prompt: "Coding-agent prompt",
  linkedin_blog_draft: "LinkedIn/blog draft",
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
  const [projectRecords, setProjectRecords] = useState<LoadState<ProjectRecord[]>>({
    status: "loading",
  });
  const [selectedCaptureId, setSelectedCaptureId] = useState<string | null>(null);
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);

  async function loadFoundationState() {
    try {
      const [health, appMetadata, unprocessedCaptures, projectOptions, records] = await Promise.all(
        [
          getDatabaseHealth(),
          getAppMetadata(),
          listCaptures("unprocessed"),
          listProjects(),
          listProjectRecords(),
        ],
      );

      setDatabaseHealth({ status: "ready", data: health });
      setMetadata({ status: "ready", data: appMetadata });
      setCaptures({ status: "ready", data: unprocessedCaptures });
      setProjects({ status: "ready", data: projectOptions });
      setProjectRecords({ status: "ready", data: records });
      setSelectedCaptureId((current) => current ?? unprocessedCaptures[0]?.id ?? null);
      setSelectedProjectId((current) => current ?? records[0]?.id ?? null);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setDatabaseHealth({ status: "error", message });
      setMetadata({ status: "error", message });
      setCaptures({ status: "error", message });
      setProjects({ status: "error", message });
      setProjectRecords({ status: "error", message });
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

  async function refreshProjects() {
    const [projectOptions, records] = await Promise.all([listProjects(), listProjectRecords()]);
    setProjects({ status: "ready", data: projectOptions });
    setProjectRecords({ status: "ready", data: records });
    setSelectedProjectId((current) => {
      if (current && records.some((project) => project.id === current)) {
        return current;
      }

      return records[0]?.id ?? null;
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
      await refreshProjects();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function acceptCaptureProject(id: string) {
    try {
      const updated = await acceptSuggestedProject(id);
      replaceCapture(updated);
      await refreshProjects();
      toast.success("Suggested project accepted");
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
        {activeRoute === "projects" && (
          <ProjectsView
            state={projectRecords}
            selectedProjectId={selectedProjectId}
            onRefresh={refreshProjects}
            onSelect={(id) => {
              setSelectedProjectId(id);
              setActiveRoute("memory");
            }}
          />
        )}
        {activeRoute === "memory" && (
          <ProjectMemoryView
            projects={projectRecords}
            selectedProjectId={selectedProjectId}
            onRefreshProjects={refreshProjects}
            onSelectProject={setSelectedProjectId}
          />
        )}
        {activeRoute === "artefacts" && (
          <ArtefactsView
            projects={projectRecords}
            selectedProjectId={selectedProjectId}
            onSelectProject={setSelectedProjectId}
          />
        )}
        {activeRoute === "search" && (
          <SearchView
            projects={projectRecords}
            selectedProjectId={selectedProjectId}
            onSelectProject={setSelectedProjectId}
            onOpenProjectMemory={(projectId) => {
              setSelectedProjectId(projectId);
              setActiveRoute("memory");
            }}
          />
        )}
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
            onAcceptSuggestion={acceptCaptureProject}
            onCaptureChange={replaceCapture}
          />
        )}
        {activeRoute !== "settings" &&
          activeRoute !== "inbox" &&
          activeRoute !== "projects" &&
          activeRoute !== "memory" &&
          activeRoute !== "artefacts" &&
          activeRoute !== "search" && (
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
  onAcceptSuggestion,
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
  onAcceptSuggestion: (id: string) => void;
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
          onAcceptSuggestion={onAcceptSuggestion}
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
  onAcceptSuggestion,
  onCaptureChange,
}: {
  capture: Capture;
  projects: LoadState<ProjectOption[]>;
  onArchive: (id: string) => void;
  onStatusChange: (id: string, status: CaptureStatus) => void;
  onProjectChange: (id: string, projectId: string | null) => void;
  onAcceptSuggestion: (id: string) => void;
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

      {capture.suggestedProjectId && capture.projectId !== capture.suggestedProjectId && (
        <div className="suggestion-panel">
          <div>
            <p className="eyebrow">Suggested project</p>
            <strong>{capture.suggestedProjectName ?? "Matched project"}</strong>
          </div>
          <Button type="button" variant="outline" onClick={() => onAcceptSuggestion(capture.id)}>
            <Link2 aria-hidden="true" />
            Accept
          </Button>
        </div>
      )}

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

function ProjectsView({
  state,
  selectedProjectId,
  onRefresh,
  onSelect,
}: {
  state: LoadState<ProjectRecord[]>;
  selectedProjectId: string | null;
  onRefresh: () => void;
  onSelect: (id: string) => void;
}) {
  const [creating, setCreating] = useState(false);

  if (state.status === "loading") {
    return <LoadingState label="Loading projects" />;
  }

  if (state.status === "error") {
    return <ErrorState title="Projects unavailable" message={state.message} />;
  }

  return (
    <div className="project-directory">
      <section className="project-panel">
        <div className="list-toolbar">
          <div>
            <p className="eyebrow">Projects</p>
            <h2>{state.data.length} active</h2>
          </div>
          <div className="detail-actions">
            <Button type="button" variant="outline" size="sm" onClick={onRefresh}>
              <RefreshCw aria-hidden="true" />
              Refresh
            </Button>
            <Button type="button" size="sm" onClick={() => setCreating((current) => !current)}>
              <FolderPlus aria-hidden="true" />
              New
            </Button>
          </div>
        </div>
        {creating && (
          <ProjectForm
            onSaved={async () => {
              setCreating(false);
              await onRefresh();
            }}
          />
        )}
        {state.data.length === 0 ? (
          <div className="panel-empty">
            <EmptyState
              kicker="Project memory"
              title="No projects yet"
              body="Create a project before attaching captures and extracted objects."
            />
          </div>
        ) : (
          <div className="project-list">
            {state.data.map((project) => (
              <button
                className="project-row"
                data-active={project.id === selectedProjectId}
                key={project.id}
                onClick={() => onSelect(project.id)}
                type="button"
              >
                <strong>{project.name}</strong>
                <span>{project.description || project.overview || "No overview yet"}</span>
                <small>Updated {formatAge(project.updatedAt)}</small>
              </button>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}

function ProjectForm({
  project,
  onSaved,
}: {
  project?: ProjectRecord;
  onSaved: (project: ProjectRecord) => void | Promise<void>;
}) {
  const [form, setForm] = useState<SaveProjectInput>({
    name: project?.name ?? "",
    description: project?.description ?? "",
    overview: project?.overview ?? "",
    currentDirection: project?.currentDirection ?? "",
  });
  const [saving, setSaving] = useState(false);

  async function save() {
    setSaving(true);
    try {
      const saved = project ? await updateProject(project.id, form) : await createProject(form);
      toast.success(project ? "Project updated" : "Project created");
      await onSaved(saved);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="project-form">
      <label className="form-field" htmlFor={`project-name-${project?.id ?? "new"}`}>
        <span>Name</span>
        <Input
          id={`project-name-${project?.id ?? "new"}`}
          value={form.name}
          onChange={(event) => setForm((current) => ({ ...current, name: event.target.value }))}
        />
      </label>
      <label className="form-field" htmlFor={`project-description-${project?.id ?? "new"}`}>
        <span>Description</span>
        <Input
          id={`project-description-${project?.id ?? "new"}`}
          value={form.description ?? ""}
          onChange={(event) =>
            setForm((current) => ({ ...current, description: event.target.value }))
          }
        />
      </label>
      <label
        className="form-field wide-field"
        htmlFor={`project-direction-${project?.id ?? "new"}`}
      >
        <span>Current direction</span>
        <Textarea
          id={`project-direction-${project?.id ?? "new"}`}
          value={form.currentDirection ?? ""}
          onChange={(event) =>
            setForm((current) => ({ ...current, currentDirection: event.target.value }))
          }
        />
      </label>
      <label className="form-field wide-field" htmlFor={`project-overview-${project?.id ?? "new"}`}>
        <span>Overview</span>
        <Textarea
          id={`project-overview-${project?.id ?? "new"}`}
          value={form.overview ?? ""}
          onChange={(event) => setForm((current) => ({ ...current, overview: event.target.value }))}
        />
      </label>
      <div className="form-actions">
        <Button type="button" onClick={save} disabled={saving || !form.name.trim()}>
          <Save aria-hidden="true" />
          Save
        </Button>
      </div>
    </div>
  );
}

function ProjectMemoryView({
  projects,
  selectedProjectId,
  onSelectProject,
  onRefreshProjects,
}: {
  projects: LoadState<ProjectRecord[]>;
  selectedProjectId: string | null;
  onSelectProject: (id: string) => void;
  onRefreshProjects: () => void;
}) {
  const [memory, setMemory] = useState<LoadState<ProjectMemory>>({ status: "loading" });

  const projectOptions = projects.status === "ready" ? projects.data : [];
  const activeProjectId = selectedProjectId ?? projectOptions[0]?.id ?? null;

  const loadMemory = useCallback(async () => {
    if (!activeProjectId) {
      setMemory({ status: "ready", data: emptyProjectMemory() });
      return;
    }

    setMemory({ status: "loading" });
    try {
      setMemory({ status: "ready", data: await getProjectMemory(activeProjectId) });
    } catch (error) {
      setMemory({
        status: "error",
        message: error instanceof Error ? error.message : String(error),
      });
    }
  }, [activeProjectId]);

  useEffect(() => {
    loadMemory();
  }, [loadMemory]);

  if (projects.status === "loading" || memory.status === "loading") {
    return <LoadingState label="Loading project memory" />;
  }

  if (projects.status === "error") {
    return <ErrorState title="Projects unavailable" message={projects.message} />;
  }

  if (memory.status === "error") {
    return <ErrorState title="Project memory unavailable" message={memory.message} />;
  }

  if (!activeProjectId || memory.data.project.id === "") {
    return (
      <EmptyState title="No project selected" body="Create a project before building memory." />
    );
  }

  const data = memory.data;

  async function reload() {
    await Promise.all([loadMemory(), onRefreshProjects()]);
  }

  return (
    <div className="project-memory-layout">
      <aside className="project-memory-list" aria-label="Project memory list">
        {projectOptions.map((project) => (
          <button
            className="project-row"
            data-active={project.id === activeProjectId}
            key={project.id}
            onClick={() => onSelectProject(project.id)}
            type="button"
          >
            <strong>{project.name}</strong>
            <span>{project.description || "No description"}</span>
          </button>
        ))}
      </aside>

      <section className="project-memory-panel">
        <header className="detail-header">
          <div>
            <p className="eyebrow">Project memory</p>
            <h2>{data.project.name}</h2>
          </div>
          <Button type="button" variant="outline" onClick={reload}>
            <RefreshCw aria-hidden="true" />
            Refresh
          </Button>
        </header>

        <Tabs defaultValue="overview" className="memory-tabs">
          <TabsList>
            <TabsTrigger value="overview">Overview</TabsTrigger>
            <TabsTrigger value="captures">Captures</TabsTrigger>
            <TabsTrigger value="decisions">Decisions</TabsTrigger>
            <TabsTrigger value="tasks">Tasks</TabsTrigger>
            <TabsTrigger value="sources">Sources</TabsTrigger>
            <TabsTrigger value="artefacts">Artefacts</TabsTrigger>
            <TabsTrigger value="questions">Questions</TabsTrigger>
          </TabsList>
          <TabsContent value="overview">
            <ProjectForm project={data.project} onSaved={reload} />
          </TabsContent>
          <TabsContent value="captures">
            <ProjectCaptures captures={data.captures} />
          </TabsContent>
          <TabsContent value="decisions">
            <DecisionList items={data.decisions} onSaved={reload} />
          </TabsContent>
          <TabsContent value="tasks">
            <TaskList items={data.tasks} onSaved={reload} />
          </TabsContent>
          <TabsContent value="sources">
            <SourceList
              projectId={data.project.id}
              captures={data.captures}
              items={data.sources}
              onSaved={reload}
            />
          </TabsContent>
          <TabsContent value="artefacts">
            <ArtefactList items={data.artefacts} />
          </TabsContent>
          <TabsContent value="questions">
            <QuestionList items={data.questions} onSaved={reload} />
          </TabsContent>
        </Tabs>
      </section>
    </div>
  );
}

function emptyProjectMemory(): ProjectMemory {
  return {
    project: {
      id: "",
      name: "",
      description: null,
      overview: "",
      currentDirection: "",
      status: "active",
      createdAt: "",
      updatedAt: "",
    },
    captures: [],
    tasks: [],
    decisions: [],
    questions: [],
    sources: [],
    artefacts: [],
  };
}

function ProjectCaptures({ captures }: { captures: Capture[] }) {
  if (captures.length === 0) {
    return <p className="muted-copy">No captures are attached to this project.</p>;
  }

  return (
    <div className="structured-list">
      {captures.map((capture) => (
        <article className="object-item" key={capture.id}>
          <strong>{capture.title || fallbackTitle(capture.rawText)}</strong>
          {capture.summary && <p>{capture.summary}</p>}
          <small>{capture.rawText}</small>
        </article>
      ))}
    </div>
  );
}

function TaskList({ items, onSaved }: { items: ProjectTask[]; onSaved: () => void }) {
  return (
    <StructuredList empty="No tasks have been attached to this project.">
      {items.map((item) => (
        <TaskEditor key={item.id} item={item} onSaved={onSaved} />
      ))}
    </StructuredList>
  );
}

function TaskEditor({ item, onSaved }: { item: ProjectTask; onSaved: () => void }) {
  const [form, setForm] = useState<SaveTaskInput>({
    title: item.title,
    description: item.description,
    status: item.status,
  });
  return (
    <EditableObject
      provenance={item.captureTitle}
      onSave={async () => {
        await updateTask(item.id, form);
        toast.success("Task updated");
        onSaved();
      }}
    >
      <Input
        value={form.title}
        onChange={(event) => setForm({ ...form, title: event.target.value })}
      />
      <Textarea
        value={form.description ?? ""}
        onChange={(event) => setForm({ ...form, description: event.target.value })}
      />
      <Input
        value={form.status}
        onChange={(event) => setForm({ ...form, status: event.target.value })}
      />
    </EditableObject>
  );
}

function DecisionList({ items, onSaved }: { items: ProjectDecision[]; onSaved: () => void }) {
  return (
    <StructuredList empty="No decisions have been attached to this project.">
      {items.map((item) => (
        <DecisionEditor key={item.id} item={item} onSaved={onSaved} />
      ))}
    </StructuredList>
  );
}

function DecisionEditor({ item, onSaved }: { item: ProjectDecision; onSaved: () => void }) {
  const [form, setForm] = useState<SaveDecisionInput>({
    title: item.title,
    context: item.context,
    decision: item.decision,
    rationale: item.rationale,
    status: item.status,
  });
  return (
    <EditableObject
      provenance={item.captureTitle}
      onSave={async () => {
        await updateDecision(item.id, form);
        toast.success("Decision updated");
        onSaved();
      }}
    >
      <Input
        value={form.title}
        onChange={(event) => setForm({ ...form, title: event.target.value })}
      />
      <Textarea
        value={form.context ?? ""}
        onChange={(event) => setForm({ ...form, context: event.target.value })}
      />
      <Textarea
        value={form.decision}
        onChange={(event) => setForm({ ...form, decision: event.target.value })}
      />
      <Textarea
        value={form.rationale ?? ""}
        onChange={(event) => setForm({ ...form, rationale: event.target.value })}
      />
      <Input
        value={form.status}
        onChange={(event) => setForm({ ...form, status: event.target.value })}
      />
    </EditableObject>
  );
}

function QuestionList({ items, onSaved }: { items: ProjectQuestion[]; onSaved: () => void }) {
  return (
    <StructuredList empty="No questions have been attached to this project.">
      {items.map((item) => (
        <QuestionEditor key={item.id} item={item} onSaved={onSaved} />
      ))}
    </StructuredList>
  );
}

function QuestionEditor({ item, onSaved }: { item: ProjectQuestion; onSaved: () => void }) {
  const [form, setForm] = useState<SaveQuestionInput>({
    question: item.question,
    answer: item.answer,
    status: item.status,
  });
  return (
    <EditableObject
      provenance={item.captureTitle}
      onSave={async () => {
        await updateQuestion(item.id, form);
        toast.success("Question updated");
        onSaved();
      }}
    >
      <Textarea
        value={form.question}
        onChange={(event) => setForm({ ...form, question: event.target.value })}
      />
      <Textarea
        value={form.answer ?? ""}
        onChange={(event) => setForm({ ...form, answer: event.target.value })}
      />
      <Input
        value={form.status}
        onChange={(event) => setForm({ ...form, status: event.target.value })}
      />
    </EditableObject>
  );
}

function SourceList({
  projectId,
  captures,
  items,
  onSaved,
}: {
  projectId: string;
  captures: Capture[];
  items: ProjectSource[];
  onSaved: () => void;
}) {
  const [form, setForm] = useState<SaveSourceInput>({
    projectId,
    captureId: null,
    title: "",
    kind: "text",
    url: "",
    rawExcerpt: "",
    notes: "",
  });

  async function saveSource() {
    await createSource(form);
    setForm({
      projectId,
      captureId: null,
      title: "",
      kind: "text",
      url: "",
      rawExcerpt: "",
      notes: "",
    });
    toast.success("Source created");
    onSaved();
  }

  return (
    <div className="source-tab">
      <div className="project-form">
        <Input
          value={form.title}
          onChange={(event) => setForm({ ...form, title: event.target.value })}
          placeholder="Source title"
        />
        <Input
          value={form.kind}
          onChange={(event) => setForm({ ...form, kind: event.target.value })}
          placeholder="url, docs, repo, message, terminal, text"
        />
        <Input
          value={form.url ?? ""}
          onChange={(event) => setForm({ ...form, url: event.target.value })}
          placeholder="https://..."
        />
        <ProjectCaptureSelect
          captures={captures}
          value={form.captureId}
          onChange={(captureId) => setForm({ ...form, captureId })}
        />
        <Textarea
          className="wide-field"
          value={form.rawExcerpt ?? ""}
          onChange={(event) => setForm({ ...form, rawExcerpt: event.target.value })}
          placeholder="Raw excerpt"
        />
        <Textarea
          className="wide-field"
          value={form.notes ?? ""}
          onChange={(event) => setForm({ ...form, notes: event.target.value })}
          placeholder="Notes"
        />
        <div className="form-actions">
          <Button type="button" onClick={saveSource} disabled={!form.title.trim()}>
            <Save aria-hidden="true" />
            Add source
          </Button>
        </div>
      </div>
      <StructuredList empty="No sources have been attached to this project.">
        {items.map((item) => (
          <SourceEditor key={item.id} item={item} captures={captures} onSaved={onSaved} />
        ))}
      </StructuredList>
    </div>
  );
}

function SourceEditor({
  item,
  captures,
  onSaved,
}: {
  item: ProjectSource;
  captures: Capture[];
  onSaved: () => void;
}) {
  const [form, setForm] = useState<SaveSourceInput>({
    projectId: item.projectId,
    captureId: item.captureId,
    title: item.title,
    kind: item.kind,
    url: item.url,
    rawExcerpt: item.rawExcerpt,
    notes: item.notes,
  });
  return (
    <EditableObject
      provenance={item.captureTitle}
      onSave={async () => {
        await updateSource(item.id, form);
        toast.success("Source updated");
        onSaved();
      }}
    >
      <Input
        value={form.title}
        onChange={(event) => setForm({ ...form, title: event.target.value })}
      />
      <Input
        value={form.kind}
        onChange={(event) => setForm({ ...form, kind: event.target.value })}
      />
      <Input
        value={form.url ?? ""}
        onChange={(event) => setForm({ ...form, url: event.target.value })}
      />
      <ProjectCaptureSelect
        captures={captures}
        value={form.captureId}
        onChange={(captureId) => setForm({ ...form, captureId })}
      />
      <Textarea
        value={form.rawExcerpt ?? ""}
        onChange={(event) => setForm({ ...form, rawExcerpt: event.target.value })}
      />
      <Textarea
        value={form.notes ?? ""}
        onChange={(event) => setForm({ ...form, notes: event.target.value })}
      />
    </EditableObject>
  );
}

function ProjectCaptureSelect({
  captures,
  value,
  onChange,
}: {
  captures: Capture[];
  value: string | null;
  onChange: (value: string | null) => void;
}) {
  return (
    <Select
      value={value ? packProjectId(value) : "none"}
      onValueChange={(next) => onChange(unpackProjectId(next))}
    >
      <SelectTrigger>
        <SelectValue placeholder="Linked capture" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="none">No capture link</SelectItem>
        {captures.map((capture) => (
          <SelectItem key={capture.id} value={packProjectId(capture.id)}>
            {capture.title || fallbackTitle(capture.rawText)}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}

function ArtefactList({ items }: { items: ProjectArtefact[] }) {
  return (
    <StructuredList empty="No artefacts have been generated for this project yet.">
      {items.map((item) => (
        <article className="object-item" key={item.id}>
          <strong>{item.title}</strong>
          <Badge variant="outline">{item.artefactType}</Badge>
          {item.summary && <p>{item.summary}</p>}
          <small>Updated {formatAge(item.updatedAt)}</small>
        </article>
      ))}
    </StructuredList>
  );
}

function ArtefactsView({
  projects,
  selectedProjectId,
  onSelectProject,
}: {
  projects: LoadState<ProjectRecord[]>;
  selectedProjectId: string | null;
  onSelectProject: (id: string) => void;
}) {
  const [memory, setMemory] = useState<LoadState<ProjectMemory>>({ status: "loading" });
  const [artefacts, setArtefacts] = useState<LoadState<ProjectArtefact[]>>({ status: "loading" });
  const [selectedArtefactId, setSelectedArtefactId] = useState<string | null>(null);
  const [draft, setDraft] = useState<ArtefactDraft | null>(null);
  const [selection, setSelection] = useState<ArtefactContextSelection | null>(null);
  const [generating, setGenerating] = useState(false);
  const [saving, setSaving] = useState(false);

  const projectOptions = projects.status === "ready" ? projects.data : [];
  const activeProjectId = selectedProjectId ?? projectOptions[0]?.id ?? null;

  const loadArtefactState = useCallback(async () => {
    if (!activeProjectId) {
      setMemory({ status: "ready", data: emptyProjectMemory() });
      setArtefacts({ status: "ready", data: [] });
      return;
    }

    setMemory({ status: "loading" });
    setArtefacts({ status: "loading" });
    try {
      const [loadedMemory, loadedArtefacts] = await Promise.all([
        getProjectMemory(activeProjectId),
        listArtefacts(activeProjectId),
      ]);
      setMemory({ status: "ready", data: loadedMemory });
      setArtefacts({ status: "ready", data: loadedArtefacts });
      setSelection((current) => current ?? defaultArtefactSelection(activeProjectId));
      setSelectedArtefactId((current) => current ?? loadedArtefacts[0]?.id ?? null);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setMemory({ status: "error", message });
      setArtefacts({ status: "error", message });
    }
  }, [activeProjectId]);

  useEffect(() => {
    loadArtefactState();
  }, [loadArtefactState]);

  useEffect(() => {
    if (activeProjectId) {
      setSelection(defaultArtefactSelection(activeProjectId));
      setDraft(null);
    }
  }, [activeProjectId]);

  if (
    projects.status === "loading" ||
    memory.status === "loading" ||
    artefacts.status === "loading"
  ) {
    return <LoadingState label="Loading artefact generator" />;
  }

  if (projects.status === "error") {
    return <ErrorState title="Projects unavailable" message={projects.message} />;
  }

  if (memory.status === "error") {
    return <ErrorState title="Project context unavailable" message={memory.message} />;
  }

  if (artefacts.status === "error") {
    return <ErrorState title="Artefacts unavailable" message={artefacts.message} />;
  }

  if (!activeProjectId || memory.data.project.id === "") {
    return (
      <EmptyState
        title="No project selected"
        body="Create a project before generating artefacts."
      />
    );
  }

  const activeSelection = selection ?? defaultArtefactSelection(activeProjectId);
  const selectedArtefact =
    artefacts.data.find((artefact) => artefact.id === selectedArtefactId) ?? artefacts.data[0];

  async function generate() {
    setGenerating(true);
    try {
      const generated = await generateArtefact(activeSelection);
      setDraft(generated);
      toast.success("Artefact generated");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setGenerating(false);
    }
  }

  async function saveDraft() {
    if (!draft) {
      return;
    }

    setSaving(true);
    try {
      const input: SaveArtefactInput = draft;
      const saved = await saveArtefact(input);
      toast.success("Artefact saved");
      setSelectedArtefactId(saved.id);
      setDraft(null);
      await loadArtefactState();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="artefact-workspace">
      <section className="artefact-generator">
        <div className="list-toolbar">
          <div>
            <p className="eyebrow">Generate</p>
            <h2>{memory.data.project.name}</h2>
          </div>
          <ProjectRecordSelect
            projects={projectOptions}
            value={activeProjectId}
            onChange={onSelectProject}
          />
        </div>

        <div className="artefact-controls">
          <div className="form-field">
            <span>Type</span>
            <Select
              value={activeSelection.artefactType}
              onValueChange={(value) =>
                setSelection({ ...activeSelection, artefactType: value as ArtefactType })
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {Object.entries(artefactTypeLabels).map(([type, label]) => (
                  <SelectItem key={type} value={type}>
                    {label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <label className="context-toggle">
            <input
              type="checkbox"
              checked={activeSelection.includeProjectMemory}
              onChange={(event) =>
                setSelection({
                  ...activeSelection,
                  includeProjectMemory: event.target.checked,
                })
              }
            />
            Project memory
          </label>
        </div>

        <ContextSelector memory={memory.data} selection={activeSelection} onChange={setSelection} />

        <div className="provider-actions">
          <Button type="button" onClick={generate} disabled={generating}>
            <Sparkles aria-hidden="true" />
            {generating ? "Generating" : "Generate"}
          </Button>
          <Button type="button" variant="outline" onClick={() => setDraft(null)} disabled={!draft}>
            Clear preview
          </Button>
        </div>

        {draft && (
          <section className="markdown-preview" aria-label="Generated artefact preview">
            <div className="distillation-header">
              <div>
                <p className="eyebrow">Preview</p>
                <h3>{draft.title}</h3>
              </div>
              <div className="provider-actions">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => copyMarkdown(draft.bodyMarkdown)}
                >
                  Copy
                </Button>
                <Button type="button" onClick={saveDraft} disabled={saving}>
                  <Save aria-hidden="true" />
                  Save
                </Button>
              </div>
            </div>
            <MarkdownBlock markdown={draft.bodyMarkdown} />
          </section>
        )}
      </section>

      <section className="artefact-library">
        <div className="list-toolbar">
          <div>
            <p className="eyebrow">Saved</p>
            <h2>{artefacts.data.length} artefacts</h2>
          </div>
          <Button type="button" variant="outline" size="sm" onClick={loadArtefactState}>
            <RefreshCw aria-hidden="true" />
            Refresh
          </Button>
        </div>
        <div className="artefact-library-body">
          <div className="project-list">
            {artefacts.data.map((artefact) => (
              <button
                className="project-row"
                data-active={artefact.id === selectedArtefact?.id}
                key={artefact.id}
                onClick={() => setSelectedArtefactId(artefact.id)}
                type="button"
              >
                <strong>{artefact.title}</strong>
                <span>{artefactTypeLabels[artefact.artefactType as ArtefactType]}</span>
                <small>{formatAge(artefact.updatedAt)}</small>
              </button>
            ))}
            {artefacts.data.length === 0 && (
              <p className="muted-copy">No saved artefacts for this project.</p>
            )}
          </div>
          {selectedArtefact && (
            <article className="saved-artefact">
              <div className="distillation-header">
                <div>
                  <p className="eyebrow">{selectedArtefact.provider ?? "Saved"}</p>
                  <h3>{selectedArtefact.title}</h3>
                </div>
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => copyMarkdown(selectedArtefact.bodyMarkdown)}
                >
                  Copy
                </Button>
              </div>
              <MarkdownBlock markdown={selectedArtefact.bodyMarkdown} />
            </article>
          )}
        </div>
      </section>
    </div>
  );
}

function defaultArtefactSelection(projectId: string): ArtefactContextSelection {
  return {
    projectId,
    artefactType: "implementation_plan",
    includeProjectMemory: true,
    captureIds: [],
    decisionIds: [],
    taskIds: [],
    questionIds: [],
    sourceIds: [],
  };
}

function ProjectRecordSelect({
  projects,
  value,
  onChange,
}: {
  projects: ProjectRecord[];
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <Select
      value={packProjectId(value)}
      onValueChange={(next) => onChange(unpackProjectId(next) ?? value)}
    >
      <SelectTrigger>
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {projects.map((project) => (
          <SelectItem key={project.id} value={packProjectId(project.id)}>
            {project.name}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}

function ContextSelector({
  memory,
  selection,
  onChange,
}: {
  memory: ProjectMemory;
  selection: ArtefactContextSelection;
  onChange: (selection: ArtefactContextSelection) => void;
}) {
  return (
    <div className="context-selector">
      <ContextGroup
        title="Captures"
        ids={selection.captureIds}
        items={memory.captures.map((capture) => ({
          id: capture.id,
          title: capture.title || fallbackTitle(capture.rawText),
        }))}
        onChange={(captureIds) => onChange({ ...selection, captureIds })}
      />
      <ContextGroup
        title="Decisions"
        ids={selection.decisionIds}
        items={memory.decisions.map((decision) => ({ id: decision.id, title: decision.title }))}
        onChange={(decisionIds) => onChange({ ...selection, decisionIds })}
      />
      <ContextGroup
        title="Tasks"
        ids={selection.taskIds}
        items={memory.tasks.map((task) => ({ id: task.id, title: task.title }))}
        onChange={(taskIds) => onChange({ ...selection, taskIds })}
      />
      <ContextGroup
        title="Questions"
        ids={selection.questionIds}
        items={memory.questions.map((question) => ({ id: question.id, title: question.question }))}
        onChange={(questionIds) => onChange({ ...selection, questionIds })}
      />
      <ContextGroup
        title="Sources"
        ids={selection.sourceIds}
        items={memory.sources.map((source) => ({ id: source.id, title: source.title }))}
        onChange={(sourceIds) => onChange({ ...selection, sourceIds })}
      />
    </div>
  );
}

function ContextGroup({
  title,
  items,
  ids,
  onChange,
}: {
  title: string;
  items: Array<{ id: string; title: string }>;
  ids: string[];
  onChange: (ids: string[]) => void;
}) {
  return (
    <section className="context-group">
      <div className="distillation-header">
        <h4>{title}</h4>
        <Badge variant="outline">{ids.length}</Badge>
      </div>
      {items.length === 0 ? (
        <p className="muted-copy">None</p>
      ) : (
        items.map((item) => (
          <label className="context-option" key={item.id}>
            <input
              type="checkbox"
              checked={ids.includes(item.id)}
              onChange={(event) => {
                onChange(
                  event.target.checked
                    ? [...ids, item.id]
                    : ids.filter((existing) => existing !== item.id),
                );
              }}
            />
            <span>{item.title}</span>
          </label>
        ))
      )}
    </section>
  );
}

function MarkdownBlock({ markdown }: { markdown: string }) {
  return (
    <div className="markdown-body">
      {markdown.split("\n").map((line, index) => {
        if (line.startsWith("# ")) {
          return <h2 key={index}>{line.replace(/^# /, "")}</h2>;
        }
        if (line.startsWith("## ")) {
          return <h3 key={index}>{line.replace(/^## /, "")}</h3>;
        }
        if (line.startsWith("- ")) {
          return <p key={index}>• {line.replace(/^- /, "")}</p>;
        }
        return line.trim() ? <p key={index}>{line}</p> : <br key={index} />;
      })}
    </div>
  );
}

async function copyMarkdown(markdown: string) {
  try {
    await navigator.clipboard.writeText(markdown);
    toast.success("Markdown copied");
  } catch (error) {
    toast.error(error instanceof Error ? error.message : String(error));
  }
}

function SearchView({
  projects,
  selectedProjectId,
  onSelectProject,
  onOpenProjectMemory,
}: {
  projects: LoadState<ProjectRecord[]>;
  selectedProjectId: string | null;
  onSelectProject: (id: string) => void;
  onOpenProjectMemory: (id: string) => void;
}) {
  const [query, setQuery] = useState("");
  const [question, setQuestion] = useState("");
  const [results, setResults] = useState<LoadState<SearchResult[]>>({ status: "ready", data: [] });
  const [answer, setAnswer] = useState<LoadState<AskAnswer | null>>({
    status: "ready",
    data: null,
  });
  const [indexing, setIndexing] = useState(false);
  const [searching, setSearching] = useState(false);
  const [asking, setAsking] = useState(false);

  if (projects.status === "loading") {
    return <LoadingState label="Loading search projects" />;
  }

  if (projects.status === "error") {
    return <ErrorState title="Projects unavailable" message={projects.message} />;
  }

  const activeProjectId = selectedProjectId ?? projects.data[0]?.id ?? null;

  async function runIndex() {
    setIndexing(true);
    try {
      const result = await indexSearchContext(activeProjectId);
      toast.success(`Indexed ${result.indexed}, skipped ${result.skipped}`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setIndexing(false);
    }
  }

  async function runSearch() {
    setSearching(true);
    setResults({ status: "loading" });
    try {
      const found = await searchContext({ query, projectId: activeProjectId, limit: 12 });
      setResults({ status: "ready", data: found });
    } catch (error) {
      setResults({
        status: "error",
        message: error instanceof Error ? error.message : String(error),
      });
    } finally {
      setSearching(false);
    }
  }

  async function runAsk(recall = false) {
    if (!activeProjectId && recall) {
      toast.error("Choose a project for recall");
      return;
    }

    setAsking(true);
    setAnswer({ status: "loading" });
    try {
      const response = recall
        ? await projectRecall(activeProjectId as string)
        : await askContext(question, activeProjectId);
      setAnswer({ status: "ready", data: response });
      setResults({ status: "ready", data: response.results });
    } catch (error) {
      setAnswer({
        status: "error",
        message: error instanceof Error ? error.message : String(error),
      });
    } finally {
      setAsking(false);
    }
  }

  return (
    <div className="search-workspace">
      <section className="search-panel">
        <div className="list-toolbar">
          <div>
            <p className="eyebrow">Search</p>
            <h2>Local context</h2>
          </div>
          {activeProjectId && (
            <ProjectRecordSelect
              projects={projects.data}
              value={activeProjectId}
              onChange={onSelectProject}
            />
          )}
        </div>

        <div className="search-controls">
          <Input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search decisions, captures, sources, artefacts..."
          />
          <Button type="button" onClick={runSearch} disabled={searching || !query.trim()}>
            Search
          </Button>
          <Button type="button" variant="outline" onClick={runIndex} disabled={indexing}>
            {indexing ? "Indexing" : "Index"}
          </Button>
        </div>

        <div className="search-controls ask-controls">
          <Input
            value={question}
            onChange={(event) => setQuestion(event.target.value)}
            placeholder="What did I decide about sync?"
          />
          <Button type="button" onClick={() => runAsk(false)} disabled={asking || !question.trim()}>
            Ask
          </Button>
          <Button type="button" variant="outline" onClick={() => runAsk(true)} disabled={asking}>
            Where did I get to?
          </Button>
        </div>

        {answer.status === "loading" && (
          <LoadingState label="Answering from local context" compact />
        )}
        {answer.status === "error" && (
          <ErrorState title="Ask failed" message={answer.message} compact />
        )}
        {answer.status === "ready" && answer.data && (
          <section className="answer-panel">
            <MarkdownBlock markdown={answer.data.answerMarkdown} />
          </section>
        )}
      </section>

      <section className="search-panel">
        <div className="list-toolbar">
          <div>
            <p className="eyebrow">Results</p>
            <h2>{results.status === "ready" ? results.data.length : 0} matches</h2>
          </div>
        </div>
        {results.status === "loading" && <LoadingState label="Searching local memory" />}
        {results.status === "error" && (
          <ErrorState title="Search failed" message={results.message} />
        )}
        {results.status === "ready" && (
          <div className="structured-list search-results">
            {results.data.length === 0 && <p className="muted-copy">No results yet.</p>}
            {results.data.map((result) => (
              <article className="object-item" key={`${result.entityType}:${result.entityId}`}>
                <div className="distillation-header">
                  <strong>{result.title}</strong>
                  <Badge variant="outline">{result.matchKind}</Badge>
                </div>
                <p>{result.snippet}</p>
                <small>
                  {result.entityType}:{result.entityId}
                  {result.projectName ? ` · ${result.projectName}` : ""}
                </small>
                {result.projectId && (
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => onOpenProjectMemory(result.projectId as string)}
                  >
                    Open project memory
                  </Button>
                )}
              </article>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}

function StructuredList({ children, empty }: { children: ReactNode; empty: string }) {
  const items = Array.isArray(children) ? children.filter(Boolean) : children;
  if (Array.isArray(items) && items.length === 0) {
    return <p className="muted-copy">{empty}</p>;
  }

  return <div className="structured-list">{children}</div>;
}

function EditableObject({
  children,
  provenance,
  onSave,
}: {
  children: ReactNode;
  provenance: string | null;
  onSave: () => Promise<void>;
}) {
  const [saving, setSaving] = useState(false);

  async function save() {
    setSaving(true);
    try {
      await onSave();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  return (
    <article className="editable-object">
      {provenance && <Badge variant="outline">From {provenance}</Badge>}
      <div className="editable-fields">{children}</div>
      <Button type="button" size="sm" onClick={save} disabled={saving}>
        <Save aria-hidden="true" />
        Save
      </Button>
    </article>
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
  return value === "unassigned" || value === "none" ? null : value.replace(/^project:/, "");
}

export default App;
