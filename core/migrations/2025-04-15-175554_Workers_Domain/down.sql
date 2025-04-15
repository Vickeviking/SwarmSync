-- This file should undo anything in `up.sql`

-- Drop worker_status first since it depends on workers.
DROP TABLE worker_status;

-- Then drop workers.
DROP TABLE workers;

