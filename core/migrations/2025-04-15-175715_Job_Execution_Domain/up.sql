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
    job_id INTEGER NOT NULL,
    worker_id INTEGER NOT NULL,
    assigned_at TIMESTAMP NOT NULL DEFAULT now(),
    started_at TIMESTAMP,
    finished_at TIMESTAMP,
    FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE,
    FOREIGN KEY (worker_id) REFERENCES workers(id) ON DELETE CASCADE
);

CREATE TABLE job_metrics (
    id SERIAL PRIMARY KEY,
    job_id INTEGER NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    worker_id INTEGER NOT NULL REFERENCES workers(id) ON DELETE CASCADE,
    duration_sec INTEGER,
    cpu_usage_pct REAL,
    mem_usage_mb REAL,
    exit_code INTEGER,
    timestamp TIMESTAMP NOT NULL DEFAULT now(),
    CONSTRAINT unique_job_worker_pair UNIQUE (job_id, worker_id)
);


