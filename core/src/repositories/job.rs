use diesel::dsl::now;
use diesel::dsl::IntervalDsl;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::models::job::{
    Job, JobAssignment, JobMetric, JobResult, NewJob, NewJobAssignment, NewJobMetric, NewJobResult,
};

use crate::enums::{
    image_format::ImageFormatEnum,
    job::{JobScheduleEnum, JobStateEnum},
    schedule::ScheduleTypeEnum,
};

use crate::schema::*;
use diesel::dsl::count_star;

use chrono::NaiveDateTime;

/**
 ==== Supported Operations ====
 == CRUD
- create
- find_by_id
- update
- delete

 == Lookup & Search
- find_by_name
- search_by_job_name
- list_by_admin
- list_by_state
- get_recent_jobs
- get_failed_jobs

 == State Transitions
- mark_running
- mark_succeeded
- mark_failed

 == Scheduling & Readiness
- list_scheduled_jobs
- list_due_cron_jobs
- list_one_time_jobs_ready

 == Aggregation / Stats
- get_job_counts_per_admin

 == Assignment-related
- get_active_jobs_for_worker
- find_jobs_assigned_to_worker
- list_jobs_with_no_assignment

*/
pub struct JobRepository;

impl JobRepository {
    pub async fn find_by_id(c: &mut AsyncPgConnection, id: i32) -> QueryResult<Job> {
        jobs::table.find(id).get_result(c).await
    }

    pub async fn create(c: &mut AsyncPgConnection, new_job: NewJob) -> QueryResult<Job> {
        diesel::insert_into(jobs::table)
            .values(new_job)
            .get_result(c)
            .await
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
        admin_id: i32,
        name: &str,
    ) -> QueryResult<Option<Job>> {
        jobs::table
            .filter(jobs::admin_id.eq(admin_id))
            .filter(jobs::job_name.eq(name))
            .first::<Job>(c)
            .await
            .optional()
    }

