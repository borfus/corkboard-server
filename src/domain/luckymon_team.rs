//! A user's saved default team.
//!
//! Optional by design: a player who has never run `/luckyteam` still gets a
//! sensible auto-picked team, so the very first `/luckybattle` works without
//! reading any documentation.

use chrono::NaiveDateTime;
use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::luckymon_team;
use crate::schema::luckymon_team::dsl::luckymon_team as all_luckymon_team;

use super::battle_sim::TEAM_SIZE;

#[derive(Serialize, Queryable, Debug, Clone)]
pub struct LuckymonTeam {
    pub id: Uuid,
    pub last_modified_date: NaiveDateTime,
    pub user_id: Option<i64>,
    pub slot: Option<i32>,
    pub pokemon_id: Option<i64>,
    pub shiny: Option<bool>,
}

#[derive(Serialize, Deserialize, Insertable, Debug, Clone)]
#[table_name = "luckymon_team"]
pub struct NewLuckymonTeam {
    pub user_id: Option<i64>,
    pub slot: Option<i32>,
    pub pokemon_id: Option<i64>,
    pub shiny: Option<bool>,
}

/// One chosen pokemon, as the bot sends it and as the simulator consumes it.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct TeamPick {
    pub pokemon_id: i64,
    pub shiny: bool,
}

impl LuckymonTeam {
    pub fn get_for_user(user_id: i64, conn: &PgConnection) -> Vec<LuckymonTeam> {
        all_luckymon_team
            .filter(luckymon_team::user_id.eq(Some(user_id)))
            .order(luckymon_team::slot)
            .load::<LuckymonTeam>(conn)
            .unwrap_or_default()
    }

    /// The saved team as picks, or `None` when the user has not saved one.
    ///
    /// A short team is valid: a player three days into their collection can
    /// still save and field what they have.
    pub fn picks_for_user(user_id: i64, conn: &PgConnection) -> Option<Vec<TeamPick>> {
        let picks: Vec<TeamPick> = Self::get_for_user(user_id, conn)
            .iter()
            .filter_map(|r| {
                r.pokemon_id.map(|pid| TeamPick {
                    pokemon_id: pid,
                    shiny: r.shiny.unwrap_or(false),
                })
            })
            .collect();

        if picks.is_empty() || picks.len() > TEAM_SIZE {
            None
        } else {
            Some(picks)
        }
    }

    /// Replaces the user's whole team. Slots are rewritten together so a
    /// partially saved team can never be read back.
    pub fn save_for_user(
        user_id: i64,
        picks: &[TeamPick],
        conn: &PgConnection,
    ) -> Vec<LuckymonTeam> {
        conn.transaction::<_, diesel::result::Error, _>(|| {
            diesel::delete(all_luckymon_team.filter(luckymon_team::user_id.eq(Some(user_id))))
                .execute(conn)?;

            let rows: Vec<NewLuckymonTeam> = picks
                .iter()
                .enumerate()
                .map(|(i, p)| NewLuckymonTeam {
                    user_id: Some(user_id),
                    slot: Some(i as i32),
                    pokemon_id: Some(p.pokemon_id),
                    shiny: Some(p.shiny),
                })
                .collect();

            diesel::insert_into(luckymon_team::table)
                .values(&rows)
                .get_results::<LuckymonTeam>(conn)
        })
        .unwrap_or_default()
    }

    pub fn clear_for_user(user_id: i64, conn: &PgConnection) -> usize {
        diesel::delete(all_luckymon_team.filter(luckymon_team::user_id.eq(Some(user_id))))
            .execute(conn)
            .unwrap_or(0)
    }
}
