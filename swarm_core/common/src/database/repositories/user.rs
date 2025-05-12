use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::database::models::user::{NewUser, User};
use crate::database::schema::*;

pub struct UserRepository;

impl UserRepository {
    pub async fn find_by_id(c: &mut AsyncPgConnection, id: i32) -> QueryResult<User> {
        users::table.find(id).get_result(c).await
    }

    pub async fn find_by_email(
        c: &mut AsyncPgConnection,
        email: &str,
    ) -> QueryResult<Option<User>> {
        users::table
            .filter(users::email.eq(email))
            .first::<User>(c)
            .await
            .optional()
    }

    pub async fn find_by_username(
        c: &mut AsyncPgConnection,
        username: &str,
    ) -> QueryResult<Option<User>> {
        users::table
            .filter(users::username.eq(username))
            .first::<User>(c)
            .await
            .optional()
    }

    pub async fn create(c: &mut AsyncPgConnection, new_user: NewUser) -> QueryResult<User> {
        diesel::insert_into(users::table)
            .values(new_user)
            .get_result(c)
            .await
    }

    pub async fn delete(c: &mut AsyncPgConnection, id: i32) -> QueryResult<usize> {
        diesel::delete(users::table.find(id)).execute(c).await
    }

    pub async fn update(c: &mut AsyncPgConnection, id: i32, user: User) -> QueryResult<User> {
        diesel::update(users::table.find(id))
            .set((
                users::username.eq(user.username),
                users::email.eq(user.email),
                users::password_hash.eq(user.password_hash),
            ))
            .get_result(c)
            .await
    }

    pub async fn search_by_username(
        c: &mut AsyncPgConnection,
        query: &str,
    ) -> QueryResult<Vec<User>> {
        users::table
            .filter(users::username.ilike(format!("%{}%", query)))
            .load(c)
            .await
    }

    pub async fn search_by_email(c: &mut AsyncPgConnection, query: &str) -> QueryResult<Vec<User>> {
        users::table
            .filter(users::email.ilike(format!("%{}%", query)))
            .load(c)
            .await
    }

    pub async fn list_all(
        c: &mut AsyncPgConnection,
        limit: i64,
        offset: i64,
    ) -> QueryResult<Vec<User>> {
        users::table.limit(limit).offset(offset).load(c).await
    }

    pub async fn exists_by_email(c: &mut AsyncPgConnection, email: &str) -> QueryResult<bool> {
        let count: i64 = users::table
            .filter(users::email.eq(email))
            .count()
            .get_result(c)
            .await?;
        Ok(count > 0)
    }

    pub async fn exists_by_username(
        c: &mut AsyncPgConnection,
        username: &str,
    ) -> QueryResult<bool> {
        let count: i64 = users::table
            .filter(users::username.eq(username))
            .count()
            .get_result(c)
            .await?;
        Ok(count > 0)
    }

    pub async fn find_users_with_jobs(c: &mut AsyncPgConnection) -> QueryResult<Vec<User>> {
        users::table
            .inner_join(jobs::table.on(jobs::user_id.eq(users::id)))
            .select(users::all_columns)
            .distinct()
            .load(c)
            .await
    }

    pub async fn get_user_with_job_counts(
        c: &mut AsyncPgConnection,
    ) -> QueryResult<Vec<(User, i64)>> {
        users::table
            .left_outer_join(jobs::table.on(jobs::user_id.eq(users::id)))
            .select((
                users::all_columns,
                diesel::dsl::sql::<diesel::sql_types::BigInt>("COUNT(jobs.id)"),
            ))
            .group_by(users::id)
            .load(c)
            .await
    }
}
