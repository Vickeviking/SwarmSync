-- This file should undo anything in `up.sql`
-- Drop foreign key constraints
ALTER TABLE logs DROP CONSTRAINT IF EXISTS fk_job_submitted;
ALTER TABLE logs DROP CONSTRAINT IF EXISTS fk_job_completed;

-- Drop the columns added (if you want to remove them)
ALTER TABLE logs
    DROP COLUMN IF EXISTS job_submitted_job_id,
    DROP COLUMN IF EXISTS job_submitted_from_module,
    DROP COLUMN IF EXISTS job_submitted_to_module,
    DROP COLUMN IF EXISTS job_completed_job_id,
    DROP COLUMN IF EXISTS job_completed_success;

-- Optionally, drop the logs table if you want to remove it entirely
DROP TABLE IF EXISTS logs;
