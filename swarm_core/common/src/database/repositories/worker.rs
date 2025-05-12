use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use chrono::NaiveDateTime;

use crate::database::models::worker::{NewWorker, Worker};
use crate::database::schema::*;

pub struct WorkerRepository;

impl WorkerRepository {
    pub async fn create(c: &mut AsyncPgConnection, new_worker: NewWorker) -> QueryResult<Worker> {
        diesel::insert_into(workers::table)
            .values(new_worker)
            .get_result(c)
            .await
    }

    pub async fn update(c: &mut AsyncPgConnection, id: i32, worker: Worker) -> QueryResult<Worker> {
        diesel::update(workers::table.find(id))
            .set((
                workers::label.eq(worker.label),
                workers::ip_address.eq(worker.ip_address),
                workers::hostname.eq(worker.hostname),
                workers::ssh_user.eq(worker.ssh_user),
                workers::ssh_key.eq(worker.ssh_key),
                workers::docker_version.eq(worker.docker_version),
                workers::arch.eq(worker.arch),
                workers::os.eq(worker.os),
                workers::tags.eq(worker.tags),
                workers::last_seen_at.eq(worker.last_seen_at),
            ))
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

    pub async fn list_all(c: &mut AsyncPgConnection) -> QueryResult<Vec<Worker>> {
        workers::table.load(c).await
    }
}
