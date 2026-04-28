ALTER TABLE `captures` RENAME COLUMN `raw_content` TO `raw_text`;
--> statement-breakpoint
ALTER TABLE `captures` ADD `source_kind` text DEFAULT 'typed' NOT NULL;
--> statement-breakpoint
ALTER TABLE `captures` ADD `source` text;
--> statement-breakpoint
ALTER TABLE `captures` ADD `project_id` text REFERENCES `projects`(`id`) ON UPDATE no action ON DELETE set null;
--> statement-breakpoint
CREATE INDEX `captures_project_idx` ON `captures` (`project_id`);
--> statement-breakpoint
CREATE INDEX `captures_source_kind_idx` ON `captures` (`source_kind`);
