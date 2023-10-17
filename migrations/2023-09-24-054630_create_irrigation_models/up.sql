CREATE TABLE "irrigation_schedule" (
  "id" INTEGER PRIMARY KEY NOT NULL,
  "active" BOOLEAN NOT NULL,
  "name" TEXT NOT NULL,
  "start_time" TIME NOT NULL,
  "days_of_week" TEXT NOT NULL,
  "hoses" TEXT NOT NULL,
  "created_at" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "updated_at" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE "irrigation_event" (
  "id" INTEGER PRIMARY KEY NOT NULL,
  "hose_id" INTEGER NOT NULL,
  "created_at" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "end_time" DATETIME,
  "status" TEXT NOT NULL,
  "schedule_id" INTEGER NOT NULL,
  FOREIGN KEY ("schedule_id") REFERENCES "irrigation_schedule" ("id") ON DELETE CASCADE
);

CREATE INDEX idx_irrigation_schedule_on_start_time ON "irrigation_schedule" ("start_time");
CREATE INDEX idx_irrigation_schedule_on_created_at ON "irrigation_schedule" ("created_at");

CREATE INDEX idx_irrigation_event_on_created_at ON "irrigation_event" ("created_at");
CREATE INDEX idx_irrigation_event_on_end_time ON "irrigation_event" ("end_time");
CREATE INDEX idx_irrigation_event_on_schedule_id ON "irrigation_event" ("schedule_id");
