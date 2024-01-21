-- Create Pages table with FK to Sites table
-- Add name and path columns as strings
-- Unique on site_id and page
CREATE TABLE
  Pages (
    page_id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid (),
    site_id UUID NOT NULL REFERENCES Sites (site_id),
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    UNIQUE (site_id, path)
  );
