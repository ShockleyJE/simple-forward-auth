-- Create API Keys table, enable UUID & crypto
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE api_keys(
   id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
   environment varchar(50) not null unique,
   token TEXT NOT NULL,
   created_at timestamptz DEFAULT NOW()
);

CREATE INDEX idx_api_keys_token ON api_keys(token);