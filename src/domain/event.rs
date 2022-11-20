use diesel;
use diesel::pg::PgConnection;
use diesel::sql_types::{Timestamp, Date};
use uuid::Uuid;

use crate::schema::event::dsl::events as all_events;

#[derive(Serialize, Queryable)]
pub struct Event {
    pub id: Uuid,
    pub last_modified_date: Timestamp,
    pub url: String,
    pub title: String,
    pub description: String,
    pub start_date: Date,
    pub end_date: Date,
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "event"]
pub struct NewEvent {
    pub url: String,
    pub title: String,
    pub description: String,
    pub start_date: Date,
    pub end_date: Date,
}

impl User {
    pub fn get_all_users(conn: &PgConnection) -> Vec<User> {
        all_users
            .order(users::id.desc())
            .load::<User>(conn)
            .expect("error!")
    }

    pub fn insert_user(user: NewUser, conn: &PgConnection) -> bool {
        diesel::insert_into(users::table)
            .values(&user)
            .execute(conn)
            .is_ok()
    }

    pub fn get_user_by_username(username: String, conn: &PgConnection) -> Vec<User> {
        all_users
            .filter(users::username.eq(username))
            .load::<User>(conn)
            .expect("error!")
    }
}

