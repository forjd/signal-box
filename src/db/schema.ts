import { sql } from "drizzle-orm";
import { index, integer, sqliteTable, text, uniqueIndex } from "drizzle-orm/sqlite-core";

const timestamps = {
  createdAt: text("created_at")
    .notNull()
    .default(sql`CURRENT_TIMESTAMP`),
  updatedAt: text("updated_at")
    .notNull()
    .default(sql`CURRENT_TIMESTAMP`),
};

export const captures = sqliteTable(
  "captures",
  {
    id: text("id").primaryKey(),
    rawText: text("raw_text").notNull(),
    title: text("title"),
    summary: text("summary"),
    captureType: text("capture_type").notNull().default("note"),
    sourceKind: text("source_kind").notNull().default("typed"),
    source: text("source"),
    status: text("status").notNull().default("unprocessed"),
    processingStatus: text("processing_status").notNull().default("idle"),
    processingError: text("processing_error"),
    projectId: text("project_id").references(() => projects.id, {
      onDelete: "set null",
    }),
    suggestedProjectId: text("suggested_project_id").references(() => projects.id, {
      onDelete: "set null",
    }),
    suggestedProjectName: text("suggested_project_name"),
    processedAt: text("processed_at"),
    archivedAt: text("archived_at"),
    ...timestamps,
  },
  (table) => [
    index("captures_status_idx").on(table.status),
    index("captures_project_idx").on(table.projectId),
    index("captures_suggested_project_idx").on(table.suggestedProjectId),
    index("captures_source_kind_idx").on(table.sourceKind),
    index("captures_processing_status_idx").on(table.processingStatus),
  ],
);

export const projects = sqliteTable("projects", {
  id: text("id").primaryKey(),
  name: text("name").notNull(),
  description: text("description"),
  overview: text("overview").notNull().default(""),
  currentDirection: text("current_direction").notNull().default(""),
  summary: text("summary"),
  memory: text("memory").notNull().default(""),
  status: text("status").notNull().default("active"),
  ...timestamps,
});

export const tasks = sqliteTable(
  "tasks",
  {
    id: text("id").primaryKey(),
    projectId: text("project_id").references(() => projects.id, { onDelete: "set null" }),
    captureId: text("capture_id").references(() => captures.id, { onDelete: "set null" }),
    title: text("title").notNull(),
    description: text("description"),
    status: text("status").notNull().default("open"),
    ...timestamps,
  },
  (table) => [
    index("tasks_project_idx").on(table.projectId),
    index("tasks_capture_idx").on(table.captureId),
  ],
);

export const decisions = sqliteTable(
  "decisions",
  {
    id: text("id").primaryKey(),
    projectId: text("project_id").references(() => projects.id, { onDelete: "set null" }),
    captureId: text("capture_id").references(() => captures.id, { onDelete: "set null" }),
    title: text("title").notNull(),
    context: text("context"),
    decision: text("decision").notNull(),
    rationale: text("rationale"),
    status: text("status").notNull().default("proposed"),
    ...timestamps,
  },
  (table) => [
    index("decisions_project_idx").on(table.projectId),
    index("decisions_capture_idx").on(table.captureId),
  ],
);

export const questions = sqliteTable(
  "questions",
  {
    id: text("id").primaryKey(),
    projectId: text("project_id").references(() => projects.id, { onDelete: "set null" }),
    captureId: text("capture_id").references(() => captures.id, { onDelete: "set null" }),
    question: text("question").notNull(),
    answer: text("answer"),
    status: text("status").notNull().default("open"),
    ...timestamps,
  },
  (table) => [
    index("questions_project_idx").on(table.projectId),
    index("questions_capture_idx").on(table.captureId),
  ],
);

export const sources = sqliteTable(
  "sources",
  {
    id: text("id").primaryKey(),
    projectId: text("project_id").references(() => projects.id, { onDelete: "set null" }),
    captureId: text("capture_id").references(() => captures.id, { onDelete: "set null" }),
    title: text("title").notNull(),
    kind: text("kind").notNull().default("text"),
    sourceType: text("source_type").notNull().default("text"),
    url: text("url"),
    rawExcerpt: text("raw_excerpt"),
    rawReference: text("raw_reference"),
    notes: text("notes"),
    ...timestamps,
  },
  (table) => [
    index("sources_project_idx").on(table.projectId),
    index("sources_capture_idx").on(table.captureId),
  ],
);

export const artefacts = sqliteTable(
  "artefacts",
  {
    id: text("id").primaryKey(),
    projectId: text("project_id").references(() => projects.id, { onDelete: "set null" }),
    title: text("title").notNull(),
    artefactType: text("artefact_type").notNull(),
    summary: text("summary"),
    bodyMarkdown: text("body_markdown").notNull().default(""),
    model: text("model"),
    provider: text("provider"),
    markdownBody: text("markdown_body").notNull(),
    metadataJson: text("metadata_json").notNull().default("{}"),
    ...timestamps,
  },
  (table) => [index("artefacts_project_idx").on(table.projectId)],
);

export const relationships = sqliteTable(
  "relationships",
  {
    id: text("id").primaryKey(),
    fromType: text("from_type").notNull(),
    fromId: text("from_id").notNull(),
    toType: text("to_type").notNull(),
    toId: text("to_id").notNull(),
    relationshipType: text("relationship_type").notNull(),
    ...timestamps,
  },
  (table) => [
    index("relationships_from_idx").on(table.fromType, table.fromId),
    index("relationships_to_idx").on(table.toType, table.toId),
    uniqueIndex("relationships_unique_idx").on(
      table.fromType,
      table.fromId,
      table.toType,
      table.toId,
      table.relationshipType,
    ),
  ],
);

export const embeddings = sqliteTable(
  "embeddings",
  {
    id: text("id").primaryKey(),
    entityType: text("entity_type"),
    entityId: text("entity_id"),
    ownerType: text("owner_type").notNull(),
    ownerId: text("owner_id").notNull(),
    provider: text("provider").notNull(),
    model: text("model").notNull(),
    dimensions: integer("dimensions").notNull(),
    embedding: text("embedding"),
    vectorJson: text("vector_json").notNull(),
    contentHash: text("content_hash").notNull(),
    ...timestamps,
  },
  (table) => [
    index("embeddings_owner_idx").on(table.ownerType, table.ownerId),
    index("embeddings_entity_idx").on(table.entityType, table.entityId),
  ],
);

export const settings = sqliteTable(
  "settings",
  {
    key: text("key").primaryKey(),
    valueJson: text("value_json").notNull(),
    isSecret: integer("is_secret", { mode: "boolean" }).notNull().default(false),
    updatedAt: text("updated_at")
      .notNull()
      .default(sql`CURRENT_TIMESTAMP`),
  },
  (table) => [index("settings_secret_idx").on(table.isSecret)],
);

export type Capture = typeof captures.$inferSelect;
export type Project = typeof projects.$inferSelect;
export type Task = typeof tasks.$inferSelect;
export type Decision = typeof decisions.$inferSelect;
export type Question = typeof questions.$inferSelect;
export type Source = typeof sources.$inferSelect;
export type Artefact = typeof artefacts.$inferSelect;
export type Relationship = typeof relationships.$inferSelect;
export type Embedding = typeof embeddings.$inferSelect;
export type Setting = typeof settings.$inferSelect;
