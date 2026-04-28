ALTER TABLE `captures` ADD `processing_status` text DEFAULT 'idle' NOT NULL;
--> statement-breakpoint
ALTER TABLE `captures` ADD `processing_error` text;
--> statement-breakpoint
ALTER TABLE `captures` ADD `suggested_project_name` text;
--> statement-breakpoint
CREATE INDEX `captures_processing_status_idx` ON `captures` (`processing_status`);
