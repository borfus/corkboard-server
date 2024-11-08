use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::{Duration, Utc};
use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

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
    pub end_date: Option<NaiveDateTime>,
    pub guild_id: Option<i64>,
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "event"]
pub struct NewEvent {
    pub guild_id: Option<i64>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
}

impl Event {
    pub fn get_all_events(guild_id: i64, conn: &PgConnection) -> Vec<Event> {
        all_events
            .filter(event::guild_id.eq(Some(guild_id)))
            .order(event::id.desc())
            .load::<Event>(conn)
            .expect("Error occurred while attempting to get all events!")
    }

    pub fn get_current_events(guild_id: i64, conn: &PgConnection) -> Vec<Event> {
        let mut result = all_events
            .filter(event::guild_id.eq(Some(guild_id)))
            .order(event::end_date)
            .load::<Event>(conn)
            .expect("Error occurred while attempting to get all current events!");

        let now = Utc::now().naive_utc() - Duration::hours(8);
        result.retain(|e| e.start_date.unwrap() <= now && e.end_date.unwrap() >= now);

        result
    }

    pub fn insert_event(event: NewEvent, conn: &PgConnection) -> Event {
        let new_event = diesel::insert_into(event::table)
            .values(&event)
            .get_result::<Event>(conn)
            .unwrap();
        new_event
    }

    pub fn get_event_by_id(event_id: &str, conn: &PgConnection) -> Vec<Event> {
        all_events
            .filter(event::id.eq(Uuid::parse_str(event_id).expect("Invalid Event ID!")))
            .load::<Event>(conn)
            .expect(
                format!(
                    "Error occurred while attempting to get event with ID {}",
                    event_id
                )
                .as_str(),
            )
    }

    pub fn update_by_id(event_id: &str, new_event: NewEvent, conn: &PgConnection) -> Event {
        let updated_event = diesel::update(
            all_events.filter(event::id.eq(Uuid::parse_str(event_id).expect("Invalid Event ID!"))),
        )
        .set((
            event::last_modified_date.eq(DateTime::from_timestamp_millis(
                Utc::now().timestamp_millis(),
            )
            .unwrap()
            .naive_utc()),
            event::title.eq(new_event.title),
            event::url.eq(new_event.url),
            event::description.eq(new_event.description),
            event::start_date.eq(new_event.start_date),
            event::end_date.eq(new_event.end_date),
        ))
        .get_result::<Event>(conn)
        .expect(format!("Unabled to update event with ID {}", event_id).as_str());

        updated_event
    }

    pub fn delete_by_id(event_id: &str, conn: &PgConnection) -> Event {
        let deleted_event = diesel::delete(
            all_events.filter(event::id.eq(Uuid::parse_str(event_id).expect("Invalid Event ID!"))),
        )
        .get_result(conn)
        .expect(format!("Unabled to delete event with ID {}", event_id).as_str());

        deleted_event
    }
}
