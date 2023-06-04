CREATE TABLE "user"
(
  id INTEGER PRIMARY KEY NOT NULL,
  email TEXT NOT NULL,
  email_verification_token TEXT,
  email_verification_token_expires_at DATETIME,
  email_verified_at DATETIME,
  password_hash TEXT NOT NULL,
  password_reset_token TEXT,
  password_reset_token_expires_at DATETIME,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT unique_email UNIQUE (email)
);

CREATE INDEX idx_user_on_email ON "user" (email);
CREATE INDEX idx_user_on_created_at ON "user" (created_at);
CREATE INDEX idx_user_on_password_reset_token_expires_at ON "user" (password_reset_token_expires_at);
