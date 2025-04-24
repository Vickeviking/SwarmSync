/*
========================
🛠️ Supported Actions Job
========================

== CRUD ==
• `POST /jobs`          → create(NewJob) -> Job
• `GET /jobs/:id`       → find_by_id(id) -> Job
• `PATCH /jobs/:id`     → update(id, Job) -> Job
• `DELETE /jobs/:id`    → delete(id) -> usize

== Lookup & Search ==
• `GET /jobs/search`    → search_by_job_name(user_id, query) -> Vec<Job>
• `GET /jobs/name/:str` → find_by_name(user_id, name) -> Option<Job>
• `GET /jobs/by_admin`  → list_by_admin(user_id, limit, offset) -> Vec<Job>
• `GET /jobs/state/:st` → list_by_state(state) -> Vec<Job>
• `GET /jobs/recent`    → get_recent_jobs(limit) -> Vec<Job>
• `GET /jobs/failed`    → get_failed_jobs(limit) -> Vec<Job>

== State Transitions ==
• `PATCH /jobs/:id/running`   → mark_running(id) -> Job
• `PATCH /jobs/:id/succeeded` → mark_succeeded(id) -> Job
• `PATCH /jobs/:id/failed`    → mark_failed(id, msg) -> Job

== Scheduling & Readiness ==
• `GET /jobs/scheduled`       → list_scheduled_jobs() -> Vec<Job>
• `GET /jobs/cron_due`        → list_due_cron_jobs(current_time) -> Vec<Job>
• `GET /jobs/ready`           → list_one_time_jobs_ready() -> Vec<Job>

== Aggregation & Stats ==
• `GET /jobs/stats/admins`    → get_job_counts_per_admin() -> Vec<(admin_id, job_count)>

== Assignment-related ==
• `GET /jobs/active/:worker`  → get_active_jobs_for_worker(worker_id) -> Vec<Job>
• `GET /jobs/assigned/:worker`→ find_jobs_assigned_to_worker(worker_id) -> Vec<Job>
• `GET /jobs/unassigned`      → list_jobs_with_no_assignment() -> Vec<Job>

*/
use diesel::dsl::now;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::database::models::job::{Job, NewJob};

use crate::shared::enums::{job::JobStateEnum, schedule::ScheduleTypeEnum};

use crate::database::schema::*;
use diesel::dsl::count_star;

use chrono::NaiveDateTime;

pub struct JobRepository;

impl JobRepository {
    pub async fn find_by_id(c: &mut AsyncPgConnection, id: i32) -> QueryResult<Job> {
        jobs::table.find(id).get_result(c).await
    }

