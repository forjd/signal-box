import { invoke } from "@tauri-apps/api/core";

export type DatabaseHealth = {
  ok: boolean;
  databasePath: string | null;
  appDataDir: string | null;
  appliedMigrations: number;
  latestMigration: string | null;
  startupError: string | null;
};

export type BackupExport = {
  path: string;
};

export type AppMetadata = {
  productName: string;
  packageName: string;
  version: string;
  databaseFileName: string;
  quickCaptureHotkey: string;
};

export type CaptureStatus = "unprocessed" | "processed" | "archived";
export type ProcessingStatus = "idle" | "processing" | "succeeded" | "failed";
export type CaptureType = "note" | "url" | "code" | "terminal" | "ai_chat" | "github_issue";
export type SourceKind = "typed" | "pasted_url" | "code" | "terminal" | "ai_chat" | "github";
export type ProviderType = "openai" | "openrouter" | "ollama";

export type Capture = {
  id: string;
  rawText: string;
  title: string | null;
  summary: string | null;
  captureType: CaptureType;
  sourceKind: SourceKind;
  source: string | null;
  status: CaptureStatus;
  processingStatus: ProcessingStatus;
  processingError: string | null;
  projectId: string | null;
  suggestedProjectId: string | null;
  suggestedProjectName: string | null;
  createdAt: string;
  updatedAt: string;
  processedAt: string | null;
  archivedAt: string | null;
};

export type ProjectOption = {
  id: string;
  name: string;
};

export type ProjectRecord = {
  id: string;
  name: string;
  description: string | null;
  overview: string;
  currentDirection: string;
  status: string;
  createdAt: string;
  updatedAt: string;
};

export type SaveProjectInput = {
  name: string;
  description: string | null;
  overview: string | null;
  currentDirection: string | null;
};

export type CreateCaptureInput = {
  rawText: string;
  captureType: CaptureType;
  sourceKind: SourceKind;
  source: string | null;
  projectId: string | null;
};

export type ProviderSettings = {
  providerType: ProviderType;
  baseUrl: string | null;
  apiKey: string;
  hasApiKey: boolean;
  chatModel: string;
  embeddingModel: string | null;
  ollamaBaseUrl: string | null;
};

export type ProviderSettingsInput = Omit<ProviderSettings, "hasApiKey">;

export type ProviderTestResult = {
  ok: boolean;
  message: string;
};

export type DistilledTask = {
  id: string;
  title: string;
  description: string | null;
  status: string;
};

export type DistilledDecision = {
  id: string;
  title: string;
  context: string | null;
  decision: string;
  rationale: string | null;
  status: string;
};

export type DistilledQuestion = {
  id: string;
  question: string;
  answer: string | null;
  status: string;
};

export type DistilledSource = {
  id: string;
  title: string;
  kind: string;
  sourceType: string;
  url: string | null;
  rawExcerpt: string | null;
  rawReference: string | null;
  notes: string | null;
};

export type CaptureDistillation = {
  capture: Capture;
  tasks: DistilledTask[];
  decisions: DistilledDecision[];
  questions: DistilledQuestion[];
  sources: DistilledSource[];
};

export type ProjectTask = {
  id: string;
  captureId: string | null;
  captureTitle: string | null;
  title: string;
  description: string | null;
  status: string;
  createdAt: string;
  updatedAt: string;
};

export type ProjectDecision = {
  id: string;
  captureId: string | null;
  captureTitle: string | null;
  title: string;
  context: string | null;
  decision: string;
  rationale: string | null;
  status: string;
  createdAt: string;
  updatedAt: string;
};

export type ProjectQuestion = {
  id: string;
  captureId: string | null;
  captureTitle: string | null;
  question: string;
  answer: string | null;
  status: string;
  createdAt: string;
  updatedAt: string;
};

export type ProjectSource = {
  id: string;
  projectId: string | null;
  captureId: string | null;
  captureTitle: string | null;
  title: string;
  kind: string;
  url: string | null;
  rawExcerpt: string | null;
  notes: string | null;
  createdAt: string;
  updatedAt: string;
};

export type ProjectArtefact = {
  id: string;
  title: string;
  summary: string | null;
  artefactType: string;
  bodyMarkdown: string;
  model: string | null;
  provider: string | null;
  createdAt: string;
  updatedAt: string;
};

export type ProjectMemory = {
  project: ProjectRecord;
  captures: Capture[];
  tasks: ProjectTask[];
  decisions: ProjectDecision[];
  questions: ProjectQuestion[];
  sources: ProjectSource[];
  artefacts: ProjectArtefact[];
};

export type SaveTaskInput = Pick<ProjectTask, "title" | "description" | "status">;
export type SaveDecisionInput = Pick<
  ProjectDecision,
  "title" | "context" | "decision" | "rationale" | "status"
>;
export type SaveQuestionInput = Pick<ProjectQuestion, "question" | "answer" | "status">;
export type SaveSourceInput = {
  projectId: string | null;
  captureId: string | null;
  title: string;
  kind: string;
  url: string | null;
  rawExcerpt: string | null;
  notes: string | null;
};

