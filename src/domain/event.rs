use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;
use chrono::NaiveDateTime;

use crate::schema::event;
use crate::schema::event::dsl::event as all_events;

#[derive(Serialize, Queryable)]
pub struct Event {
    pub id: Uuid,
    pub last_modified_date: NaiveDateTime,
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "event"]
pub struct NewEvent {
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>
}

impl Event {
    pub fn get_all_events(conn: &PgConnection) -> Vec<Event> {
        all_events
            .order(event::id.desc())
            .load::<Event>(conn)
            .expect("error!")
    }

    pub fn insert_event(event: NewEvent, conn: &PgConnection) -> bool {
        diesel::insert_into(event::table)
            .values(&event)
            .execute(conn)
            .is_ok()
    }

    pub fn get_event_by_id(event_id: &str, conn: &PgConnection) -> Vec<Event> {
        all_events
            .filter(event::id.eq(Uuid::parse_str(event_id).unwrap()))
            .load::<Event>(conn)
            .expect("error!")
    }
}

