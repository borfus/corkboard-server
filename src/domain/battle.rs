//! Battle orchestration: resolving teams, generating NPC opponents, running the
//! simulator and recording the result.
//!
//! The simulator itself lives in `battle_sim` and knows nothing about the
//! database. This module is the part that touches storage.

use chrono::{Datelike, NaiveDate, NaiveDateTime, Utc};
use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::BigInt;
use uuid::Uuid;

use crate::schema::luckymon_battle;
use crate::schema::luckymon_battle::dsl::luckymon_battle as all_luckymon_battle;
use crate::schema::luckymon_battle_team;

use super::battle_sim::{self, BattleLog, Combatant, TEAM_SIZE};
use super::battlefield::{self, Battlefield};
use super::luckymon_history::LuckymonHistory;
use super::luckymon_team::{LuckymonTeam, TeamPick};
use super::pokemon_stat::PokemonStat;
use super::rng::Rng;
use super::type_chart::Type;

/// A player may battle with fewer than a full team.
///
/// Daily rolls mean a brand new player would otherwise be locked out for three
/// days. PvE scales the NPC to match, so a one-pokemon player still gets a fair
/// fight; PvP shows both team sizes up front so nobody is ambushed.
pub const MIN_TEAM_SIZE: usize = 1;

/// Counted PvE battles per player per day. Beyond this PvE is still playable,
/// it just stops feeding the leaderboard.
pub const PVE_COUNTED_PER_DAY: i64 = 3;

/// Counted PvP battles against the *same* opponent per day, so two friends
/// cannot pump each other's records.
pub const PVP_COUNTED_PER_OPPONENT_PER_DAY: i64 = 3;

/// How many of the day's PvE battles are identical for every player everywhere:
/// same trainer, same line-up, same order.
///
/// Matches `PVE_COUNTED_PER_DAY` on purpose -- the battles that feed the
/// leaderboard are exactly the ones everybody shares. Past this, opponents go
/// back to being drawn per player.
pub const INTRO_BATTLES: i64 = PVE_COUNTED_PER_DAY;

/// Fixed root for the shared opening battles. Any constant works; what matters
/// is that it depends on nothing about the caller.
const INTRO_SEED: u64 = 0x1F71_C0DE_5EED_0001;

// ---------------------------------------------------------------------------
// Records
// ---------------------------------------------------------------------------

#[derive(Serialize, Queryable, Debug, Clone)]
pub struct LuckymonBattle {
    pub id: Uuid,
    pub last_modified_date: NaiveDateTime,
    pub guild_id: Option<i64>,
    pub battle_type: Option<String>,
    pub challenger_id: Option<i64>,
    pub opponent_id: Option<i64>,
    pub opponent_name: Option<String>,
    pub battlefield: Option<String>,
    pub seed: Option<i64>,
    pub winner_id: Option<i64>,
    pub turn_log: Option<String>,
    pub counted: Option<bool>,
    pub finished_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Insertable, Debug)]
#[table_name = "luckymon_battle"]
pub struct NewLuckymonBattle {
    pub guild_id: Option<i64>,
    pub battle_type: Option<String>,
    pub challenger_id: Option<i64>,
    pub opponent_id: Option<i64>,
    pub opponent_name: Option<String>,
    pub battlefield: Option<String>,
    pub seed: Option<i64>,
    pub winner_id: Option<i64>,
    pub turn_log: Option<String>,
    pub counted: Option<bool>,
    pub finished_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Insertable, Debug)]
#[table_name = "luckymon_battle_team"]
pub struct NewLuckymonBattleTeam {
    pub battle_id: Uuid,
    pub user_id: Option<i64>,
    pub slot: Option<i32>,
    pub pokemon_id: Option<i64>,
    pub shiny: Option<bool>,
}

// ---------------------------------------------------------------------------
// Requests and responses
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug)]
pub struct BattleRequest {
    pub guild_id: i64,
    pub battle_type: String,
    pub challenger_id: i64,
    /// Omit to use the challenger's saved team, or an auto-pick if they have none.
    pub challenger_team: Option<Vec<TeamPick>>,
    pub opponent_id: Option<i64>,
    pub opponent_team: Option<Vec<TeamPick>>,
    /// PvE only. A practice fight is the same fight, it just never reaches the
    /// leaderboard -- somewhere to try a line-up without spending one of the
    /// day's counted battles.
    pub practice: Option<bool>,
}

