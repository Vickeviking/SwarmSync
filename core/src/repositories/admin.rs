use crate::models::admin::{Admin, NewAdmin};
use crate::models::*;
use crate::schema::*;
use diesel::dsl::now;
use diesel::dsl::IntervalDsl;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

/* ==== AdminRepository Operations ====

Basic CRUD:
- find_by_id(id: i32) -> Admin
- find_by_email(email: &str) -> Option<Admin>
- find_by_username(username: &str) -> Option<Admin>
- create(new_admin: NewAdmin) -> Admin
- delete(id: i32) -> usize
- update(id: i32, admin: Admin) -> Admin

Search:
- search_by_username(query: &str) -> Vec<Admin>
- search_by_email(query: &str) -> Vec<Admin>

Listing:
- list_all(limit: i64, offset: i64) -> Vec<Admin>

Existence Checks:
- exists_by_email(email: &str) -> bool
- exists_by_username(username: &str) -> bool

Relational & Aggregation:
- find_admins_with_jobs() -> Vec<Admin>
- get_admin_with_job_counts() -> Vec<(Admin, i64)>
*/

pub struct AdminRepository;

impl AdminRepository {
    pub async fn find_by_id(c: &mut AsyncPgConnection, id: i32) -> QueryResult<Admin> {
        admins::table.find(id).get_result(c).await
    }

    pub async fn find_by_email(
        c: &mut AsyncPgConnection,
        email: &str,
    ) -> QueryResult<Option<Admin>> {
        admins::table
            .filter(admins::email.eq(email))
            .first::<Admin>(c)
            .await
            .optional()
    }

    pub async fn find_by_username(
        c: &mut AsyncPgConnection,
        username: &str,
    ) -> QueryResult<Option<Admin>> {
        admins::table
            .filter(admins::username.eq(username))
            .first::<Admin>(c)
            .await
            .optional()
    }

    pub async fn create(c: &mut AsyncPgConnection, new_admin: NewAdmin) -> QueryResult<Admin> {
        diesel::insert_into(admins::table)
            .values(new_admin)
            .get_result(c)
            .await
    }

    pub async fn delete(c: &mut AsyncPgConnection, id: i32) -> QueryResult<usize> {
        diesel::delete(admins::table.find(id)).execute(c).await
    }

    pub async fn update(c: &mut AsyncPgConnection, id: i32, admin: Admin) -> QueryResult<Admin> {
        diesel::update(admins::table.find(id))
            .set((
                admins::username.eq(admin.username),
                admins::email.eq(admin.email),
                admins::password_hash.eq(admin.password_hash),
            ))
            .get_result(c)
            .await
    }

    pub async fn search_by_username(
        c: &mut AsyncPgConnection,
        query: &str,
    ) -> QueryResult<Vec<Admin>> {
        admins::table
            .filter(admins::username.ilike(format!("%{}%", query)))
            .load(c)
            .await
    }

    pub async fn search_by_email(
        c: &mut AsyncPgConnection,
        query: &str,
    ) -> QueryResult<Vec<Admin>> {
        admins::table
            .filter(admins::email.ilike(format!("%{}%", query)))
            .load(c)
            .await
    }

    pub async fn list_all(
        c: &mut AsyncPgConnection,
        limit: i64,
        offset: i64,
    ) -> QueryResult<Vec<Admin>> {
        admins::table.limit(limit).offset(offset).load(c).await
    }

    pub async fn exists_by_email(c: &mut AsyncPgConnection, email: &str) -> QueryResult<bool> {
        let count: i64 = admins::table
            .filter(admins::email.eq(email))
            .count()
            .get_result(c)
            .await?;
        Ok(count > 0)
    }

    pub async fn exists_by_username(
        c: &mut AsyncPgConnection,
        username: &str,
    ) -> QueryResult<bool> {
        let count: i64 = admins::table
            .filter(admins::username.eq(username))
            .count()
            .get_result(c)
            .await?;
        Ok(count > 0)
    }

    pub async fn find_admins_with_jobs(c: &mut AsyncPgConnection) -> QueryResult<Vec<Admin>> {
        admins::table
            .inner_join(jobs::table.on(jobs::admin_id.eq(admins::id)))
            .select(admins::all_columns)
            .distinct()
            .load(c)
            .await
    }

    pub async fn get_admin_with_job_counts(
        c: &mut AsyncPgConnection,
    ) -> QueryResult<Vec<(Admin, i64)>> {
        admins::table
            .left_outer_join(jobs::table.on(jobs::admin_id.eq(admins::id)))
            .select((
                admins::all_columns,
                diesel::dsl::sql::<diesel::sql_types::BigInt>("COUNT(jobs.id)"),
            ))
            .group_by(admins::id)
            .load(c)
            .await
    }
}
