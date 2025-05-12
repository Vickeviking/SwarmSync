use chrono::NaiveDateTime;
use diesel::dsl::count_star;
use diesel::dsl::now;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::database::models::job::{Job, NewJob};
use crate::database::schema::*;
use crate::enums::{job::JobStateEnum, schedule::ScheduleTypeEnum};

/// Job repository, functions for interacting with the database
pub struct JobRepository;

impl JobRepository {
    /// Find a job by id from postgres
    /// # Arguments
    /// * `c` - The database connection
    /// * `id` - The id of the job
    /// # Returns
    /// * `QueryResult<Job>`
    /// # Example
    /// ```
    /// use swarm_core::database::repositories::job::JobRepository;
    /// use swarm_core::database::models::job::Job;
    ///
    /// let mut c = load_db_connection().await?;
    /// let job: Job = JobRepository::find_by_id(&mut c, 1).await?;
    /// println!("Job: {}", job.job_name);
    /// ```
    /// # Panics
    /// Panics if the query fails, or if database connection fails
    pub async fn find_by_id(c: &mut AsyncPgConnection, id: i32) -> QueryResult<Job> {
        jobs::table.find(id).get_result(c).await
    }

    /// Create a new job in the database
    /// # Arguments
    /// * `c` - The database connection
    /// * `new_job` - The new job to create
    /// # Returns
    /// * `QueryResult<Job>`
    /// # Example
    /// ```
    /// use swarm_core::database::repositories::job::JobRepository;
    /// use swarm_core::database::models::job::NewJob;
    /// use swarm_core::database::models::job::Job;
    ///
    /// let mut c = load_db_connection().await?;
    /// let new_job = NewJob {...};
    /// let job: Job = JobRepository::create(&mut c, new_job).await?;
    /// println!("Job: {}", job.job_name);
    /// ```
    /// # Panics
    /// Panics if the query fails, or if database connection fails
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

    /// Update a job in the database
    /// # Arguments
    /// * `c` - The database connection
    /// * `id` - The id of the job
    /// * `job` - The job to update
    /// # Returns
    /// * `QueryResult<Job>`
    /// # Example
    /// ```
    /// use swarm_core::database::repositories::job::JobRepository;
    /// use swarm_core::database::models::job::Job;
    ///
    /// let mut c = load_db_connection().await?;
    /// let job = Job {...};
    /// let job: Job = JobRepository::update(&mut c, 1, job).await?;
    /// println!("Job: {}", job.job_name);
    /// ```
    /// # Panics
    /// Panics if the query fails, or if database connection fails
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

    /// Delete a job from the database
    /// # Arguments
    /// * `c` - The database connection
    /// * `id` - The id of the job
    /// # Returns
    /// * `QueryResult<usize>`
    pub async fn delete(c: &mut AsyncPgConnection, id: i32) -> QueryResult<usize> {
        diesel::delete(jobs::table.find(id)).execute(c).await
    }

    /// Find a job by name
    /// # Arguments
    /// * `c` - The database connection
    /// * `user_id` - The id of the user
    /// * `name` - The name of the job
    /// # Returns
    /// * `QueryResult<Option<Job>>`
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

    /// List all jobs for a user
    /// # Arguments
    /// * `c` - The database connection
    /// * `user_id` - The id of the user
    /// * `limit` - The number of jobs to return
    /// * `offset` - The number of jobs to skip
    /// # Returns
    /// * `QueryResult<Vec<Job>>`
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

    /// List all scheduled jobs
    /// # Arguments
    /// * `c` - The database connection
    /// # Returns
    /// * `QueryResult<Vec<Job>>`
    pub async fn list_scheduled_jobs(c: &mut AsyncPgConnection) -> QueryResult<Vec<Job>> {
        jobs::table
            .filter(jobs::state.eq(JobStateEnum::Queued))
            .order(jobs::created_at.desc())
            .load(c)
            .await
    }

    /// Search for jobs by name
    /// # Arguments
    /// * `c` - The database connection
    /// * `user_id` - The id of the user
    /// * `query` - The query to search for, should match any part of the job name
    /// # Returns
    /// * `QueryResult<Vec<Job>>`
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

    /// List all jobs by state
    /// # Arguments
    /// * `c` - The database connection
    /// * `state` - The state of the job
    /// # Returns
    /// * `QueryResult<Vec<Job>>`
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

    /// Mark a job as failed
    /// # Arguments
    /// * `c` - The database connection
    /// * `id` - The id of the job
    /// * `message` - The error message
    /// # Returns
    /// * `QueryResult<Job>`
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

    /// Mark a job as succeeded
    /// # Arguments
    /// * `c` - The database connection
    /// * `id` - The id of the job
    /// # Returns
    /// * `QueryResult<Job>`
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

