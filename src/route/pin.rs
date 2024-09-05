use crate::prelude::*;

use crate::config::db::Conn as DbConn;
use rocket_contrib::json::Json;
use serde_json::Value;

#[get("/pin/guild/<path_guild_id>", format = "application/json")]
pub fn get_all(conn: DbConn, path_guild_id: String) -> Json<Value> {
    let pins = Pin::get_all_pins(path_guild_id.as_str().parse::<i64>().unwrap(), &conn);
    Json(json!(pins))
}

#[post("/pin", format = "application/json", data = "<new_pin>")]
pub fn new_pin(conn: DbConn, new_pin: Json<NewPin>) -> Json<Value> {
    Json(json!(Pin::insert_pin(new_pin.into_inner(), &conn)))
}

#[get("/pin/<pin_id>/guild/<path_guild_id>", format = "application/json")]
pub fn get_one(conn: DbConn, pin_id: String, path_guild_id: String) -> Json<Value> {
    Json(json!(Pin::get_pin_by_id(
        pin_id.as_str(),
        path_guild_id.as_str().parse::<i64>().unwrap(),
        &conn
    )))
}

#[put("/pin/<pin_id>", format = "application/json", data = "<new_pin>")]
pub fn update(conn: DbConn, new_pin: Json<NewPin>, pin_id: String) -> Json<Value> {
    Json(json!(Pin::update_by_id(
        pin_id.as_str(),
        new_pin.into_inner(),
        &conn
    )))
}

#[delete("/pin/delete/<pin_id>", format = "application/json")]
pub fn delete(conn: DbConn, pin_id: String) -> Json<Value> {
    Json(json!(Pin::delete_by_id(pin_id.as_str(), &conn)))
}
