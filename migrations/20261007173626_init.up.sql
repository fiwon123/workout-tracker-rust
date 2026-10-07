CREATE TABLE users (
    id            SERIAL PRIMARY KEY,
    email         TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE exercises (
    id          SERIAL PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    category    TEXT NOT NULL,               -- cardio | strength | flexibility
    muscle_group TEXT NOT NULL DEFAULT ''    -- chest | back | legs | ...
);

CREATE TABLE workouts (
    id           SERIAL PRIMARY KEY,
    user_id      INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    scheduled_at TIMESTAMPTZ NOT NULL,
    status       TEXT NOT NULL DEFAULT 'pending',  -- pending | completed
    comments     TEXT NOT NULL DEFAULT '',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE workout_exercises (
    id            SERIAL PRIMARY KEY,
    workout_id    INT NOT NULL REFERENCES workouts(id) ON DELETE CASCADE,
    exercise_id   INT NOT NULL REFERENCES exercises(id),
    position      INT NOT NULL DEFAULT 0,    -- order within the workout
    target_sets   INT NOT NULL,
    target_reps   INT NOT NULL,
    target_weight NUMERIC(6,2) NOT NULL DEFAULT 0
);

CREATE INDEX idx_workouts_user_scheduled  ON workouts (user_id, scheduled_at);
CREATE INDEX idx_workout_exercises_workout ON workout_exercises (workout_id);