    /// Mark a job as running
    /// # Arguments
    /// * `c` - The database connection
    /// * `id` - The id of the job
    /// # Returns
    /// * `QueryResult<Job>`
    pub async fn mark_running(c: &mut AsyncPgConnection, id: i32) -> QueryResult<Job> {
        diesel::update(jobs::table.find(id))
            .set((
                jobs::state.eq(JobStateEnum::Running),
                jobs::updated_at.eq(now),
            ))
            .get_result(c)
            .await
    }

    /// Mark a job as submitted
    /// # Arguments
    /// * `c` - The database connection
    /// * `id` - The id of the job
    /// # Returns
    /// * `QueryResult<Job>`
    pub async fn mark_submitted(c: &mut AsyncPgConnection, id: i32) -> QueryResult<Job> {
        diesel::update(jobs::table.find(id))
            .set((
                jobs::state.eq(JobStateEnum::Submitted),
                jobs::updated_at.eq(now),
            ))
            .get_result(c)
            .await
    }

    /// Mark a job as queued
    /// # Arguments
    /// * `c` - The database connection
    /// * `id` - The id of the job
    /// # Returns
    /// * `QueryResult<Job>`
    pub async fn mark_queued(c: &mut AsyncPgConnection, id: i32) -> QueryResult<Job> {
        diesel::update(jobs::table.find(id))
            .set((
                jobs::state.eq(JobStateEnum::Queued),
                jobs::updated_at.eq(now),
            ))
            .get_result(c)
            .await
    }

    /// List all cron jobs that are due
    /// # Arguments
    /// * `c` - The database connection
    /// * `current_time` - The current time
    /// # Returns
    /// * `QueryResult<Vec<Job>>`
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

    /// List all one time jobs that are ready
    /// # Arguments
    /// * `c` - The database connection
    /// # Returns
    /// * `QueryResult<Vec<Job>>`
    pub async fn list_one_time_jobs_ready(c: &mut AsyncPgConnection) -> QueryResult<Vec<Job>> {
        jobs::table
            .filter(jobs::schedule_type.eq(ScheduleTypeEnum::Once))
            .filter(jobs::state.eq(JobStateEnum::Queued))
            .load(c)
            .await
    }

    /// Get the count of jobs per admin
    /// # Arguments
    /// * `c` - The database connection
    /// # Returns
    /// * `QueryResult<Vec<(i32, i64)>>`
    pub async fn get_job_counts_per_admin(
        c: &mut AsyncPgConnection,
    ) -> QueryResult<Vec<(i32, i64)>> {
        jobs::table
            .group_by(jobs::user_id) // Call on jobs::table, not jobs::group_by
            .select((jobs::user_id, count_star()))
            .load::<(i32, i64)>(c)
            .await
    }

    /// Get recent jobs
    /// # Arguments
    /// * `c` - The database connection
    /// * `limit` - The limit of jobs to return
    /// # Returns
    /// * `QueryResult<Vec<Job>>`
    pub async fn get_recent_jobs(c: &mut AsyncPgConnection, limit: i64) -> QueryResult<Vec<Job>> {
        jobs::table
            .order(jobs::created_at.desc())
            .limit(limit)
            .load(c)
            .await
    }

    /// Get failed jobs
    /// # Arguments
    /// * `c` - The database connection
    /// * `limit` - The limit of jobs to return
    /// # Returns
    /// * `QueryResult<Vec<Job>>`
    pub async fn get_failed_jobs(c: &mut AsyncPgConnection, limit: i64) -> QueryResult<Vec<Job>> {
        jobs::table
            .filter(jobs::state.eq(JobStateEnum::Failed))
            .order(jobs::updated_at.desc())
            .limit(limit)
            .load(c)
            .await
    }

    /// Get active jobs for a worker
    /// # Arguments
    /// * `c` - The database connection
    /// * `worker_id` - The id of the worker
    /// # Returns
    /// * `QueryResult<Vec<Job>>`
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

    /// Get jobs assigned to a worker
    /// # Arguments
    /// * `c` - The database connection
    /// * `worker_id` - The id of the worker
    /// # Returns
    /// * `QueryResult<Vec<Job>>`
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

    /// Get jobs with no assignment
    /// # Arguments
    /// * `c` - The database connection
    /// # Returns
    /// * `QueryResult<Vec<Job>>`
    pub async fn list_jobs_with_no_assignment(c: &mut AsyncPgConnection) -> QueryResult<Vec<Job>> {
        jobs::table
            .left_outer_join(job_assignments::table.on(job_assignments::job_id.eq(jobs::id)))
            .filter(job_assignments::job_id.is_null())
            .select(jobs::all_columns)
            .load(c)
            .await
    }
}
