use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::database::models::job::{JobMetric, NewJobMetric};
use crate::database::schema::job_metrics;

pub struct JobMetricRepository;

impl JobMetricRepository {
    pub async fn create(
        c: &mut AsyncPgConnection,
        new_metric: NewJobMetric,
    ) -> QueryResult<JobMetric> {
        use crate::database::schema::job_metrics::dsl::*;

        diesel::insert_into(job_metrics)
            .values(&new_metric)
            .on_conflict((job_id, worker_id))
            .do_update()
            .set((
                duration_sec.eq(new_metric.duration_sec),
                cpu_usage_pct.eq(new_metric.cpu_usage_pct),
                mem_usage_mb.eq(new_metric.mem_usage_mb),
                exit_code.eq(new_metric.exit_code),
                timestamp.eq(Utc::now().naive_utc()),
            ))
            .get_result(c)
            .await
    }

    pub async fn find_by_id(c: &mut AsyncPgConnection, id: i32) -> QueryResult<JobMetric> {
        job_metrics::table.find(id).get_result(c).await
    }

    pub async fn find_by_job_id(
        c: &mut AsyncPgConnection,
        job_id_val: i32,
    ) -> QueryResult<JobMetric> {
        use crate::database::schema::job_metrics::dsl::*;

        job_metrics
            .filter(job_id.eq(job_id_val))
            .first::<JobMetric>(c)
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
