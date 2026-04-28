ALTER TABLE `projects` ADD `description` text;
--> statement-breakpoint
ALTER TABLE `projects` ADD `overview` text DEFAULT '' NOT NULL;
--> statement-breakpoint
ALTER TABLE `projects` ADD `current_direction` text DEFAULT '' NOT NULL;
--> statement-breakpoint
ALTER TABLE `sources` ADD `kind` text DEFAULT 'text' NOT NULL;
--> statement-breakpoint
ALTER TABLE `sources` ADD `raw_excerpt` text;
--> statement-breakpoint
ALTER TABLE `sources` ADD `notes` text;
--> statement-breakpoint
CREATE UNIQUE INDEX `relationships_unique_idx` ON `relationships` (`from_type`,`from_id`,`to_type`,`to_id`,`relationship_type`);
