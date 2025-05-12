# ChronoSwarm Roadmap

## Vision & Objectives

Sync a swarm of jobs with real-time telemetry and multi-node scheduling.

---

## Project Modules

- **swarm-core**: central scheduler, persistence, API
- **swarm-consumer**: dashboard, monitoring, user interface
- **swarm-runner** (formerly swarm-worker): executes jobs and streams logs

---

## High-Level Timeline

1. **3 weeks – Core development**

   - Finalize scheduler, persistence, and API endpoints
   - Define database schema for cron_jobs, runners, and job_logs
   - Implement REST and WebSocket interfaces in swarm-core

2. **2 weeks – Runner development**

   - Rename swarm-worker to swarm-runner across codebase
   - Build runner registration, heartbeat, and job execution logic
   - Enable secure Docker isolation and log streaming

3. **Ongoing – Consumer and UI enhancements**

   - Integrate live telemetry from core and runners
   - Improve scheduling UI and job management features
   - Polish authentication, access control, and alerts

--

## Next Steps

- Update module names (`swarm-worker` → `swarm-runner`) in repository