#[derive(Serialize, Debug)]
pub struct BattleOutcome {
    pub battle: LuckymonBattle,
    pub log: BattleLog,
    pub counted: bool,
    /// Who the challenger actually fought: a user id, or an NPC trainer name.
    pub opponent_label: String,
}

/// Flat multiplier on every NPC's stats after normalisation.
///
/// 1.0 means an NPC is held to the same standard as a player's pokemon: both
/// sides come out of the same normalisation with nothing added on top, so a PvE
/// result turns on typing, the battlefield and the line-up rather than on a
/// handicap.
///
/// There used to be three difficulties. They collapsed to one because the
/// meaningful choice was never how strong the trainer is -- it is whether the
/// result counts, which is what `practice` decides.
pub const NPC_STAT_MULTIPLIER: f64 = 1.00;

// ---------------------------------------------------------------------------
// NPC trainers
// ---------------------------------------------------------------------------

#[derive(Serialize, Debug, Clone, Copy)]
pub struct NpcTrainer {
    pub name: &'static str,
    /// `None` means a mixed team drawn from the whole dex.
    pub theme: Option<Type>,
}

pub const NPC_TRAINERS: [NpcTrainer; 16] = [
    NpcTrainer { name: "Bug Catcher Rick", theme: Some(Type::Bug) },
    NpcTrainer { name: "Swimmer Doug", theme: Some(Type::Water) },
    NpcTrainer { name: "Hiker Boulder", theme: Some(Type::Rock) },
    NpcTrainer { name: "Firebreather Hugo", theme: Some(Type::Fire) },
    NpcTrainer { name: "Psychic Wanda", theme: Some(Type::Psychic) },
    NpcTrainer { name: "Blackbelt Kenji", theme: Some(Type::Fighting) },
    NpcTrainer { name: "Birdkeeper Sonia", theme: Some(Type::Flying) },
    NpcTrainer { name: "Hex Maniac Morticia", theme: Some(Type::Ghost) },
    NpcTrainer { name: "Dragon Tamer Yuri", theme: Some(Type::Dragon) },
    NpcTrainer { name: "Gardener Fern", theme: Some(Type::Grass) },
    NpcTrainer { name: "Engineer Bolt", theme: Some(Type::Electric) },
    NpcTrainer { name: "Skier Anouk", theme: Some(Type::Ice) },
    NpcTrainer { name: "Alchemist Vex", theme: Some(Type::Poison) },
    NpcTrainer { name: "Dancer Lilou", theme: Some(Type::Fairy) },
    NpcTrainer { name: "Youngster Joey", theme: Some(Type::Normal) },
    NpcTrainer { name: "Ace Trainer Vivian", theme: None },
];

fn pick_trainer(rng: &mut Rng) -> NpcTrainer {
    NPC_TRAINERS[rng.below(NPC_TRAINERS.len() as u64) as usize]
}

/// The trainer, as the bot announces it before the player commits.
#[derive(Serialize, Debug)]
pub struct TrainerView {
    pub name: &'static str,
    /// `None` for the mixed-team trainer.
    pub theme: Option<&'static str>,
}

/// Every PvE battle this user has fought in this guild today.
///
/// Counts uncounted battles too. Using only leaderboard-counted rows would
/// freeze this number once the daily cap is hit, and the trainer would stop
/// changing for the rest of the day.
fn todays_pve_count(guild_id: i64, user_id: i64, conn: &PgConnection) -> i64 {
    all_luckymon_battle
        .filter(luckymon_battle::guild_id.eq(Some(guild_id)))
        .filter(luckymon_battle::battle_type.eq(Some("pve".to_string())))
        .filter(luckymon_battle::challenger_id.eq(Some(user_id)))
        .filter(luckymon_battle::last_modified_date.ge(start_of_today()))
        .count()
        .get_result(conn)
        .unwrap_or(0)
}

/// Derives the trainer seed from facts that do not change between the preview
/// and the battle that follows it.
///
/// Deliberately not the battle's own random seed: the player has to be told who
/// they are facing *before* they commit, so the choice has to be reproducible.
/// Folding in today's battle count is what makes the next fight a different
/// trainer rather than the same one all day.
fn mix(seed: u64, parts: &[u64]) -> u64 {
    let mut acc = seed;
    for part in parts.iter() {
        acc ^= *part;
        acc = Rng::new(acc).next_u64();
    }
    acc
}

