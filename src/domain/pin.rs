use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::pin;
use crate::schema::pin::dsl::pin as all_pins;

#[derive(Serialize, Queryable)]
pub struct Pin {
    pub id: Uuid,
    pub last_modified_date: NaiveDateTime,
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub guild_id: Option<i64>,
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "pin"]
pub struct NewPin {
    pub guild_id: Option<i64>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
}

impl Pin {
    pub fn get_all_pins(guild_id: i64, conn: &PgConnection) -> Vec<Pin> {
        all_pins
            .filter(pin::guild_id.eq(Some(guild_id)))
            .order(pin::id.desc())
            .load::<Pin>(conn)
            .expect("error!")
    }

    pub fn insert_pin(pin: NewPin, conn: &PgConnection) -> Pin {
        let new_pin = diesel::insert_into(pin::table)
            .values(&pin)
            .get_result::<Pin>(conn)
            .unwrap();
        new_pin
    }

    pub fn get_pin_by_id(pin_id: &str, guild_id: i64, conn: &PgConnection) -> Vec<Pin> {
        all_pins
            .filter(pin::id.eq(Uuid::parse_str(pin_id).unwrap()))
            .filter(pin::guild_id.eq(Some(guild_id)))
            .load::<Pin>(conn)
            .expect("error!")
    }

    pub fn update_by_id(pin_id: &str, new_pin: NewPin, conn: &PgConnection) -> Pin {
        let updated_pin = diesel::update(
            all_pins.filter(pin::id.eq(Uuid::parse_str(pin_id).expect("Invalid Pin ID!"))),
        )
        .set((
            pin::last_modified_date.eq(DateTime::from_timestamp_millis(
                Utc::now().timestamp_millis(),
            )
            .unwrap()
            .naive_utc()),
            pin::title.eq(new_pin.title),
            pin::url.eq(new_pin.url),
            pin::description.eq(new_pin.description),
        ))
        .get_result::<Pin>(conn)
        .expect(format!("Unabled to update pin with ID {}", pin_id).as_str());

        updated_pin
    }

    pub fn delete_by_id(pin_id: &str, conn: &PgConnection) -> Pin {
        let deleted_pin = diesel::delete(
            all_pins.filter(pin::id.eq(Uuid::parse_str(pin_id).expect("Invalid Pin ID!"))),
        )
        .get_result(conn)
        .expect(format!("Unabled to delete pin with ID {}", pin_id).as_str());

        deleted_pin
    }
}
