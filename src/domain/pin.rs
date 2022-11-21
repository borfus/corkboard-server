use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;
use chrono::NaiveDateTime;

use crate::schema::pin;
use crate::schema::pin::dsl::pin as all_pins;

#[derive(Serialize, Queryable)]
pub struct Pin {
    pub id: Uuid,
    pub last_modified_date: NaiveDateTime,
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "pin"]
pub struct NewPin {
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>
}

impl Pin {
    pub fn get_all_pins(conn: &PgConnection) -> Vec<Pin> {
        all_pins
            .order(pin::id.desc())
            .load::<Pin>(conn)
            .expect("error!")
    }

    pub fn insert_pin(pin: NewPin, conn: &PgConnection) -> bool {
        diesel::insert_into(pin::table)
            .values(&pin)
            .execute(conn)
            .is_ok()
    }

    pub fn get_pin_by_id(pin_id: &str, conn: &PgConnection) -> Vec<Pin> {
        all_pins
            .filter(pin::id.eq(Uuid::parse_str(pin_id).unwrap()))
            .load::<Pin>(conn)
            .expect("error!")
    }
}

