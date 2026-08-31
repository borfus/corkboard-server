use crate::prelude::*;

use crate::config::db::Conn as DbConn;
use crate::domain::battle;
use rocket::http::Status;
use rocket::response::status;
use rocket_contrib::json::Json;
use serde_json::Value;

fn bad_request(message: String) -> status::Custom<Json<Value>> {
    status::Custom(Status::BadRequest, Json(json!({ "error": message })))
}

/// Runs a battle end to end: resolve teams, simulate, persist, return the log.
///
/// The bot sends chosen pokemon ids and the server checks them against the
/// caller's untraded collection before anything is simulated. That check is the
/// whole anti-cheat story, so it must not move to the client.
#[post("/luckymon-battle", format = "application/json", data = "<req>")]
pub fn create(conn: DbConn, req: Json<BattleRequest>) -> status::Custom<Json<Value>> {
    match LuckymonBattle::run(req.into_inner(), &conn) {
        Ok(outcome) => status::Custom(Status::Ok, Json(json!(outcome))),
        Err(message) => bad_request(message),
    }
}

#[get("/luckymon-battle/<battle_id>", format = "application/json")]
pub fn get_one(conn: DbConn, battle_id: String) -> status::Custom<Json<Value>> {
    match LuckymonBattle::get_by_id(battle_id.as_str(), &conn) {
        Some(b) => status::Custom(Status::Ok, Json(json!(b))),
        None => status::Custom(
            Status::NotFound,
            Json(json!({ "error": format!("No battle with ID {}.", battle_id) })),
        ),
    }
}

/// Top N by wins, scoped to one guild and one mode.
#[get(
    "/luckymon-battle/leaderboard/<guild_id>?<battle_type>&<limit>",
    format = "application/json"
)]
pub fn leaderboard(
    conn: DbConn,
    guild_id: String,
    battle_type: Option<String>,
    limit: Option<i64>,
) -> status::Custom<Json<Value>> {
    let guild_id = match guild_id.parse::<i64>() {
        Ok(g) => g,
        Err(_) => return bad_request("guild_id must be a number.".to_string()),
    };

    let mode = battle_type.unwrap_or_else(|| "pvp".to_string()).to_lowercase();
    if mode != "pvp" && mode != "pve" {
        return bad_request("battle_type must be 'pvp' or 'pve'.".to_string());
    }

    let limit = limit.unwrap_or(10).max(1).min(50);
    let rows = battle::leaderboard(guild_id, &mode, limit, &conn);

    status::Custom(
        Status::Ok,
        Json(json!({
            "guild_id": guild_id,
            "battle_type": mode,
            "entries": rows,
        })),
    )
}

/// The NPC this player would face next, announced before they commit so they
/// can counter-pick with `/luckyteam`.
///
/// Stable until they actually battle: the answer is derived from the date and
/// how many PvE battles they have already fought today, not from the battle's
/// random seed.
#[get(
    "/luckymon-battle/next-trainer/<guild_id>/<user_id>",
    format = "application/json"
)]
pub fn next_trainer(
    conn: DbConn,
    guild_id: String,
    user_id: String,
) -> status::Custom<Json<Value>> {
    let guild_id = match guild_id.parse::<i64>() {
        Ok(g) => g,
        Err(_) => return bad_request("guild_id must be a number.".to_string()),
    };
    let user_id = match user_id.parse::<i64>() {
        Ok(u) => u,
        Err(_) => return bad_request("user_id must be a number.".to_string()),
    };

    status::Custom(
        Status::Ok,
        Json(json!(battle::next_trainer_view(guild_id, user_id, &conn))),
    )
}

/// Today's battlefield. Computed from the date, so every caller agrees.
#[get("/luckymon-battlefield", format = "application/json")]
pub fn todays_battlefield(_conn: DbConn) -> Json<Value> {
    Json(json!(battle::todays_battlefield()))
}
