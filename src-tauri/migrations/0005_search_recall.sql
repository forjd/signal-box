ALTER TABLE `embeddings` ADD `entity_type` text;
--> statement-breakpoint
ALTER TABLE `embeddings` ADD `entity_id` text;
--> statement-breakpoint
ALTER TABLE `embeddings` ADD `embedding` text;
--> statement-breakpoint
CREATE INDEX `embeddings_entity_idx` ON `embeddings` (`entity_type`,`entity_id`);
