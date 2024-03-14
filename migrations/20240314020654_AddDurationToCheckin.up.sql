-- Add migration script here
-- Add a new column to the checkin table
-- `duration_ms` stores the duration of the checkin in milliseconds
ALTER TABLE Checkins
ADD COLUMN duration_nanos BIGINT;
