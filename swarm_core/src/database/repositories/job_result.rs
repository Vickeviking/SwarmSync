/*
==============================
🛠️ Supported Actions JobResult
==============================

== CRUD ==
• `POST /results`        → create(NewJobResult) -> JobResult
• `GET /results/:id`     → find_by_id(id) -> JobResult
• `DELETE /results/:id`  → delete(id) -> usize

== Lookup & Search ==
• `GET /results/job/:job_id`         → find_by_job_id(job_id) -> Vec<JobResult>
• `GET /results/list/:job_id`        → list_results_for_job(job_id) -> Vec<JobResult>
• `GET /results/recent/:job_id`      → get_most_recent_for_job(job_id) -> Option<JobResult>

== Field Updates ==
• `PATCH /results/:id/stdout`        → update_stdout(id, new_stdout) -> JobResult
• `PATCH /results/:id/files`         → update_files(id, new_files) -> JobResult
*/

use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::database::models::job::{
    JobResult, NewJobResult,
};


use crate::database::schema::*;


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