    pub async fn create(c: &mut AsyncPgConnection, new_job: NewJob) -> QueryResult<Job> {
        dbg!(&new_job.state);

        match diesel::insert_into(jobs::table)
            .values(new_job)
            .get_result::<Job>(c)
            .await
        {
            Ok(job) => {
                let _ = Self::mark_submitted(c, job.id).await;
                Ok(job)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn update(c: &mut AsyncPgConnection, id: i32, job: Job) -> QueryResult<Job> {
        diesel::update(jobs::table.find(id))
            .set((
                jobs::job_name.eq(job.job_name),
                jobs::image_url.eq(job.image_url),
                jobs::image_format.eq(job.image_format),
                jobs::docker_flags.eq(job.docker_flags),
                jobs::output_type.eq(job.output_type),
                jobs::output_paths.eq(job.output_paths),
                jobs::schedule_type.eq(job.schedule_type),
                jobs::cron_expression.eq(job.cron_expression),
                jobs::notes.eq(job.notes),
                jobs::state.eq(job.state),
                jobs::error_message.eq(job.error_message),
                jobs::updated_at.eq(now),
            ))
            .get_result(c)
            .await
    }

    pub async fn delete(c: &mut AsyncPgConnection, id: i32) -> QueryResult<usize> {
        diesel::delete(jobs::table.find(id)).execute(c).await
    }

    pub async fn find_by_name(
        c: &mut AsyncPgConnection,
        user_id: i32,
        name: &str,
    ) -> QueryResult<Option<Job>> {
        jobs::table
            .filter(jobs::user_id.eq(user_id))
            .filter(jobs::job_name.eq(name))
            .first::<Job>(c)
            .await
            .optional()
    }

    pub async fn list_by_admin(
        c: &mut AsyncPgConnection,
        user_id: i32,
        limit: i64,
        offset: i64,
    ) -> QueryResult<Vec<Job>> {
        jobs::table
            .filter(jobs::user_id.eq(user_id))
            .limit(limit)
            .offset(offset)
            .order(jobs::created_at.desc())
            .load(c)
            .await
    }

    pub async fn list_scheduled_jobs(c: &mut AsyncPgConnection) -> QueryResult<Vec<Job>> {
        jobs::table
            .filter(jobs::state.eq(JobStateEnum::Queued))
            .order(jobs::created_at.desc())
            .load(c)
            .await
    }

    pub async fn search_by_job_name(
        c: &mut AsyncPgConnection,
        user_id: i32,
        query: &str,
    ) -> QueryResult<Vec<Job>> {
        jobs::table
            .filter(jobs::user_id.eq(user_id))
            .filter(jobs::job_name.ilike(format!("%{}%", query)))
            .order(jobs::created_at.desc())
            .load(c)
            .await
    }

    pub async fn list_by_state(
        c: &mut AsyncPgConnection,
        state: JobStateEnum,
    ) -> QueryResult<Vec<Job>> {
        jobs::table
            .filter(jobs::state.eq(state))
            .order(jobs::created_at.desc())
            .load(c)
            .await
    }

    pub async fn mark_failed(
        c: &mut AsyncPgConnection,
        id: i32,
        message: &str,
    ) -> QueryResult<Job> {
        diesel::update(jobs::table.find(id))
            .set((
                jobs::state.eq(JobStateEnum::Failed),
                jobs::error_message.eq::<Option<String>>(Some(message.to_string())),
                jobs::updated_at.eq(now),
            ))
            .get_result(c)
            .await
    }

    pub async fn mark_succeeded(c: &mut AsyncPgConnection, id: i32) -> QueryResult<Job> {
        diesel::update(jobs::table.find(id))
            .set((
                jobs::state.eq(JobStateEnum::Completed),
                jobs::error_message.eq(None::<String>),
                jobs::updated_at.eq(now),
            ))
            .get_result(c)
            .await
    }

    pub async fn mark_running(c: &mut AsyncPgConnection, id: i32) -> QueryResult<Job> {
        diesel::update(jobs::table.find(id))
            .set((
                jobs::state.eq(JobStateEnum::Running),
                jobs::updated_at.eq(now),
            ))
            .get_result(c)
            .await
    }

    pub async fn mark_submitted(c: &mut AsyncPgConnection, id: i32) -> QueryResult<Job> {
        diesel::update(jobs::table.find(id))
            .set((
                jobs::state.eq(JobStateEnum::Submitted),
                jobs::updated_at.eq(now),
            ))
            .get_result(c)
            .await
    }

    pub async fn mark_queued(c: &mut AsyncPgConnection, id: i32) -> QueryResult<Job> {
        diesel::update(jobs::table.find(id))
            .set((
                jobs::state.eq(JobStateEnum::Queued),
                jobs::updated_at.eq(now),
            ))
            .get_result(c)
            .await
    }

    pub async fn list_due_cron_jobs(
        c: &mut AsyncPgConnection,
        current_time: NaiveDateTime,
    ) -> QueryResult<Vec<Job>> {
        jobs::table
            .filter(jobs::schedule_type.eq(ScheduleTypeEnum::Cron))
            .filter(jobs::created_at.le(current_time))
            .load(c)
            .await
    }

    pub async fn list_one_time_jobs_ready(c: &mut AsyncPgConnection) -> QueryResult<Vec<Job>> {
        jobs::table
            .filter(jobs::schedule_type.eq(ScheduleTypeEnum::Once))
            .filter(jobs::state.eq(JobStateEnum::Queued))
            .load(c)
            .await
    }

    pub async fn get_job_counts_per_admin(
        c: &mut AsyncPgConnection,
    ) -> QueryResult<Vec<(i32, i64)>> {
        jobs::table
            .group_by(jobs::user_id) // Call on jobs::table, not jobs::group_by
            .select((jobs::user_id, count_star()))
            .load::<(i32, i64)>(c)
            .await
    }

    pub async fn get_recent_jobs(c: &mut AsyncPgConnection, limit: i64) -> QueryResult<Vec<Job>> {
        jobs::table
            .order(jobs::created_at.desc())
            .limit(limit)
            .load(c)
            .await
    }

    pub async fn get_failed_jobs(c: &mut AsyncPgConnection, limit: i64) -> QueryResult<Vec<Job>> {
        jobs::table
            .filter(jobs::state.eq(JobStateEnum::Failed))
            .order(jobs::updated_at.desc())
            .limit(limit)
            .load(c)
            .await
    }

    pub async fn get_active_jobs_for_worker(
        c: &mut AsyncPgConnection,
        worker_id: i32,
    ) -> QueryResult<Vec<Job>> {
        job_assignments::table
            .inner_join(jobs::table.on(job_assignments::job_id.eq(jobs::id)))
            .filter(job_assignments::worker_id.eq(worker_id))
            .filter(jobs::state.eq(JobStateEnum::Running))
            .select(jobs::all_columns)
            .load(c)
            .await
    }

    pub async fn find_jobs_assigned_to_worker(
        c: &mut AsyncPgConnection,
        worker_id: i32,
    ) -> QueryResult<Vec<Job>> {
        job_assignments::table
            .inner_join(jobs::table.on(job_assignments::job_id.eq(jobs::id)))
            .filter(job_assignments::worker_id.eq(worker_id))
            .select(jobs::all_columns)
            .load(c)
            .await
    }

    pub async fn list_jobs_with_no_assignment(c: &mut AsyncPgConnection) -> QueryResult<Vec<Job>> {
        jobs::table
            .left_outer_join(job_assignments::table.on(job_assignments::job_id.eq(jobs::id)))
            .filter(job_assignments::job_id.is_null())
            .select(jobs::all_columns)
            .load(c)
            .await
    }
}