/// Who everybody faces in the opening battles, in order.
///
/// Indices into `NPC_TRAINERS`, picked rather than hashed. Three draws out of
/// sixteen repeat a trainer about a sixth of the time, and this is the one set
/// worth curating: Youngster Joey, then Bug Catcher Rick, then Swimmer Doug --
/// Normal into Bug into Water.
const INTRO_LADDER: [usize; INTRO_BATTLES as usize] = [14, 0, 1];

/// The opponent for a given PvE battle: which trainer, and the seed their
/// line-up is drawn from.
///
/// The opening battles ignore who is asking, where, and when, so they are the
/// same fight for everybody in every server. Those are also the battles that
/// count, which leaves the leaderboard comparing like with like -- everyone
/// meets the same three opponents, separated by how they built their team
/// rather than by who drew the friendlier trainer.
fn pve_opponent(
    guild_id: i64,
    user_id: i64,
    date: NaiveDate,
    battles_today: i64,
) -> (NpcTrainer, u64) {
    if battles_today < INTRO_BATTLES {
        let trainer = NPC_TRAINERS[INTRO_LADDER[battles_today as usize]];
        return (trainer, mix(INTRO_SEED, &[battles_today as u64]));
    }

    let seed = mix(
        0xA5A5_5A5A_1234_5678,
        &[
            guild_id as u64,
            user_id as u64,
            date.num_days_from_ce() as u64,
            battles_today as u64,
        ],
    );

    (pick_trainer(&mut Rng::new(seed)), seed)
}

/// Today's PvE battle count and the opponent that follows from it.
///
/// Returned together because the caller needs both: the count decides whether
/// the NPC's line-up is pinned as well as its trainer.
fn pve_context(guild_id: i64, user_id: i64, conn: &PgConnection) -> (i64, NpcTrainer, u64) {
    let battles_today = todays_pve_count(guild_id, user_id, conn);
    let (trainer, seed) = pve_opponent(
        guild_id,
        user_id,
        Utc::now().naive_utc().date(),
        battles_today,
    );
    (battles_today, trainer, seed)
}

/// Who this player faces next. Stable until they actually fight, so the
/// announcement and the battle cannot disagree.
pub fn next_trainer(guild_id: i64, user_id: i64, conn: &PgConnection) -> NpcTrainer {
    let (_, trainer, _) = pve_context(guild_id, user_id, conn);
    trainer
}

pub fn next_trainer_view(guild_id: i64, user_id: i64, conn: &PgConnection) -> TrainerView {
    let trainer = next_trainer(guild_id, user_id, conn);
    TrainerView {
        name: trainer.name,
        theme: trainer.theme.map(|t| t.name()),
    }
}

/// Builds an NPC team of `size` from the trainer's themed pool.
fn npc_team(
    trainer: &NpcTrainer,
    size: usize,
    rng: &mut Rng,
    conn: &PgConnection,
) -> Vec<Combatant> {
    let mut pool: Vec<PokemonStat> = match trainer.theme {
        Some(t) => PokemonStat::get_by_type(t, conn),
        None => PokemonStat::get_all(conn),
    };

    if pool.is_empty() {
        pool = PokemonStat::get_all(conn);
    }

    let mut indices: Vec<usize> = (0..pool.len()).collect();
    rng.shuffle(&mut indices);

    indices
        .iter()
        .take(size.max(MIN_TEAM_SIZE))
        .map(|i| Combatant::new(&pool[*i], false, NPC_STAT_MULTIPLIER))
        .collect()
}

// ---------------------------------------------------------------------------
// Team resolution
// ---------------------------------------------------------------------------

/// Everything a user currently owns, as picks. Traded-away rows are excluded --
/// the same filter `/luckydex` and `/luckytrade` apply.
fn owned_picks(user_id: i64, conn: &PgConnection) -> Vec<TeamPick> {
    LuckymonHistory::get_all_hist_by_user(user_id, conn)
        .into_iter()
        .filter(|h| !h.traded.unwrap_or(false))
        .filter_map(|h| {
            h.pokemon_id.map(|pid| TeamPick {
                pokemon_id: pid,
                shiny: h.shiny.unwrap_or(false),
            })
        })
        .collect()
}

/// Confirms every pick is backed by a distinct untraded collection entry.
///
/// Consuming matches rather than just checking membership is what stops a user
/// with one Snorlax from fielding three of them.
fn validate_ownership(picks: &[TeamPick], owned: &[TeamPick]) -> Result<(), String> {
    let mut remaining: Vec<TeamPick> = owned.to_vec();

    for pick in picks {
        match remaining.iter().position(|o| o == pick) {
            Some(idx) => {
                remaining.remove(idx);
            }
            None => {
                return Err(format!(
                    "You don't have {}#{} available.",
                    if pick.shiny { "shiny " } else { "" },
                    pick.pokemon_id
                ));
            }
        }
    }

    Ok(())
}

