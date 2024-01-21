CREATE TABLE
  Checkins (
    checkin_id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid (),
    page_id UUID NOT NULL REFERENCES Pages (page_id),
    outcome TEXT NOT NULL,
    status_code INTEGER,
    created_at TIMESTAMP
    WITH
      TIME ZONE NOT NULL DEFAULT now ()
  );
