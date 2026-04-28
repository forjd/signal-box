import { invoke } from "@tauri-apps/api/core";

export type DatabaseHealth = {
  ok: boolean;
  databasePath: string | null;
  appDataDir: string | null;
  appliedMigrations: number;
  latestMigration: string | null;
  startupError: string | null;
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
  sourceType: string;
  url: string | null;
  rawReference: string | null;
};

export type CaptureDistillation = {
  capture: Capture;
  tasks: DistilledTask[];
  decisions: DistilledDecision[];
  questions: DistilledQuestion[];
  sources: DistilledSource[];
};

export function getDatabaseHealth() {
  return invoke<DatabaseHealth>("database_health");
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

export function updateCaptureStatus(id: string, status: CaptureStatus) {
  return invoke<Capture>("update_capture_status", { id, status });
}

export function updateCaptureProject(id: string, projectId: string | null) {
  return invoke<Capture>("update_capture_project", { id, projectId });
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