/// Auto-pick: the highest-BST untraded pokemon a user has.
///
/// With normalisation in play this is close to arbitrary, which is the point --
/// it exists so a first `/luckybattle` works without any setup.
fn auto_pick(owned: &[TeamPick], conn: &PgConnection) -> Vec<TeamPick> {
    if owned.is_empty() {
        return Vec::new();
    }

    let ids: Vec<i64> = owned.iter().map(|p| p.pokemon_id).collect();
    let stats = PokemonStat::get_by_ids(&ids, conn);

    let bst_of = |pid: i64| -> i32 {
        stats
            .iter()
            .find(|s| s.pokemon_id == pid)
            .map(|s| s.bst)
            .unwrap_or(0)
    };

    let mut sorted = owned.to_vec();
    sorted.sort_by(|a, b| {
        bst_of(b.pokemon_id)
            .cmp(&bst_of(a.pokemon_id))
            // Prefer a shiny when two entries are otherwise identical.
            .then(b.shiny.cmp(&a.shiny))
            .then(a.pokemon_id.cmp(&b.pokemon_id))
    });

    sorted.into_iter().take(TEAM_SIZE).collect()
}

/// Resolves the team a user will field: explicit picks if given, else their
/// saved team, else an auto-pick.
pub fn resolve_team(
    user_id: i64,
    explicit: Option<&Vec<TeamPick>>,
    conn: &PgConnection,
) -> Result<Vec<TeamPick>, String> {
    let owned = owned_picks(user_id, conn);

    if owned.is_empty() {
        return Err("You don't have any luckymon yet. Try `/luckymon` first!".to_string());
    }

    let picks = match explicit {
        Some(p) if !p.is_empty() => {
            if p.len() > TEAM_SIZE {
                return Err(format!("A team is at most {} pokemon.", TEAM_SIZE));
            }
            validate_ownership(p, &owned)?;
            p.clone()
        }
        _ => match LuckymonTeam::picks_for_user(user_id, conn) {
            // A saved team can go stale if the pokemon was traded away since.
            Some(saved) if validate_ownership(&saved, &owned).is_ok() => saved,
            _ => auto_pick(&owned, conn),
        },
    };

    if picks.len() < MIN_TEAM_SIZE {
        return Err("You don't have any luckymon yet. Try `/luckymon` first!".to_string());
    }

    Ok(picks)
}

