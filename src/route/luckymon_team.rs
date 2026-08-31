use crate::prelude::*;

use crate::config::db::Conn as DbConn;
use crate::domain::battle;
use crate::domain::pokemon_stat::PokemonStat;
use rocket::http::Status;
use rocket::response::status;
use rocket_contrib::json::Json;
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct SaveTeamRequest {
    pub user_id: i64,
    pub picks: Vec<TeamPick>,
}

/// A pick with its species name attached, so the bot can label a team without
/// a round trip to PokeAPI for every slot.
#[derive(Serialize, Debug)]
pub struct TeamEntry {
    pub pokemon_id: i64,
    pub shiny: bool,
    pub name: String,
}

fn enrich(picks: &[TeamPick], conn: &DbConn) -> Vec<TeamEntry> {
    let ids: Vec<i64> = picks.iter().map(|p| p.pokemon_id).collect();
    let stats = PokemonStat::get_by_ids(&ids, conn);

    picks
        .iter()
        .map(|p| TeamEntry {
            pokemon_id: p.pokemon_id,
            shiny: p.shiny,
            name: stats
                .iter()
                .find(|s| s.pokemon_id == p.pokemon_id)
                .map(|s| s.name.clone())
                .unwrap_or_else(|| format!("#{}", p.pokemon_id)),
        })
        .collect()
}

/// The team a user would field right now: their saved team if it is still
/// valid, otherwise the auto-pick. Resolving server-side means the challenge
/// embed and the battle always agree on what is being fielded.
#[get("/luckymon-team/user-id/<user_id>", format = "application/json")]
pub fn get_for_user(conn: DbConn, user_id: String) -> status::Custom<Json<Value>> {
    let user_id = match user_id.parse::<i64>() {
        Ok(u) => u,
        Err(_) => {
            return status::Custom(
                Status::BadRequest,
                Json(json!({ "error": "user_id must be a number." })),
            )
        }
    };

    let saved = LuckymonTeam::picks_for_user(user_id, &conn);

    match battle::resolve_team(user_id, None, &conn) {
        Ok(picks) => status::Custom(
            Status::Ok,
            Json(json!({
                "user_id": user_id,
                "picks": enrich(&picks, &conn),
                "saved": saved.is_some(),
            })),
        ),
        Err(message) => status::Custom(Status::BadRequest, Json(json!({ "error": message }))),
    }
}

/// Saves a team, validating ownership first so a stored team is always
/// fieldable at the moment it was set.
#[post("/luckymon-team", format = "application/json", data = "<req>")]
pub fn save(conn: DbConn, req: Json<SaveTeamRequest>) -> status::Custom<Json<Value>> {
    let req = req.into_inner();

    if req.picks.is_empty() {
        return status::Custom(
            Status::BadRequest,
            Json(json!({ "error": "Pick at least one pokemon." })),
        );
    }

    match battle::resolve_team(req.user_id, Some(&req.picks), &conn) {
        Ok(validated) => {
            let saved = LuckymonTeam::save_for_user(req.user_id, &validated, &conn);
            status::Custom(
                Status::Ok,
                Json(json!({
                    "user_id": req.user_id,
                    "picks": enrich(&validated, &conn),
                    "saved": !saved.is_empty(),
                })),
            )
        }
        Err(message) => status::Custom(Status::BadRequest, Json(json!({ "error": message }))),
    }
}

#[delete("/luckymon-team/<user_id>", format = "application/json")]
pub fn clear(conn: DbConn, user_id: String) -> status::Custom<Json<Value>> {
    let user_id = match user_id.parse::<i64>() {
        Ok(u) => u,
        Err(_) => {
            return status::Custom(
                Status::BadRequest,
                Json(json!({ "error": "user_id must be a number." })),
            )
        }
    };

    let removed = LuckymonTeam::clear_for_user(user_id, &conn);
    status::Custom(Status::Ok, Json(json!({ "removed": removed })))
}
