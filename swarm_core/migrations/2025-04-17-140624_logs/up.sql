-- Your SQL goes here
CREATE TABLE logs (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    level VARCHAR(64) NOT NULL,
    module VARCHAR(64) NOT NULL,
    action VARCHAR(64) NOT NULL,
    expires_at TIMESTAMP NOT NULL,

    -- Embedding payload directly as nullable fields
    client_connected_ip TEXT,
    client_connected_username TEXT,

    job_submitted_job_id INTEGER,
    job_submitted_from_module VARCHAR(64),
    job_submitted_to_module VARCHAR(64),

    job_completed_job_id INTEGER,
    job_completed_success BOOLEAN,

    custom_msg TEXT
);

-- Add foreign key constraints for job references
ALTER TABLE logs
    ADD CONSTRAINT fk_job_submitted FOREIGN KEY (job_submitted_job_id) REFERENCES jobs(id) ON DELETE CASCADE,
    ADD CONSTRAINT fk_job_completed FOREIGN KEY (job_completed_job_id) REFERENCES jobs(id) ON DELETE CASCADE;




