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
        user_id: i32,
    ) -> QueryResult<Vec<Worker>> {
        workers::table
            .filter(workers::user_id.eq(user_id))
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
        user_id: i32,
        limit: i64,
        offset: i64,
    ) -> QueryResult<Vec<Worker>> {
        workers::table
            .filter(workers::user_id.eq(user_id))
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
