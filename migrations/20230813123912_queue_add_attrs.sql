-- Add migration script here
ALTER TABLE queues ADD COLUMN type TEXT NOT NULL DEFAULT '';
