/*
==============================
🛠️ Supported Actions JobMetric
==============================

== CRUD ==
• `POST /metrics`        → create(NewJobMetric) -> JobMetric
• `GET /metrics/:id`     → find_by_id(id) -> JobMetric
• `DELETE /metrics/:id`  → delete(id) -> usize

== Lookup & Search ==
• `GET /metrics/by_job/:job_id`          → find_by_job_id(job_id) -> Vec<JobMetric>
• `GET /metrics/by_worker/:worker_id`    → find_by_worker_id(worker_id) -> Vec<JobMetric>
• `GET /metrics/recent/:job_id`          → get_most_recent_for_job(job_id) -> Option<JobMetric>
• `GET /metrics/chronological/:job_id`   → list_metrics_for_job(job_id) -> Vec<JobMetric>
• `GET /metrics/worker_stream/:worker_id` → get_metrics_by_worker(worker_id) -> Vec<JobMetric>
*/

use diesel::dsl::now;
use diesel::dsl::IntervalDsl;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::database::models::job::{
    Job, JobAssignment, JobMetric, JobResult, NewJob, NewJobAssignment, NewJobMetric, NewJobResult,
};

use crate::shared::enums::{
    image_format::ImageFormatEnum,
    job::{JobScheduleEnum, JobStateEnum},
    schedule::ScheduleTypeEnum,
};

use crate::database::schema::*;
use diesel::dsl::count_star;

use chrono::NaiveDateTime;

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
