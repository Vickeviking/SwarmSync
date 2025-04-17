-- Your SQL goes here

CREATE TABLE workers (
    id SERIAL PRIMARY KEY,
    user_id SERIAL REFERENCES users(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    ip_address TEXT NOT NULL,
    hostname TEXT NOT NULL,
    ssh_user TEXT NOT NULL,
    ssh_key TEXT NOT NULL,  -- Encrypted or store a reference to a secrets store
    docker_version TEXT NOT NULL,
    arch TEXT NOT NULL,     -- e.g., "x86_64", "arm64"
    os varchar(64) NOT NULL,  -- Operating System of the worker
    tags TEXT[] DEFAULT '{}',  -- For affinity or constraints
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMP
);

CREATE TABLE worker_status (
    id SERIAL PRIMARY KEY,
    worker_id SERIAL UNIQUE REFERENCES workers(id) ON DELETE CASCADE,
    status varchar(64) NOT NULL,
    last_heartbeat TIMESTAMP,
    active_job_id INTEGER REFERENCES jobs(id),  -- Nullable reference to a current job
    uptime_sec INTEGER,
    load_avg REAL[],  -- e.g., [1min, 5min, 15min] load averages
    last_error TEXT,
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

