-- Denormalize the schedule name onto the event so history survives a rename
-- or a delete (the schedule_id FK is ON DELETE SET NULL).
ALTER TABLE "garden_event" ADD COLUMN "schedule_name" TEXT;

UPDATE "garden_event"
SET "schedule_name" = (
  SELECT "name"
  FROM "garden_schedule"
  WHERE "garden_schedule"."id" = "garden_event"."schedule_id"
)
WHERE "schedule_id" IS NOT NULL;
