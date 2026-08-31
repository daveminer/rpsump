CREATE TABLE "invite"
(
  id INTEGER PRIMARY KEY NOT NULL,
  email TEXT NOT NULL,
  token TEXT NOT NULL,
  invited_by_user_id INTEGER NOT NULL,
  expires_at DATETIME NOT NULL,
  accepted_at DATETIME,
  accepted_by_user_id INTEGER,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT fk_invite_invited_by FOREIGN KEY (invited_by_user_id) REFERENCES "user" (id),
  CONSTRAINT fk_invite_accepted_by FOREIGN KEY (accepted_by_user_id) REFERENCES "user" (id)
);

CREATE UNIQUE INDEX idx_invite_on_token ON "invite" (token);
CREATE INDEX idx_invite_on_email ON "invite" (email);
