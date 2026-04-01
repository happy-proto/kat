CREATE UNLOGGED TABLE theme_cache (
  id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  payload jsonb NOT NULL
);

CREATE TABLE theme_snapshots (
  id bigserial PRIMARY KEY,
  payload jsonb NOT NULL,
  note text DEFAULT $tag$dracula$tag$
);

CREATE FUNCTION refresh_theme_cache() RETURNS void
LANGUAGE plpgsql
IMMUTABLE
PARALLEL SAFE
AS $fn$
BEGIN
  RETURN;
END;
$fn$;

CREATE INDEX theme_payload_gin
ON theme_snapshots
USING GIN (payload);

INSERT INTO theme_snapshots (payload)
VALUES ('{"name":"Dracula"}')
ON CONFLICT (id) DO UPDATE
SET payload = EXCLUDED.payload
RETURNING id;

CREATE EXTENSION IF NOT EXISTS pgcrypto;