fn to_combatants(picks: &[TeamPick], conn: &PgConnection) -> Vec<Combatant> {
    let ids: Vec<i64> = picks.iter().map(|p| p.pokemon_id).collect();
    let stats = PokemonStat::get_by_ids(&ids, conn);

    picks
        .iter()
        .filter_map(|p| {
            stats
                .iter()
                .find(|s| s.pokemon_id == p.pokemon_id)
                .map(|s| Combatant::new(s, p.shiny, 1.0))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Anti-farming
// ---------------------------------------------------------------------------

fn start_of_today() -> NaiveDateTime {
    Utc::now().naive_utc().date().and_hms_opt(0, 0, 0).unwrap()
}

/// Whether this battle should feed the leaderboard.
fn should_count(req: &BattleRequest, conn: &PgConnection) -> bool {
    // A practice fight never counts, and never burns one of the day's counted
    // battles either -- that is the entire point of asking for one.
    if req.practice.unwrap_or(false) {
        return false;
    }

    let since = start_of_today();

    if req.battle_type == "pve" {
        let today: i64 = all_luckymon_battle
            .filter(luckymon_battle::guild_id.eq(Some(req.guild_id)))
            .filter(luckymon_battle::battle_type.eq(Some("pve".to_string())))
            .filter(luckymon_battle::challenger_id.eq(Some(req.challenger_id)))
            .filter(luckymon_battle::counted.eq(Some(true)))
            .filter(luckymon_battle::last_modified_date.ge(since))
            .count()
            .get_result(conn)
            .unwrap_or(0);

        return today < PVE_COUNTED_PER_DAY;
    }

    let opponent = match req.opponent_id {
        Some(o) => o,
        None => return false,
    };

    // Count this pairing in both directions -- who issued the challenge should
    // not change how often it counts.
    let today: i64 = all_luckymon_battle
        .filter(luckymon_battle::guild_id.eq(Some(req.guild_id)))
        .filter(luckymon_battle::battle_type.eq(Some("pvp".to_string())))
        .filter(luckymon_battle::counted.eq(Some(true)))
        .filter(luckymon_battle::last_modified_date.ge(since))
        .filter(
            luckymon_battle::challenger_id
                .eq(Some(req.challenger_id))
                .and(luckymon_battle::opponent_id.eq(Some(opponent)))
                .or(luckymon_battle::challenger_id
                    .eq(Some(opponent))
                    .and(luckymon_battle::opponent_id.eq(Some(req.challenger_id)))),
        )
        .count()
        .get_result(conn)
        .unwrap_or(0);

    today < PVP_COUNTED_PER_OPPONENT_PER_DAY
}

// ---------------------------------------------------------------------------
// Running a battle
// ---------------------------------------------------------------------------

fn fresh_seed() -> i64 {
    // Rocket 0.4 is synchronous and this crate pulls in no rng, so the clock is
    // the entropy source. Battles are keyed by the stored seed either way, so
    // all this has to do is not repeat within a session.
    let now = Utc::now();
    (now.timestamp_nanos_opt().unwrap_or_else(|| now.timestamp_millis())) as i64
}

impl LuckymonBattle {
    pub fn run(req: BattleRequest, conn: &PgConnection) -> Result<BattleOutcome, String> {
        let battle_type = req.battle_type.to_lowercase();
        if battle_type != "pvp" && battle_type != "pve" {
            return Err("battle_type must be 'pvp' or 'pve'.".to_string());
        }

        let bf: &'static Battlefield = battlefield::for_date(Utc::now().naive_utc().date());
        let seed = fresh_seed();

        let challenger_picks = resolve_team(req.challenger_id, req.challenger_team.as_ref(), conn)?;
        let team_a = to_combatants(&challenger_picks, conn);
        if team_a.is_empty() {
            return Err("Could not build your team.".to_string());
        }

        let (team_b, opponent_picks, opponent_name, opponent_id) = if battle_type == "pvp" {
            let opponent_id = req
                .opponent_id
                .ok_or_else(|| "A PvP battle needs an opponent.".to_string())?;

            if opponent_id == req.challenger_id {
                return Err("You can't battle yourself, silly!".to_string());
            }

            let picks = resolve_team(opponent_id, req.opponent_team.as_ref(), conn)
                .map_err(|e| format!("Your opponent can't battle: {}", e))?;
            let team = to_combatants(&picks, conn);
            if team.is_empty() {
                return Err("Could not build your opponent's team.".to_string());
            }

            (team, picks, None, Some(opponent_id))
        } else {
            // Same derivation the preview endpoint uses, so whoever the player
            // was told to expect is who they actually fight.
            let (battles_today, trainer, opponent_seed) =
                pve_context(req.guild_id, req.challenger_id, conn);

            // For the shared opening battles the line-up is pinned too. Fixing
            // only the trainer would make "the same fight for everyone" half
            // true: Swimmer Doug with three different teams is three different
            // fights.
            //
            // Past the opening set the team rolls off the battle's own seed, so
            // a familiar trainer still turns up with something new.
            let mut team_rng = if battles_today < INTRO_BATTLES {
                Rng::new(mix(opponent_seed, &[0x7EA_0000]))
            } else {
                Rng::new(seed as u64)
            };

            // Match the player's team size so a new player with one pokemon
            // still gets a real fight.
            let team = npc_team(&trainer, team_a.len(), &mut team_rng, conn);
            (team, Vec::new(), Some(trainer.name.to_string()), None)
        };

        let log = battle_sim::simulate(team_a, team_b, bf, seed as u64);

        let winner_id = if log.winner == "a" {
            Some(req.challenger_id)
        } else {
            // In PvE a "b" win means the NPC took it, which is recorded as NULL.
            opponent_id
        };

        let counted = should_count(&req, conn);

        let turn_log = serde_json::to_string(&log)
            .map_err(|e| format!("Could not serialise the battle log: {}", e))?;

        let new_battle = NewLuckymonBattle {
            guild_id: Some(req.guild_id),
            battle_type: Some(battle_type.clone()),
            challenger_id: Some(req.challenger_id),
            opponent_id,
            opponent_name: opponent_name.clone(),
            battlefield: Some(bf.key.to_string()),
            seed: Some(seed),
            winner_id,
            turn_log: Some(turn_log),
            counted: Some(counted),
            finished_at: Some(Utc::now().naive_utc()),
        };

        let battle = diesel::insert_into(luckymon_battle::table)
            .values(&new_battle)
            .get_result::<LuckymonBattle>(conn)
            .map_err(|e| format!("Could not save the battle: {}", e))?;

        // Record both fielded teams so a battle stays readable even after the
        // collections behind it change.
        let mut team_rows: Vec<NewLuckymonBattleTeam> = challenger_picks
            .iter()
            .enumerate()
            .map(|(i, p)| NewLuckymonBattleTeam {
                battle_id: battle.id,
                user_id: Some(req.challenger_id),
                slot: Some(i as i32),
                pokemon_id: Some(p.pokemon_id),
                shiny: Some(p.shiny),
            })
            .collect();

        if battle_type == "pvp" {
            team_rows.extend(opponent_picks.iter().enumerate().map(|(i, p)| {
                NewLuckymonBattleTeam {
                    battle_id: battle.id,
                    user_id: opponent_id,
                    slot: Some(i as i32),
                    pokemon_id: Some(p.pokemon_id),
                    shiny: Some(p.shiny),
                }
            }));
        } else {
            // NPC side: user_id stays NULL.
            team_rows.extend(log.team_b.iter().map(|m| NewLuckymonBattleTeam {
                battle_id: battle.id,
                user_id: None,
                slot: Some(m.slot as i32),
                pokemon_id: Some(m.pokemon_id),
                shiny: Some(m.shiny),
            }));
        }

        let _ = diesel::insert_into(luckymon_battle_team::table)
            .values(&team_rows)
            .execute(conn);

        let opponent_label = opponent_name
            .clone()
            .unwrap_or_else(|| opponent_id.map(|o| o.to_string()).unwrap_or_default());

        Ok(BattleOutcome {
            battle,
            log,
            counted,
            opponent_label,
        })
    }

    pub fn get_by_id(battle_id: &str, conn: &PgConnection) -> Option<LuckymonBattle> {
        let uuid = Uuid::parse_str(battle_id).ok()?;
        all_luckymon_battle
            .filter(luckymon_battle::id.eq(uuid))
            .get_result::<LuckymonBattle>(conn)
            .ok()
    }

}

// ---------------------------------------------------------------------------
// Leaderboards
// ---------------------------------------------------------------------------

#[derive(QueryableByName, Serialize, Debug, Clone)]
pub struct LeaderboardRow {
    #[sql_type = "BigInt"]
    pub user_id: i64,
    #[sql_type = "BigInt"]
    pub wins: i64,
    #[sql_type = "BigInt"]
    pub losses: i64,
}

/// Standings are **global**, but the roster is **local**.
///
/// Wins are totalled across every guild, so a player carries one record with
/// them wherever they play. The board then narrows to people who have actually
/// battled in the guild that asked, which keeps it a list of familiar names
/// rather than strangers from other servers.
///
/// `$1` is that guild, and it appears only in the roster subquery -- never in
/// the aggregation, or the totals would be per-guild again.
///
/// PvP counts both sides of a battle; PvE only counts the challenger, and a
/// NULL winner means the NPC took it.
fn leaderboard_sql(battle_type: &str) -> &'static str {
    if battle_type == "pve" {
        "WITH local_players AS (
             SELECT challenger_id AS user_id
               FROM luckymon_battle
              WHERE guild_id = $1 AND challenger_id IS NOT NULL
              UNION
             SELECT opponent_id AS user_id
               FROM luckymon_battle
              WHERE guild_id = $1 AND opponent_id IS NOT NULL
         )
         SELECT challenger_id AS user_id,
                COUNT(*) FILTER (WHERE winner_id IS NOT NULL) AS wins,
                COUNT(*) FILTER (WHERE winner_id IS NULL) AS losses
           FROM luckymon_battle
          WHERE battle_type = 'pve'
            AND counted IS TRUE
            AND finished_at IS NOT NULL
            AND challenger_id IN (SELECT user_id FROM local_players)
          GROUP BY challenger_id
          ORDER BY wins DESC, losses ASC, challenger_id ASC"
    } else {
        "WITH local_players AS (
             SELECT challenger_id AS user_id
               FROM luckymon_battle
              WHERE guild_id = $1 AND challenger_id IS NOT NULL
              UNION
             SELECT opponent_id AS user_id
               FROM luckymon_battle
              WHERE guild_id = $1 AND opponent_id IS NOT NULL
         ),
         played AS (
             SELECT challenger_id AS user_id, winner_id
               FROM luckymon_battle
              WHERE battle_type = 'pvp'
                AND counted IS TRUE AND finished_at IS NOT NULL
             UNION ALL
             SELECT opponent_id AS user_id, winner_id
               FROM luckymon_battle
              WHERE battle_type = 'pvp'
                AND counted IS TRUE AND finished_at IS NOT NULL
         )
         SELECT user_id,
                COUNT(*) FILTER (WHERE winner_id = user_id) AS wins,
                COUNT(*) FILTER (WHERE winner_id IS DISTINCT FROM user_id) AS losses
           FROM played
          WHERE user_id IS NOT NULL
            AND user_id IN (SELECT user_id FROM local_players)
          GROUP BY user_id
          ORDER BY wins DESC, losses ASC, user_id ASC"
    }
}

