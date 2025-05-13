use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use chrono::Utc;

use crate::database::models::worker::{NewWorkerStatus, WorkerStatus};
use crate::database::schema::*;
use crate::enums::workers::WorkerStatusEnum;

pub struct WorkerStatusRepository;

impl WorkerStatusRepository {
    pub async fn create(
        c: &mut AsyncPgConnection,
        new_worker_status: NewWorkerStatus,
    ) -> QueryResult<WorkerStatus> {
        diesel::insert_into(worker_status::table)
            .values(new_worker_status)
            .get_result(c)
            .await
    }

    pub async fn find_by_id(c: &mut AsyncPgConnection, id: i32) -> QueryResult<WorkerStatus> {
        worker_status::table.find(id).get_result(c).await
    }

    pub async fn find_by_worker_id(
        c: &mut AsyncPgConnection,
        worker_id: i32,
    ) -> QueryResult<Option<WorkerStatus>> {
        worker_status::table
            .filter(worker_status::worker_id.eq(worker_id))
            .first::<WorkerStatus>(c)
            .await
            .optional()
    }

    pub async fn update_status(
        c: &mut AsyncPgConnection,
        id: i32,
        status: WorkerStatusEnum,
    ) -> QueryResult<WorkerStatus> {
        diesel::update(worker_status::table.find(id))
            .set(worker_status::status.eq(status))
            .get_result(c)
            .await
    }

    pub async fn update_last_heartbeat(
        c: &mut AsyncPgConnection,
        id: i32,
    ) -> QueryResult<WorkerStatus> {
        diesel::update(worker_status::table.find(id))
            .set(worker_status::last_heartbeat.eq(Utc::now().naive_utc()))
            .get_result(c)
            .await
    }

    pub async fn update_active_job_id(
        c: &mut AsyncPgConnection,
        id: i32,
        active_job_id: Option<i32>,
    ) -> QueryResult<WorkerStatus> {
        let query = diesel::update(worker_status::table.find(id))
            .set(worker_status::active_job_id.eq(active_job_id))
            .get_result::<WorkerStatus>(c);

        match query.await {
            Ok(res) => Ok(res),
            Err(e) => {
                eprintln!("🔥 Diesel error while updating active_job_id: {}", e);
                Err(e)
            }
        }
    }

    pub async fn update_uptime(
        c: &mut AsyncPgConnection,
        id: i32,
        uptime_sec: Option<i32>,
    ) -> QueryResult<WorkerStatus> {
        diesel::update(worker_status::table.find(id))
            .set(worker_status::uptime_sec.eq(uptime_sec))
            .get_result(c)
            .await
    }

    pub async fn update_load_avg(
        c: &mut AsyncPgConnection,
        id: i32,
        load_avg: Option<Vec<f32>>,
    ) -> QueryResult<WorkerStatus> {
        diesel::update(worker_status::table.find(id))
            .set(worker_status::load_avg.eq(load_avg))
            .get_result(c)
            .await
    }

    pub async fn update_last_error(
        c: &mut AsyncPgConnection,
        id: i32,
        last_error: Option<String>,
    ) -> QueryResult<WorkerStatus> {
        diesel::update(worker_status::table.find(id))
            .set(worker_status::last_error.eq(last_error))
            .get_result(c)
            .await
    }

    pub async fn delete_worker_status(c: &mut AsyncPgConnection, id: i32) -> QueryResult<usize> {
        diesel::delete(worker_status::table.find(id))
            .execute(c)
            .await
    }

    pub async fn update_status_by_worker_id(
        c: &mut AsyncPgConnection,
        worker_id: i32,
        status: WorkerStatusEnum,
    ) -> QueryResult<WorkerStatus> {
        diesel::update(worker_status::table.filter(worker_status::worker_id.eq(worker_id)))
            .set(worker_status::status.eq(status))
            .get_result(c)
            .await
    }
}
