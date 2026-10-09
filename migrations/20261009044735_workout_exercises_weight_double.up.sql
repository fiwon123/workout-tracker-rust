ALTER TABLE workout_exercises
    ALTER COLUMN target_weight TYPE DOUBLE PRECISION
    USING target_weight::double precision;