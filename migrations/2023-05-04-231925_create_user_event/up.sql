CREATE TABLE "user_event" (
  "id" INTEGER PRIMARY KEY NOT NULL,
  "user_id" INTEGER NOT NULL,
  "event_type" TEXT CHECK(event_type IN ('login', 'logout', 'signup', 'failed_login', 'locked_login', 'password_reset')) NOT NULL,
  "ip_address" TEXT NOT NULL,
  "created_at" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY ("user_id") REFERENCES "user" ("id") ON DELETE CASCADE
);

CREATE INDEX idx_user_event_on_user_id ON "user_event" ("user_id");
CREATE INDEX idx_user_event_on_ip_address ON "user_event" ("ip_address");
CREATE INDEX idx_user_event_on_created_at ON "user_event" ("created_at");
