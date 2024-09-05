use crate::prelude::*;

use crate::config::db::Conn as DbConn;
use rocket_contrib::json::Json;
use serde_json::Value;

#[get(
    "/luckymon-history/user-id/<path_user_id>",
    format = "application/json"
)]
pub fn get_all(conn: DbConn, path_user_id: String) -> Json<Value> {
    let hists =
        LuckymonHistory::get_all_hist_by_user(path_user_id.as_str().parse::<i64>().unwrap(), &conn);
    Json(json!(hists))
}

#[post(
    "/luckymon-history?<trade>",
    format = "application/json",
    data = "<new_hist>"
)]
pub fn new_hist(
    conn: DbConn,
    new_hist: Json<NewLuckymonHistory>,
    trade: Option<bool>,
) -> Json<Value> {
    let trade = trade.unwrap_or(false);

    Json(json!(LuckymonHistory::insert_hist(
        new_hist.into_inner(),
        &conn,
        trade
    )))
}

#[get("/luckymon-history/<hist_id>", format = "application/json")]
pub fn get_one(conn: DbConn, hist_id: String) -> Json<Value> {
    Json(json!(LuckymonHistory::get_hist_by_id(
        hist_id.as_str(),
        &conn
    )))
}

#[put(
    "/luckymon-history/<hist_id>",
    format = "application/json",
    data = "<new_hist>"
)]
pub fn update(conn: DbConn, new_hist: Json<NewLuckymonHistory>, hist_id: String) -> Json<Value> {
    Json(json!(LuckymonHistory::update_by_id(
        hist_id.as_str(),
        new_hist.into_inner(),
        &conn
    )))
}

#[put("/luckymon-history/traded/<hist_id>", format = "application/json")]
pub fn update_traded(conn: DbConn, hist_id: String) -> Json<Value> {
    Json(json!(LuckymonHistory::update_as_traded(
        hist_id.to_string().as_str(),
        &conn
    )))
}

#[delete("/luckymon-history/delete/<hist_id>", format = "application/json")]
pub fn delete(conn: DbConn, hist_id: String) -> Json<Value> {
    Json(json!(LuckymonHistory::delete_by_id(
        hist_id.as_str(),
        &conn
    )))
}
