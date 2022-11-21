use crate::prelude::*;

use crate::config::db::Conn as DbConn;
use rocket_contrib::json::Json;
use serde_json::Value;

#[get("/faq", format = "application/json")]
pub fn get_all(conn: DbConn) -> Json<Value> {
    let faqs = Faq::get_all_faqs(&conn);
    Json(json!(faqs))
}

#[post("/faq", format = "application/json", data = "<new_faq>")]
pub fn new_faq(conn: DbConn, new_faq: Json<NewFaq>) -> Json<Value> {
    Faq::insert_faq(new_faq.into_inner(), &conn);
    Json(json!(Faq::get_all_faqs(&conn).first()))
}

#[get("/faq/<faq_id>", format = "application/json")]
pub fn get_one(conn: DbConn, faq_id: String) -> Json<Value> {
    Json(json!(Faq::get_faq_by_id(faq_id.as_str(), &conn)))
}

