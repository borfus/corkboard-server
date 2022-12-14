use crate::prelude::*;

use crate::config::db::Conn as DbConn;
use rocket_contrib::json::Json;
use serde_json::Value;

#[get("/event/guild/<path_guild_id>", format = "application/json")]
pub fn get_all(conn: DbConn, path_guild_id: String) -> Json<Value> {
    let events = Event::get_all_events(path_guild_id.as_str().parse::<i64>().unwrap(), &conn);
    Json(json!(events))
}

#[get("/event/current/guild/<path_guild_id>", format = "application/json")]
pub fn get_all_current(conn: DbConn, path_guild_id: String) -> Json<Value> {
    let events = Event::get_current_events(path_guild_id.as_str().parse::<i64>().unwrap(), &conn);
    Json(json!(events))
}

#[post("/event", format = "application/json", data = "<new_event>")]
pub fn new_event(conn: DbConn, new_event: Json<NewEvent>) -> Json<Value> {
    Json(json!(Event::insert_event(new_event.into_inner(), &conn)))
}

#[get("/event/<event_id>", format = "application/json")]
pub fn get_one(conn: DbConn, event_id: String) -> Json<Value> {
    Json(json!(Event::get_event_by_id(event_id.as_str(), &conn)))
}

#[put("/event/<event_id>", format = "application/json", data = "<new_event>")]
pub fn update(conn: DbConn, new_event: Json<NewEvent>, event_id: String) -> Json<Value> {
    Json(json!(Event::update_by_id(event_id.as_str(), new_event.into_inner(), &conn)))
}

// We should be able to use the delete macro, but DELETE routes seem to be bugged
#[get("/event/delete/<event_id>", format = "application/json")]
pub fn delete(conn: DbConn, event_id: String) -> Json<Value> {
    Json(json!(Event::delete_by_id(event_id.as_str(), &conn)))
}

