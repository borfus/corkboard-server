use crate::prelude::*;

use crate::config::db::Conn as DbConn;
use rocket_contrib::json::Json;
use serde_json::Value;

#[get("/pin", format = "application/json")]
pub fn get_all(conn: DbConn) -> Json<Value> {
    let pins = Pin::get_all_pins(&conn);
    Json(json!(pins))
}

#[post("/pin", format = "application/json", data = "<new_pin>")]
pub fn new_pin(conn: DbConn, new_pin: Json<NewPin>) -> Json<Value> {
    let new_pin = Pin::insert_pin(new_pin.into_inner(), &conn);
    Json(json!(Pin::get_pin_by_id(new_pin.to_string().as_str(), &conn)))
}

#[get("/pin/<pin_id>", format = "application/json")]
pub fn get_one(conn: DbConn, pin_id: String) -> Json<Value> {
    Json(json!(Pin::get_pin_by_id(pin_id.as_str(), &conn)))
}

#[put("/pin/<pin_id>", format = "application/json", data = "<new_pin>")]
pub fn update(conn: DbConn, new_pin: Json<NewPin>, pin_id: String) -> Json<Value> {
    Json(json!(Pin::update_by_id(pin_id.as_str(), new_pin.into_inner(), &conn)))
}

// We should be able to use the delete macro, but DELETE routes seem to be bugged
#[get("/pin/delete/<pin_id>", format = "application/json")]
pub fn delete(conn: DbConn, pin_id: String) -> Json<Value> {
    Json(json!(Pin::delete_by_id(pin_id.as_str(), &conn)))
}

