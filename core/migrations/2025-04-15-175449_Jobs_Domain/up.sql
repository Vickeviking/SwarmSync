-- Your SQL goes here
CREATE TABLE jobs (
    id SERIAL PRIMARY KEY,
    admin_id SERIAL REFERENCES admins(id) ON DELETE CASCADE,
    job_name TEXT NOT NULL,
    image_url TEXT NOT NULL,
    image_format varchar(64) NOT NULL,
    docker_flags TEXT[],  -- Optional flags for container execution
    output_type varchar(64) NOT NULL,  -- Either "Stdout" or "Files"
    output_paths TEXT[],  -- Only used when output_type = 'Files'
    schedule_type varchar(64) NOT NULL,  -- Either "Once" or "Cron"
    cron_expression TEXT,  -- Populated only if schedule_type = 'Cron'
    notes TEXT,
    state varchar(64) NOT NULL,
    error_message TEXT,  -- Populated only if state = 'Failed'
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

