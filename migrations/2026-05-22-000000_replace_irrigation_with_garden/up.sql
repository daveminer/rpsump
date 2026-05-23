DROP TABLE "irrigation_event";
DROP TABLE "irrigation_schedule";

CREATE TABLE "garden_schedule" (
  "id" INTEGER PRIMARY KEY NOT NULL,
  "name" TEXT NOT NULL,
  "active" BOOLEAN NOT NULL DEFAULT 1,
  "start_times" TEXT NOT NULL,
  "days_of_week" TEXT NOT NULL,
  "duration_secs" INTEGER NOT NULL,
  "created_at" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "updated_at" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE "garden_event" (
  "id" INTEGER PRIMARY KEY NOT NULL,
  "schedule_id" INTEGER,
  "source" TEXT NOT NULL,
  "status" TEXT NOT NULL,
  "scheduled_for" DATETIME NOT NULL,
  "duration_secs" INTEGER NOT NULL,
  "start_time" DATETIME,
  "end_time" DATETIME,
  "created_at" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY ("schedule_id") REFERENCES "garden_schedule" ("id") ON DELETE SET NULL
);

CREATE UNIQUE INDEX idx_garden_event_on_schedule_scheduled_for
  ON "garden_event" ("schedule_id", "scheduled_for");
CREATE INDEX idx_garden_event_on_status ON "garden_event" ("status");
CREATE INDEX idx_garden_event_on_created_at ON "garden_event" ("created_at");
CREATE INDEX idx_garden_schedule_on_active ON "garden_schedule" ("active");
