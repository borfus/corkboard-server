use crate::prelude::*;

use crate::config::db::Conn as DbConn;
use rocket_contrib::json::Json;
use serde_json::Value;

#[get("/faq/guild/<path_guild_id>", format = "application/json")]
pub fn get_all(conn: DbConn, path_guild_id: String) -> Json<Value> {
    let faqs = Faq::get_all_faqs(path_guild_id.as_str().parse::<i64>().unwrap(), &conn);
    Json(json!(faqs))
}

#[post("/faq", format = "application/json", data = "<new_faq>")]
pub fn new_faq(conn: DbConn, new_faq: Json<NewFaq>) -> Json<Value> {
    Json(json!(Faq::insert_faq(new_faq.into_inner(), &conn)))
}

#[get("/faq/<faq_id>/guild/<path_guild_id>", format = "application/json")]
pub fn get_one(conn: DbConn, faq_id: String, path_guild_id: String) -> Json<Value> {
    Json(json!(Faq::get_faq_by_id(
        faq_id.as_str(),
        path_guild_id.as_str().parse::<i64>().unwrap(),
        &conn
    )))
}

#[put("/faq/<faq_id>", format = "application/json", data = "<new_faq>")]
pub fn update(conn: DbConn, new_faq: Json<NewFaq>, faq_id: String) -> Json<Value> {
    Json(json!(Faq::update_by_id(
        faq_id.as_str(),
        new_faq.into_inner(),
        &conn
    )))
}

#[delete("/faq/delete/<faq_id>", format = "application/json")]
pub fn delete(conn: DbConn, faq_id: String) -> Json<Value> {
    Json(json!(Faq::delete_by_id(faq_id.as_str(), &conn)))
}
