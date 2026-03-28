CREATE TABLE "refresh_token"
(
  id INTEGER PRIMARY KEY NOT NULL,
  user_id INTEGER NOT NULL,
  token TEXT NOT NULL,
  expires_at DATETIME NOT NULL,
  revoked_at DATETIME,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT fk_refresh_token_user FOREIGN KEY (user_id) REFERENCES "user" (id)
);

CREATE UNIQUE INDEX idx_refresh_token_on_token ON "refresh_token" (token);
CREATE INDEX idx_refresh_token_on_user_id ON "refresh_token" (user_id);