export type ArtefactType =
  | "product_brief"
  | "implementation_plan"
  | "adr"
  | "coding_agent_prompt"
  | "linkedin_blog_draft";

export type SelectedContextItem = {
  itemType: string;
  itemId: string;
  title: string;
};

export type ArtefactContextSelection = {
  projectId: string;
  artefactType: ArtefactType;
  includeProjectMemory: boolean;
  captureIds: string[];
  decisionIds: string[];
  taskIds: string[];
  questionIds: string[];
  sourceIds: string[];
};

export type ArtefactDraft = {
  projectId: string;
  artefactType: ArtefactType;
  title: string;
  summary: string | null;
  bodyMarkdown: string;
  model: string | null;
  provider: string | null;
  context: SelectedContextItem[];
};

export type SaveArtefactInput = ArtefactDraft;

export type IndexResult = {
  indexed: number;
  skipped: number;
};

export type SearchInput = {
  query: string;
  projectId: string | null;
  limit: number | null;
};

export type SearchResult = {
  entityType: string;
  entityId: string;
  title: string;
  snippet: string;
  projectId: string | null;
  projectName: string | null;
  score: number;
  matchKind: string;
};

export type AskAnswer = {
  answerMarkdown: string;
  results: SearchResult[];
};

export function getDatabaseHealth() {
  return invoke<DatabaseHealth>("database_health");
}

export function exportDatabaseBackup() {
  return invoke<BackupExport>("export_database_backup");
}

export function getAppMetadata() {
  return invoke<AppMetadata>("app_metadata");
}

export function createCapture(input: CreateCaptureInput) {
  return invoke<Capture>("create_capture", { input });
}

export function listCaptures(status?: CaptureStatus) {
  return invoke<Capture[]>("list_captures", { status: status ?? null });
}

export function listProjects() {
  return invoke<ProjectOption[]>("list_projects");
}

export function listProjectRecords() {
  return invoke<ProjectRecord[]>("list_project_records");
}

export function createProject(input: SaveProjectInput) {
  return invoke<ProjectRecord>("create_project", { input });
}

export function updateProject(id: string, input: SaveProjectInput) {
  return invoke<ProjectRecord>("update_project", { id, input });
}

export function getProjectMemory(id: string) {
  return invoke<ProjectMemory>("get_project_memory", { id });
}

export function generateArtefact(input: ArtefactContextSelection) {
  return invoke<ArtefactDraft>("generate_artefact", { input });
}

export function saveArtefact(input: SaveArtefactInput) {
  return invoke<ProjectArtefact>("save_artefact", { input });
}

export function listArtefacts(projectId?: string | null) {
  return invoke<ProjectArtefact[]>("list_artefacts", { projectId: projectId ?? null });
}

export function getArtefact(id: string) {
  return invoke<ProjectArtefact>("get_artefact", { id });
}

export function indexSearchContext(projectId?: string | null) {
  return invoke<IndexResult>("index_search_context", { projectId: projectId ?? null });
}

export function searchContext(input: SearchInput) {
  return invoke<SearchResult[]>("search_context", { input });
}

export function askContext(question: string, projectId?: string | null) {
  return invoke<AskAnswer>("ask_context", {
    input: { question, projectId: projectId ?? null },
  });
}

export function projectRecall(projectId: string) {
  return invoke<AskAnswer>("project_recall", { projectId });
}

export function updateCaptureStatus(id: string, status: CaptureStatus) {
  return invoke<Capture>("update_capture_status", { id, status });
}

export function updateCaptureProject(id: string, projectId: string | null) {
  return invoke<Capture>("update_capture_project", { id, projectId });
}

export function acceptSuggestedProject(id: string) {
  return invoke<Capture>("accept_suggested_project", { id });
}

export function updateTask(id: string, input: SaveTaskInput) {
  return invoke<void>("update_task", { id, input });
}

export function updateDecision(id: string, input: SaveDecisionInput) {
  return invoke<void>("update_decision", { id, input });
}

export function updateQuestion(id: string, input: SaveQuestionInput) {
  return invoke<void>("update_question", { id, input });
}

export function createSource(input: SaveSourceInput) {
  return invoke<ProjectSource>("create_source", { input });
}

export function updateSource(id: string, input: SaveSourceInput) {
  return invoke<ProjectSource>("update_source", { id, input });
}

export function getProviderSettings() {
  return invoke<ProviderSettings>("get_provider_settings");
}

export function saveProviderSettings(input: ProviderSettingsInput) {
  return invoke<ProviderSettings>("save_provider_settings", { input });
}

export function testProviderSettings(input: ProviderSettingsInput) {
  return invoke<ProviderTestResult>("test_provider_settings", { input });
}

export function getCaptureDistillation(id: string) {
  return invoke<CaptureDistillation>("get_capture_distillation", { id });
}

export function processCapture(id: string) {
  return invoke<CaptureDistillation>("process_capture", { id });
}

export function showQuickCapture() {
  return invoke<void>("show_quick_capture");
}
