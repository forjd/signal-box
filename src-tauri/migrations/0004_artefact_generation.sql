ALTER TABLE `artefacts` ADD `summary` text;
--> statement-breakpoint
ALTER TABLE `artefacts` ADD `body_markdown` text DEFAULT '' NOT NULL;
--> statement-breakpoint
ALTER TABLE `artefacts` ADD `model` text;
--> statement-breakpoint
ALTER TABLE `artefacts` ADD `provider` text;
