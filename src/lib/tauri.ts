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
export type CaptureType = "note" | "url" | "code" | "terminal" | "ai_chat" | "github_issue";
export type SourceKind = "typed" | "pasted_url" | "code" | "terminal" | "ai_chat" | "github";

export type Capture = {
  id: string;
  rawText: string;
  title: string | null;
  summary: string | null;
  captureType: CaptureType;
  sourceKind: SourceKind;
  source: string | null;
  status: CaptureStatus;
  projectId: string | null;
  suggestedProjectId: string | null;
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

export function showQuickCapture() {
  return invoke<void>("show_quick_capture");
}
