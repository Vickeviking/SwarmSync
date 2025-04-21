use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::database::models::job::{
    JobAssignment, NewJobAssignment,
};


use crate::database::schema::*;


use chrono::NaiveDateTime;

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
