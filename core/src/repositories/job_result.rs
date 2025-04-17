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
