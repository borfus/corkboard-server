use crate::prelude::*;

use crate::config::db::Conn as DbConn;
use rocket_contrib::json::Json;
use serde_json::Value;

#[get("/user", format = "application/json")]
pub fn get_all(conn: DbConn) -> Json<Value> {
    let users = User::get_all_users(&conn);
    Json(json!(users))
}

#[post("/user", format = "application/json", data = "<new_user>")]
pub fn new_user(conn: DbConn, new_user: Json<NewUser>) -> Json<Value> {
    User::insert_user(new_user.into_inner(), &conn);
    Json(json!(User::get_all_users(&conn).first()))
}

#[get("/user/<username>", format = "application/json")]
pub fn get_one(conn: DbConn, username: String) -> Json<Value> {
    Json(json!(User::get_user_by_username(username, &conn)))
}

