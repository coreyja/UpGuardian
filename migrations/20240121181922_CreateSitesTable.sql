-- Create Sites table, with FK to Users table for owner
-- With name, domain and description fields. Name and domain and non-null
-- but description is nullable
-- domain is unique to the owner
CREATE TABLE
  Sites (
    site_id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid (),
    name TEXT NOT NULL,
    domain TEXT NOT NULL,
    description TEXT,
    user_id UUID NOT NULL REFERENCES Users (user_id),
    UNIQUE (user_id, domain)
  );
