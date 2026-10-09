ALTER TABLE workout_exercises
    ALTER COLUMN target_weight TYPE NUMERIC(6,2)
    USING target_weight::numeric(6,2);