-- This file should undo anything in `up.sql`
-- Drop job_metrics first as it references job_results and job_assignments.
DROP TABLE job_metrics;

-- Drop job_assignments next as it references job_results.
DROP TABLE job_assignments;

-- Finally, drop job_results since no other table depends on it.
DROP TABLE job_results;

