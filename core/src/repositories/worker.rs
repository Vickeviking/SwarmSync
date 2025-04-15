use diesel::dsl::now;
use diesel::dsl::IntervalDsl;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::enums::workers::WorkerStatusEnum;
use crate::models::worker::{NewWorker, NewWorkerStatus, Worker, WorkerStatus};
use crate::schema::*;
use chrono::NaiveDateTime;

/**
 ==== Supported Operations ====

 == CRUD
- create
- find_by_id
- delete_worker

 == Lookup & Search
- find_by_admin_id
- find_by_label
- find_by_ip_address
- list_workers_by_admin

 == State Update
- update_last_seen_at
*/

pub struct WorkerRepository;

impl WorkerRepository {
    pub async fn create(c: &mut AsyncPgConnection, new_worker: NewWorker) -> QueryResult<Worker> {
        diesel::insert_into(workers::table)
            .values(new_worker)
            .get_result(c)
            .await
    }

    pub async fn find_by_id(c: &mut AsyncPgConnection, id: i32) -> QueryResult<Worker> {
        workers::table.find(id).get_result(c).await
    }

    pub async fn find_by_admin_id(
        c: &mut AsyncPgConnection,
        admin_id: i32,
    ) -> QueryResult<Vec<Worker>> {
        workers::table
            .filter(workers::admin_id.eq(admin_id))
            .load(c)
            .await
    }

    pub async fn find_by_label(
        c: &mut AsyncPgConnection,
        label: &str,
    ) -> QueryResult<Option<Worker>> {
        workers::table
            .filter(workers::label.eq(label))
            .first::<Worker>(c)
            .await
            .optional()
    }

    pub async fn find_by_ip_address(
        c: &mut AsyncPgConnection,
        ip_address: &str,
    ) -> QueryResult<Option<Worker>> {
        workers::table
            .filter(workers::ip_address.eq(ip_address))
            .first::<Worker>(c)
            .await
            .optional()
    }

    pub async fn list_workers_by_admin(
        c: &mut AsyncPgConnection,
        admin_id: i32,
        limit: i64,
        offset: i64,
    ) -> QueryResult<Vec<Worker>> {
        workers::table
            .filter(workers::admin_id.eq(admin_id))
            .limit(limit)
            .offset(offset)
            .order(workers::created_at.desc())
            .load(c)
            .await
    }

    pub async fn update_last_seen_at(
        c: &mut AsyncPgConnection,
        id: i32,
        last_seen_at: NaiveDateTime,
    ) -> QueryResult<Worker> {
        diesel::update(workers::table.find(id))
            .set(workers::last_seen_at.eq(last_seen_at))
            .get_result(c)
            .await
    }

    pub async fn delete_worker(c: &mut AsyncPgConnection, id: i32) -> QueryResult<usize> {
        diesel::delete(workers::table.find(id)).execute(c).await
    }
}

/**
 ==== Supported Operations ====

 == CRUD
- create
- find_by_id
- delete_worker_status

 == Lookup & Search
- find_by_worker_id

 == State Update
- update_status
- update_last_heartbeat
- update_active_job_id
- update_uptime
- update_load_avg
- update_last_error
*/

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
        last_heartbeat: NaiveDateTime,
    ) -> QueryResult<WorkerStatus> {
        diesel::update(worker_status::table.find(id))
            .set(worker_status::last_heartbeat.eq(last_heartbeat))
            .get_result(c)
            .await
    }

    pub async fn update_active_job_id(
        c: &mut AsyncPgConnection,
        id: i32,
        active_job_id: Option<i32>,
    ) -> QueryResult<WorkerStatus> {
        diesel::update(worker_status::table.find(id))
            .set(worker_status::active_job_id.eq(active_job_id))
            .get_result(c)
            .await
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
}
