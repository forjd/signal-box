import { sql } from "drizzle-orm";
import { index, integer, sqliteTable, text } from "drizzle-orm/sqlite-core";

const timestamps = {
  createdAt: text("created_at").notNull().default(sql`CURRENT_TIMESTAMP`),
  updatedAt: text("updated_at").notNull().default(sql`CURRENT_TIMESTAMP`),
};

export const captures = sqliteTable(
  "captures",
  {
    id: text("id").primaryKey(),
    rawContent: text("raw_content").notNull(),
    title: text("title"),
    summary: text("summary"),
    captureType: text("capture_type").notNull().default("note"),
    status: text("status").notNull().default("unprocessed"),
    suggestedProjectId: text("suggested_project_id").references(() => projects.id, {
      onDelete: "set null",
    }),
    processedAt: text("processed_at"),
    archivedAt: text("archived_at"),
    ...timestamps,
  },
  (table) => [
    index("captures_status_idx").on(table.status),
    index("captures_suggested_project_idx").on(table.suggestedProjectId),
  ],
);

export const projects = sqliteTable("projects", {
  id: text("id").primaryKey(),
  name: text("name").notNull(),
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
    sourceType: text("source_type").notNull().default("text"),
    url: text("url"),
    rawReference: text("raw_reference"),
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
  ],
);

export const embeddings = sqliteTable(
  "embeddings",
  {
    id: text("id").primaryKey(),
    ownerType: text("owner_type").notNull(),
    ownerId: text("owner_id").notNull(),
    provider: text("provider").notNull(),
    model: text("model").notNull(),
    dimensions: integer("dimensions").notNull(),
    vectorJson: text("vector_json").notNull(),
    contentHash: text("content_hash").notNull(),
    ...timestamps,
  },
  (table) => [index("embeddings_owner_idx").on(table.ownerType, table.ownerId)],
);

export const settings = sqliteTable(
  "settings",
  {
    key: text("key").primaryKey(),
    valueJson: text("value_json").notNull(),
    isSecret: integer("is_secret", { mode: "boolean" }).notNull().default(false),
    updatedAt: text("updated_at").notNull().default(sql`CURRENT_TIMESTAMP`),
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
