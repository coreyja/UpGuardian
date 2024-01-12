-- Add migration script here
CREATE TABLE
  Users (
    user_id UUID PRIMARY KEY NOT NULL,
    coreyja_user_id UUID NOT NULL
  );
