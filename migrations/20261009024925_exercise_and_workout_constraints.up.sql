CREATE UNIQUE INDEX idx_exercises_name ON exercises (name);

ALTER TABLE workouts ADD COLUMN completed_at TIMESTAMPTZ;

ALTER TABLE workouts
    ADD CONSTRAINT workouts_status_check
    CHECK (status IN ('pending', 'completed'));