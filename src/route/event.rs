use crate::prelude::*;

use crate::config::db::Conn as DbConn;
use rocket_contrib::json::Json;
use serde_json::Value;

#[get("/event", format = "application/json")]
pub fn get_all(conn: DbConn) -> Json<Value> {
    let events = Event::get_all_events(&conn);
    Json(json!(events))
}

#[post("/event", format = "application/json", data = "<new_event>")]
pub fn new_event(conn: DbConn, new_event: Json<NewEvent>) -> Json<Value> {
    Event::insert_event(new_event.into_inner(), &conn);
    Json(json!(Event::get_all_events(&conn).first()))
}

#[get("/event/<event_id>", format = "application/json")]
pub fn get_one(conn: DbConn, event_id: String) -> Json<Value> {
    Json(json!(Event::get_event_by_id(event_id.as_str(), &conn)))
}

