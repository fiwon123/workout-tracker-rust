ALTER TABLE workouts DROP CONSTRAINT IF EXISTS workouts_status_check;
ALTER TABLE workouts DROP COLUMN IF EXISTS completed_at;
DROP INDEX IF EXISTS idx_exercises_name;