# Workout Tracker API

A RESTful backend for a workout tracker built with Rust (axum), PostgreSQL (sqlx), and JWT authentication.

Users can sign up, log in, create workout plans composed of exercises (sets/reps/weight), schedule them, mark them complete, and generate progress reports.

## Features

- JWT authentication (HS256) with password hashing (argon2) and token revocation via `token_version`
- Exercise catalog seeded from `src/bin/seed.rs`
- Workout CRUD with scheduled dates and per-exercise sets, reps, and weight
- Mark workouts complete (once), list by `pending`/`completed` status sorted by time
- Reports summary: totals, volume, breakdown by muscle group and category, weekly frequency
- OpenAPI 3 docs served via Swagger UI
- Unit + integration tests (`#[sqlx::test]`, isolated DB per test)

## Requirements

- Rust (edition 2024)
- Docker (for PostgreSQL), or a local Postgres server

## Setup

```bash
# 1. configure environment
cp .env.example .env
#    edit .env and set a strong JWT_SECRET

# 2. start the database
docker compose up -d db

# 3. run migrations
cargo run --bin migrate

# 4. seed the exercise catalog (idempotent)
cargo run --bin seed

# 5. start the API
cargo run
```

The API listens on `http://127.0.0.1:3000` by default.

Alternatively, run everything with `docker compose up --build` (db, migrate, app).

## Environment Variables

| Variable       | Default           | Description                              |
| -------------- | ----------------- | ---------------------------------------- |
| `DATABASE_URL` | (required)        | Postgres connection string               |
| `JWT_SECRET`   | (required)        | HS256 signing secret for JWTs            |
| `JWT_TTL_SECS` | `3600`            | Token lifetime in seconds                |
| `LISTEN_ADDR`  | `127.0.0.1:3000`  | HTTP listen address                      |
| `POSTGRES_USER`/`POSTGRES_PASSWORD`/`POSTGRES_DB` | `workout` | Used by docker-compose |

## API Overview

Interactive docs: http://127.0.0.1:3000/swagger-ui
Spec (JSON): http://127.0.0.1:3000/api-docs/openapi.json

| Method | Path                     | Auth | Description                          |
| ------ | ------------------------ | ---- | ------------------------------------ |
| POST   | `/auth/signup`           | No   | Create an account                    |
| POST   | `/auth/login`            | No   | Log in, returns a JWT                |
| POST   | `/auth/logout`           | Yes  | Revoke the current token             |
| GET    | `/exercises`             | No   | List exercises (filter by category / muscle_group) |
| POST   | `/workouts`              | Yes  | Create a workout with exercises      |
| GET    | `/workouts`              | Yes  | List workouts (filter by status)     |
| GET    | `/workouts/{id}`         | Yes  | Get one workout with its exercises   |
| PATCH  | `/workouts/{id}`         | Yes  | Update name/schedule/comments/exercises |
| DELETE | `/workouts/{id}`         | Yes  | Delete a workout                     |
| POST   | `/workouts/{id}/complete`| Yes  | Mark a pending workout completed     |
| GET    | `/reports/summary`       | Yes  | Aggregated progress report           |
| GET    | `/health`                | No   | Health check                         |

All protected endpoints require `Authorization: Bearer <token>` and only return the logged-in user's own workouts.

## Usage Examples (curl)

```bash
# sign up
curl -i -X POST http://127.0.0.1:3000/auth/signup \
  -H 'Content-Type: application/json' \
  -d '{"email": "user@example.com", "password": "password1"}'
# -> 201 {"token": "...."}

TOKEN="<token-from-signup-or-login>"

# list exercises
curl http://127.0.0.1:3000/exercises?category=strength

# create a scheduled workout composed of exercises
curl -i -X POST http://127.0.0.1:3000/workouts \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "Push day",
    "scheduled_at": "2030-01-02T10:00:00Z",
    "comments": "",
    "exercises": [
      {"exercise_id": 1, "sets": 3, "reps": 10, "weight": 50.0},
      {"exercise_id": 6, "sets": 4, "reps": 8,  "weight": 20.0}
    ]
  }'
# -> 201 {"id": 1}

# list pending workouts, sorted by date
curl -H "Authorization: Bearer $TOKEN" \
  'http://127.0.0.1:3000/workouts?status=pending'

# update (e.g. add a comment)
curl -i -X PATCH http://127.0.0.1:3000/workouts/1 \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"comments": "felt great today"}'

# mark complete
curl -i -X POST http://127.0.0.1:3000/workouts/1/complete \
  -H "Authorization: Bearer $TOKEN"
# -> 409 if already completed

# progress report
curl -H "Authorization: Bearer $TOKEN" \
  'http://127.0.0.1:3000/reports/summary'
```

## Reports

`GET /reports/summary` accepts optional `from` and `to` (RFC 3339) filters and returns:

- `totals` — total/completed/pending workouts, sets, reps, and total volume (`sets * reps * weight`)
- `by_muscle_group` / `by_category` — completed-workout volume breakdown
- `weekly_frequency` — completed workouts grouped by week

## Tests

```bash
docker compose up -d db
cargo test
```

`#[sqlx::test]` creates an isolated temporary database per test and runs migrations automatically, so no seed data is needed for tests.

## Project Layout

```
src/
  bin/          migrate and seed binaries
  routes/       auth, exercises, workouts, reports handlers
  auth/         password hashing, JWT, auth middleware
  error.rs      unified ApiError -> HTTP responses
  openapi.rs    OpenAPI document + security scheme
  lib.rs        AppState + router assembly
tests/          integration tests
migrations/     sqlx migrations
```