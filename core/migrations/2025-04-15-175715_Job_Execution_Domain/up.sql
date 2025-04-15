-- Your SQL goes here
CREATE TABLE job_results (
    id SERIAL PRIMARY KEY,
    job_id SERIAL REFERENCES jobs(id) ON DELETE CASCADE,
    stdout TEXT,  -- Captured stdout when output_type = 'Stdout'
    files TEXT[],  -- JSON structure mapping filenames to details (download URL, hash, etc.)
    saved_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE TABLE job_assignments (
    id SERIAL PRIMARY KEY,
    job_id SERIAL REFERENCES jobs(id) ON DELETE CASCADE,
    worker_id SERIAL REFERENCES workers(id) ON DELETE CASCADE,
    assigned_at TIMESTAMP NOT NULL DEFAULT now(),
    started_at TIMESTAMP,
    finished_at TIMESTAMP
);

CREATE TABLE job_metrics (
    id SERIAL PRIMARY KEY,
    job_id SERIAL UNIQUE REFERENCES jobs(id) ON DELETE CASCADE,
    worker_id SERIAL REFERENCES workers(id) ON DELETE CASCADE,
    duration_sec INTEGER,      -- Runtime duration in seconds
    cpu_usage_pct REAL,        -- Average CPU usage percentage
    mem_usage_mb REAL,         -- Maximum memory usage in megabytes
    exit_code INTEGER,         -- Exit code from the job's execution process
    timestamp TIMESTAMP NOT NULL DEFAULT now()  -- Timestamp when metrics are recorded (e.g., on completion)
);