    pub async fn list_by_admin(
        c: &mut AsyncPgConnection,
        admin_id: i32,
        limit: i64,
        offset: i64,
    ) -> QueryResult<Vec<Job>> {
        jobs::table
            .filter(jobs::admin_id.eq(admin_id))
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
        aadmin_id: i32,
        query: &str,
    ) -> QueryResult<Vec<Job>> {
        jobs::table
            .filter(jobs::admin_id.eq(aadmin_id))
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
            .group_by(jobs::admin_id) // Call on jobs::table, not jobs::group_by
            .select((jobs::admin_id, count_star()))
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

/**
 ==== Supported Operations ====

 == CRUD
- create
- find_by_id
- delete

 == Lookup & Search
- find_by_job_id
- find_by_worker_id
- find_assignment_by_job_and_worker
- find_assignments_for_worker_in_time_range

 == State Updates
- update_started_at
- update_finished_at

 == Filtering / Status
- list_active_assignments
*/

pub struct JobAssignmentRepository;

impl JobAssignmentRepository {
    // Create a new job assignment
    pub async fn create(
        c: &mut AsyncPgConnection,
        new_assignment: NewJobAssignment,
    ) -> QueryResult<JobAssignment> {
        diesel::insert_into(job_assignments::table)
            .values(new_assignment)
            .get_result(c)
            .await
    }

    // Find a job assignment by its ID
    pub async fn find_by_id(c: &mut AsyncPgConnection, id: i32) -> QueryResult<JobAssignment> {
        job_assignments::table.find(id).get_result(c).await
    }

    // Find all assignments for a specific job
    pub async fn find_by_job_id(
        c: &mut AsyncPgConnection,
        job_id: i32,
    ) -> QueryResult<Vec<JobAssignment>> {
        job_assignments::table
            .filter(job_assignments::job_id.eq(job_id))
            .load(c)
            .await
    }

    // Find all assignments for a specific worker
    pub async fn find_by_worker_id(
        c: &mut AsyncPgConnection,
        worker_id: i32,
    ) -> QueryResult<Vec<JobAssignment>> {
        job_assignments::table
            .filter(job_assignments::worker_id.eq(worker_id))
            .load(c)
            .await
    }

    // Update the 'started_at' field for a job assignment
    pub async fn update_started_at(
        c: &mut AsyncPgConnection,
        id: i32,
        started_at: NaiveDateTime,
    ) -> QueryResult<JobAssignment> {
        diesel::update(job_assignments::table.find(id))
            .set(job_assignments::started_at.eq(started_at))
            .get_result(c)
            .await
    }

    // Update the 'finished_at' field for a job assignment
    pub async fn update_finished_at(
        c: &mut AsyncPgConnection,
        id: i32,
        finished_at: NaiveDateTime,
    ) -> QueryResult<JobAssignment> {
        diesel::update(job_assignments::table.find(id))
            .set(job_assignments::finished_at.eq(finished_at))
            .get_result(c)
            .await
    }

    // Delete a job assignment
    pub async fn delete(c: &mut AsyncPgConnection, id: i32) -> QueryResult<usize> {
        diesel::delete(job_assignments::table.find(id))
            .execute(c)
            .await
    }

    // List all active job assignments (i.e., where finished_at is null)
    pub async fn list_active_assignments(
        c: &mut AsyncPgConnection,
    ) -> QueryResult<Vec<JobAssignment>> {
        job_assignments::table
            .filter(job_assignments::finished_at.is_null())
            .load(c)
            .await
    }

    // Find assignment by job_id and worker_id
    pub async fn find_assignment_by_job_and_worker(
        c: &mut AsyncPgConnection,
        job_id: i32,
        worker_id: i32,
    ) -> QueryResult<Option<JobAssignment>> {
        job_assignments::table
            .filter(job_assignments::job_id.eq(job_id))
            .filter(job_assignments::worker_id.eq(worker_id))
            .first(c)
            .await
            .optional()
    }

    // Find all assignments for a worker within a specific time range
    pub async fn find_assignments_for_worker_in_time_range(
        c: &mut AsyncPgConnection,
        worker_id: i32,
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
    ) -> QueryResult<Vec<JobAssignment>> {
        job_assignments::table
            .filter(job_assignments::worker_id.eq(worker_id))
            .filter(job_assignments::assigned_at.ge(start_time))
            .filter(job_assignments::assigned_at.le(end_time))
            .load(c)
            .await
    }
}

/**
 ==== Supported Operations ====

 == CRUD
- create
- find_by_id
- delete

 == Lookup & Search
- find_by_job_id
- list_results_for_job
- get_most_recent_for_job

 == Field Updates
- update_stdout
- update_files
*/
pub struct JobResultRepository;

impl JobResultRepository {
    pub async fn create(
        c: &mut AsyncPgConnection,
        new_result: NewJobResult,
    ) -> QueryResult<JobResult> {
        diesel::insert_into(job_results::table)
            .values(new_result)
            .get_result(c)
            .await
    }

    pub async fn find_by_id(c: &mut AsyncPgConnection, id: i32) -> QueryResult<JobResult> {
        job_results::table.find(id).get_result(c).await
    }

    pub async fn find_by_job_id(
        c: &mut AsyncPgConnection,
        job_id: i32,
    ) -> QueryResult<Vec<JobResult>> {
        job_results::table
            .filter(job_results::job_id.eq(job_id))
            .load(c)
            .await
    }

    pub async fn update_stdout(
        c: &mut AsyncPgConnection,
        id: i32,
        new_stdout: Option<String>,
    ) -> QueryResult<JobResult> {
        diesel::update(job_results::table.find(id))
            .set(job_results::stdout.eq(new_stdout))
            .get_result(c)
            .await
    }

    pub async fn update_files(
        c: &mut AsyncPgConnection,
        id: i32,
        new_files: Option<Vec<String>>,
    ) -> QueryResult<JobResult> {
        diesel::update(job_results::table.find(id))
            .set(job_results::files.eq(new_files))
            .get_result(c)
            .await
    }

    pub async fn delete(c: &mut AsyncPgConnection, id: i32) -> QueryResult<usize> {
        diesel::delete(job_results::table.find(id)).execute(c).await
    }

    pub async fn list_results_for_job(
        c: &mut AsyncPgConnection,
        job_id: i32,
    ) -> QueryResult<Vec<JobResult>> {
        job_results::table
            .filter(job_results::job_id.eq(job_id))
            .load(c)
            .await
    }

    pub async fn get_most_recent_for_job(
        c: &mut AsyncPgConnection,
        job_id: i32,
    ) -> QueryResult<Option<JobResult>> {
        job_results::table
            .filter(job_results::job_id.eq(job_id))
            .order(job_results::saved_at.desc())
            .first(c)
            .await
            .optional()
    }
}

/**
 ==== Supported Operations ====

 == CRUD
- create
- find_by_id
- delete

 == Lookup & Search
- find_by_job_id
- find_by_worker_id
- get_metrics_by_worker
- list_metrics_for_job
- get_most_recent_for_job
*/
pub struct JobMetricRepository;

impl JobMetricRepository {
    pub async fn create(
        c: &mut AsyncPgConnection,
        new_metric: NewJobMetric,
    ) -> QueryResult<JobMetric> {
        diesel::insert_into(job_metrics::table)
            .values(new_metric)
            .get_result(c)
            .await
    }

    pub async fn find_by_id(c: &mut AsyncPgConnection, id: i32) -> QueryResult<JobMetric> {
        job_metrics::table.find(id).get_result(c).await
    }

    pub async fn find_by_job_id(
        c: &mut AsyncPgConnection,
        job_id: i32,
    ) -> QueryResult<Vec<JobMetric>> {
        job_metrics::table
            .filter(job_metrics::job_id.eq(job_id))
            .load(c)
            .await
    }

    pub async fn find_by_worker_id(
        c: &mut AsyncPgConnection,
        worker_id: i32,
    ) -> QueryResult<Vec<JobMetric>> {
        job_metrics::table
            .filter(job_metrics::worker_id.eq(worker_id))
            .load(c)
            .await
    }

    pub async fn list_metrics_for_job(
        c: &mut AsyncPgConnection,
        job_id: i32,
    ) -> QueryResult<Vec<JobMetric>> {
        job_metrics::table
            .filter(job_metrics::job_id.eq(job_id))
            .order(job_metrics::timestamp.desc())
            .load(c)
            .await
    }

    pub async fn get_most_recent_for_job(
        c: &mut AsyncPgConnection,
        job_id: i32,
    ) -> QueryResult<Option<JobMetric>> {
        job_metrics::table
            .filter(job_metrics::job_id.eq(job_id))
            .order(job_metrics::timestamp.desc())
            .first(c)
            .await
            .optional()
    }

    pub async fn get_metrics_by_worker(
        c: &mut AsyncPgConnection,
        worker_id: i32,
    ) -> QueryResult<Vec<JobMetric>> {
        job_metrics::table
            .filter(job_metrics::worker_id.eq(worker_id))
            .order(job_metrics::timestamp.desc())
            .load(c)
            .await
    }

    pub async fn delete(c: &mut AsyncPgConnection, id: i32) -> QueryResult<usize> {
        diesel::delete(job_metrics::table.find(id)).execute(c).await
    }
}