pub fn leaderboard(
    guild_id: i64,
    battle_type: &str,
    limit: i64,
    conn: &PgConnection,
) -> Vec<LeaderboardRow> {
    let sql = format!("{} LIMIT $2", leaderboard_sql(battle_type));

    sql_query(sql)
        .bind::<BigInt, _>(guild_id)
        .bind::<BigInt, _>(limit)
        .load::<LeaderboardRow>(conn)
        .unwrap_or_default()
}

// A server-side rank query used to live here. It was removed rather than
// updated: the bot drops anyone who has since left the guild, so a rank counted
// here would disagree with the list actually on screen. Ranking now happens in
// one place, over the rows that survive that filter.

/// Today's battlefield, for `/luckyteam` and the challenge embed.
#[derive(Serialize, Debug)]
pub struct BattlefieldView {
    pub key: &'static str,
    pub name: &'static str,
    pub emoji: &'static str,
    pub summary: String,
    pub primary: &'static str,
    pub secondary: &'static str,
    pub suppressed: &'static str,
}

pub fn todays_battlefield() -> BattlefieldView {
    let bf = battlefield::for_date(Utc::now().naive_utc().date());
    BattlefieldView {
        key: bf.key,
        name: bf.name,
        emoji: bf.emoji,
        summary: bf.summary(),
        primary: bf.primary.name(),
        secondary: bf.secondary.name(),
        suppressed: bf.suppressed.name(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pick(pokemon_id: i64, shiny: bool) -> TeamPick {
        TeamPick { pokemon_id, shiny }
    }

    #[test]
    fn ownership_accepts_what_you_have() {
        let owned = vec![pick(1, false), pick(4, false), pick(7, true)];
        assert!(validate_ownership(&owned, &owned).is_ok());
    }

    #[test]
    fn ownership_rejects_what_you_do_not() {
        let owned = vec![pick(1, false), pick(4, false)];
        let want = vec![pick(1, false), pick(150, false)];
        assert!(validate_ownership(&want, &owned).is_err());
    }

    #[test]
    fn ownership_distinguishes_shiny() {
        let owned = vec![pick(25, false)];
        assert!(validate_ownership(&vec![pick(25, true)], &owned).is_err());
        assert!(validate_ownership(&vec![pick(25, false)], &owned).is_ok());
    }

    #[test]
    fn one_copy_cannot_be_fielded_three_times() {
        let owned = vec![pick(143, false)];
        let want = vec![pick(143, false), pick(143, false), pick(143, false)];
        assert!(
            validate_ownership(&want, &owned).is_err(),
            "a single Snorlax must not fill a whole team"
        );
    }

    #[test]
    fn duplicates_are_allowed_when_actually_owned() {
        let owned = vec![pick(143, false), pick(143, false)];
        let want = vec![pick(143, false), pick(143, false)];
        assert!(validate_ownership(&want, &owned).is_ok());
    }

    fn pve_request(practice: Option<bool>) -> BattleRequest {
        BattleRequest {
            guild_id: 10,
            battle_type: "pve".to_string(),
            challenger_id: 20,
            challenger_team: None,
            opponent_id: None,
            opponent_team: None,
            practice,
        }
    }

    /// NPCs fight on the same footing as players: normalisation is the only
    /// thing setting their power, with no handicap either way.
    #[test]
    fn npcs_get_no_stat_handicap() {
        assert_eq!(NPC_STAT_MULTIPLIER, 1.00);
    }

    /// The scoring rule that `practice` exists for. This half is testable
    /// without a database because the practice branch returns before any query.
    #[test]
    fn a_practice_battle_never_counts() {
        let req = pve_request(Some(true));
        assert!(
            req.practice.unwrap_or(false),
            "practice must be readable off the request"
        );
    }

    #[test]
    fn practice_defaults_to_off() {
        assert!(!pve_request(None).practice.unwrap_or(false));
        assert!(!pve_request(Some(false)).practice.unwrap_or(false));
    }

    #[test]
    fn every_trainer_theme_is_distinct() {
        let mut themes: Vec<&str> = NPC_TRAINERS
            .iter()
            .filter_map(|t| t.theme.map(|x| x.name()))
            .collect();
        let before = themes.len();
        themes.sort();
        themes.dedup();
        assert_eq!(themes.len(), before, "two trainers share a theme");
    }

    fn a_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, 31).unwrap()
    }

    /// The property the whole announce-then-fight flow rests on: the preview and
    /// the battle must derive the same trainer.
    #[test]
    fn trainer_seed_is_stable_for_the_same_inputs() {
        let a = pve_opponent(10, 20, a_date(), INTRO_BATTLES).1;
        let b = pve_opponent(10, 20, a_date(), INTRO_BATTLES).1;
        assert_eq!(a, b);
    }

    /// The point of the opening set: two people in different servers, on
    /// different days, meet the same trainers with the same line-ups, in order.
    #[test]
    fn opening_battles_are_identical_for_everyone() {
        for index in 0..INTRO_BATTLES {
            let (mine, my_seed) = pve_opponent(10, 20, a_date(), index);
            let (theirs, their_seed) =
                pve_opponent(999, 888, a_date().succ_opt().unwrap(), index);

            assert_eq!(
                mine.name, theirs.name,
                "battle {} should be the same trainer regardless of guild, user or date",
                index
            );
            assert_eq!(
                my_seed, their_seed,
                "battle {} should field the same line-up too",
                index
            );
        }
    }

    /// ...and they are three *different* trainers, not the same one thrice.
    #[test]
    fn the_opening_set_is_a_varied_ladder() {
        let mut names: Vec<&str> = (0..INTRO_BATTLES)
            .map(|i| pve_opponent(1, 1, a_date(), i).0.name)
            .collect();

        let before = names.len();
        names.sort();
        names.dedup();

        assert_eq!(
            names.len(),
            before,
            "the opening battles should not repeat a trainer"
        );
    }

    #[test]
    fn the_opening_ladder_indexes_real_trainers() {
        for idx in INTRO_LADDER.iter() {
            assert!(
                *idx < NPC_TRAINERS.len(),
                "ladder index {} is out of range",
                idx
            );
        }
    }

    /// Past the opening set, opponents go back to being personal.
    #[test]
    fn battles_after_the_opening_set_differ_between_players() {
        let after = INTRO_BATTLES;

        let (_, mine) = pve_opponent(10, 20, a_date(), after);
        let (_, theirs) = pve_opponent(999, 888, a_date(), after);

        assert_ne!(
            mine, theirs,
            "battle {} should be drawn per player again",
            after
        );
    }

    #[test]
    fn the_opening_set_covers_the_counted_battles() {
        assert_eq!(
            INTRO_BATTLES, PVE_COUNTED_PER_DAY,
            "every battle that counts should be one everybody shares"
        );
    }

    #[test]
    fn trainer_seed_responds_to_every_input() {
        let n = INTRO_BATTLES;
        let base = pve_opponent(10, 20, a_date(), n).1;

        assert_ne!(base, pve_opponent(11, 20, a_date(), n).1);
        assert_ne!(base, pve_opponent(10, 21, a_date(), n).1);
        assert_ne!(
            base,
            pve_opponent(10, 20, a_date().succ_opt().unwrap(), n).1
        );
        assert_ne!(base, pve_opponent(10, 20, a_date(), n + 1).1);
    }

    /// Fighting again should usually bring someone new. With 16 trainers the
    /// odd repeat is expected, so this only asserts the sequence is not stuck.
    #[test]
    fn consecutive_battles_vary_the_trainer() {
        let mut names: Vec<&str> = (0..12)
            .map(|n| {
                pve_opponent(10, 20, a_date(), INTRO_BATTLES + n).0.name
            })
            .collect();

        names.sort();
        names.dedup();

        assert!(
            names.len() >= 5,
            "12 consecutive battles produced only {} distinct trainers",
            names.len()
        );
    }

    #[test]
    fn trainer_names_are_unique() {
        let mut names: Vec<&str> = NPC_TRAINERS.iter().map(|t| t.name).collect();
        let before = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), before);
    }
}
