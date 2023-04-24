CREATE TABLE "user"
(
  id INTEGER PRIMARY KEY NOT NULL,
  email TEXT NOT NULL,
  password_hash TEXT NOT NULL,
  password_reset_token_hash TEXT,
  password_reset_token_expires_at TEXT,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_user_on_email ON "user" (email);
CREATE INDEX idx_user_on_created_at ON "user" (created_at);
CREATE INDEX idx_user_on_password_reset_token_expires_at ON "user" (password_reset_token_expires_at);
