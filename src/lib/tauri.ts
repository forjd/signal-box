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
};

export function getDatabaseHealth() {
  return invoke<DatabaseHealth>("database_health");
}

export function getAppMetadata() {
  return invoke<AppMetadata>("app_metadata");
}
