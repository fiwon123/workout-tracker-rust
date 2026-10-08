-- Add up migration script here
ALTER TABLE users ADD COLUMN token_version INT NOT NULL DEFAULT 0;