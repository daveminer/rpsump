CREATE TABLE "user"
(
  id INTEGER PRIMARY KEY NOT NULL,
  email TEXT NOT NULL,
  email_verification_token TEXT,
  email_verification_token_expires_at TIMESTAMP,
  email_verified_at TIMESTAMP,
  password_hash TEXT NOT NULL,
  password_reset_token TEXT,
  password_reset_token_expires_at TEXT,
  activated BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT unique_email UNIQUE (email)
);

CREATE INDEX idx_user_on_email ON "user" (email);
CREATE INDEX idx_user_on_created_at ON "user" (created_at);
CREATE INDEX idx_user_on_password_reset_token_expires_at ON "user" (password_reset_token_expires_at);
