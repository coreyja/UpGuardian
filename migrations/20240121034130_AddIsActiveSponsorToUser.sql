-- Add migration script here
ALTER TABLE "users"
ADD COLUMN "is_active_sponsor" boolean NOT NULL DEFAULT false;
