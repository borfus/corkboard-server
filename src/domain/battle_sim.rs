//! The battle simulator.
//!
//! Pure functions: no database, no Discord, no clock. It takes two teams, a
//! battlefield and a seed, and returns a log. Everything random is drawn from
//! the seeded `Rng`, so `(seed, teams, battlefield)` reproduces a battle byte
//! for byte -- which is what makes stored battles replayable and bugs
//! reportable.

use super::battlefield::Battlefield;
use super::pokemon_stat::PokemonStat;
use super::rng::Rng;
use super::type_chart::{self, Type, STAB};

/// Every combatant fights at level 50. Levels were considered as a reward for
/// duplicate catches and deliberately cut: a flat level keeps a twelve-day-old
/// collection competitive against a four-hundred-day-old one.
pub const LEVEL: f64 = 50.0;

/// Base stat total every pokemon is scaled toward.
pub const TARGET_BST: f64 = 500.0;

/// How hard to pull toward `TARGET_BST`.
///
/// At 1.0 every pokemon is exactly equal and rolling a legendary means nothing.
/// At 0.0 nothing changes and the dex spans a 4x power range. 0.9 compresses
/// the real spread (Wishiwashi 175 to Arceus 720) into roughly 450-519, about
/// 15% -- enough that a legendary feels like an edge, little enough that a type
/// matchup erases it.
pub const NORMALIZATION_K: f64 = 0.9;

/// Shinies hit 5% harder across the board. The only mechanical payoff left for
/// a 1/400 roll now that duplicate levels are cut.
pub const SHINY_BONUS: f64 = 1.05;

pub const CRIT_CHANCE: f64 = 0.0625;
pub const CRIT_MULT: f64 = 1.5;

/// Damage roll spread, as in the mainline games.
pub const ROLL_LOW: f64 = 0.85;
pub const ROLL_HIGH: f64 = 1.0;

/// Two walls with no offence between them would otherwise trade chip damage
/// forever. At the cap the healthier side is declared the winner.
pub const MAX_TURNS: usize = 50;

pub const TEAM_SIZE: usize = 3;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Physical,
    Special,
}

#[derive(Serialize, Debug, Clone, Copy)]
pub struct Move {
    pub name: &'static str,
    /// `None` is typeless -- Struggle, and only Struggle. A typeless move takes
    /// no STAB, no matchup multiplier and no battlefield bonus.
    pub move_type: Option<Type>,
    pub category: Category,
    pub power: f64,
    pub accuracy: f64,
}

/// The last resort, as in the mainline games: typeless, weak, and impossible to
/// be immune to.
///
/// Only reachable when every one of a pokemon's own types is blocked by the
/// defender -- a pure Electric against Golurk's Ground/Ghost, say. Without it,
/// restricting movesets to a pokemon's own types would reintroduce matchups
/// where one side simply cannot deal damage for fifty turns.
fn struggle(category: Category) -> Move {
    Move {
        name: "Struggle",
        move_type: None,
        category,
        power: 50.0,
        accuracy: 1.0,
    }
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    A,
    B,
}

impl Side {
    pub fn other(&self) -> Side {
        match self {
            Side::A => Side::B,
            Side::B => Side::A,
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            Side::A => "a",
            Side::B => "b",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Combatant {
    pub pokemon_id: i64,
    pub name: String,
    pub shiny: bool,
    pub type_1: Type,
    pub type_2: Option<Type>,
    pub max_hp: f64,
    pub hp: f64,
    pub atk: f64,
    pub def: f64,
    pub spa: f64,
    pub spd: f64,
    pub spe: f64,
    /// Whichever attacking stat this pokemon actually has. Every move it throws
    /// uses this category, including Struggle.
    pub preferred: Category,
    pub moves: Vec<Move>,
}

impl Combatant {
    /// Builds a combatant from a seeded stat row, normalising as it goes.
    ///
    /// `difficulty` is a flat multiplier applied after normalisation; it is 1.0
    /// for every player-controlled pokemon and only varies for NPC teams.
    pub fn new(stat: &PokemonStat, shiny: bool, difficulty: f64) -> Combatant {
        let scale = (TARGET_BST / stat.bst as f64).powf(NORMALIZATION_K)
            * if shiny { SHINY_BONUS } else { 1.0 }
            * difficulty;

        // Mainline stat formulas at level 50 with no IVs or EVs, applied to the
        // normalised base stats so damage lands in a familiar range.
        let hp_stat = |base: i32| ((2.0 * (base as f64 * scale) * LEVEL) / 100.0) + LEVEL + 10.0;
        let other = |base: i32| (((2.0 * (base as f64 * scale) * LEVEL) / 100.0) + 5.0).max(1.0);

        let type_1 = stat.primary_type();
        let type_2 = stat.secondary_type().filter(|t| *t != type_1);

        let atk = other(stat.atk);
        let spa = other(stat.spa);
        let max_hp = hp_stat(stat.hp);

        let preferred = if spa > atk {
            Category::Special
        } else {
            Category::Physical
        };

        let moves = build_moveset(type_1, type_2, preferred);

        Combatant {
            pokemon_id: stat.pokemon_id,
            name: stat.name.clone(),
            shiny,
            type_1,
            type_2,
            max_hp,
            hp: max_hp,
            atk,
            def: other(stat.def),
            spa,
            spd: other(stat.spd),
            spe: other(stat.spe),
            preferred,
            moves,
        }
    }

    pub fn fainted(&self) -> bool {
        self.hp <= 0.0
    }

    /// Remaining health as a whole percentage, where **0 means fainted and
    /// nothing else**.
    ///
    /// Rounding alone would break that: a pokemon clinging on at 0.4% rounds to
    /// zero and reads as dead. Clients decide what to draw from this number --
    /// a greyed-out sprite, a crossed-out portrait -- so the invariant has to
    /// hold exactly rather than nearly.
    pub fn hp_pct(&self) -> i64 {
        if self.fainted() || self.max_hp <= 0.0 {
            return 0;
        }
        (((self.hp / self.max_hp) * 100.0).round() as i64).max(1)
    }

    pub fn has_type(&self, t: Type) -> bool {
        self.type_1 == t || self.type_2 == Some(t)
    }
}

/// The distinct types a pokemon can attack with: its own, and nothing else.
fn own_types(type_1: Type, type_2: Option<Type>) -> Vec<Type> {
    let mut types = vec![type_1];
    if let Some(t2) = type_2 {
        if t2 != type_1 {
            types.push(t2);
        }
    }
    types
}

/// Two moves per type the pokemon actually has: one reliable, one heavy.
///
/// Everything here is on-type. An earlier version gave every pokemon a Normal
/// filler move to guarantee it always had something that could connect, but a
/// Bulbasaur throwing Normal attacks reads as a bug, and an off-type move gets
/// no STAB and no battlefield bonus -- so it was both odd-looking and weak.
/// Struggle covers the blocked case instead, exactly as the games do.
///
/// Real movesets were considered and cut: they are a large data problem for
/// little payoff when nobody picks moves turn to turn. The names below are real
/// ones, mapped by type, purely for flavour.
fn build_moveset(type_1: Type, type_2: Option<Type>, preferred: Category) -> Vec<Move> {
    let mut moves = Vec::new();

    for t in own_types(type_1, type_2) {
        moves.push(Move {
            name: move_name(t, preferred, false),
            move_type: Some(t),
            category: preferred,
            power: 80.0,
            accuracy: 1.0,
        });
        // The swing move. Ends games, misses three times in ten.
        moves.push(Move {
            name: move_name(t, preferred, true),
            move_type: Some(t),
            category: preferred,
            power: 110.0,
            accuracy: 0.7,
        });
    }

    moves
}

fn move_name(t: Type, category: Category, heavy: bool) -> &'static str {
    use Type::*;

    if heavy {
        return match t {
            Normal => "Giga Impact",
            Fire => "Fire Blast",
            Water => "Hydro Pump",
            Electric => "Thunder",
            Grass => "Solar Beam",
            Ice => "Blizzard",
            Fighting => "Focus Blast",
            Poison => "Gunk Shot",
            Ground => "Precipice Blades",
            Flying => "Sky Attack",
            Psychic => "Future Sight",
            Bug => "Megahorn",
            Rock => "Stone Edge",
            Ghost => "Phantom Force",
            Dragon => "Draco Meteor",
            Dark => "Night Daze",
            Steel => "Iron Tail",
            Fairy => "Light of Ruin",
        };
    }

    match (t, category) {
        (Normal, Category::Physical) => "Body Slam",
        (Normal, Category::Special) => "Swift",
        (Fire, Category::Physical) => "Flare Blitz",
        (Fire, Category::Special) => "Flamethrower",
        (Water, Category::Physical) => "Waterfall",
        (Water, Category::Special) => "Surf",
        (Electric, Category::Physical) => "Wild Charge",
        (Electric, Category::Special) => "Thunderbolt",
        (Grass, Category::Physical) => "Power Whip",
        (Grass, Category::Special) => "Energy Ball",
        (Ice, Category::Physical) => "Ice Fang",
        (Ice, Category::Special) => "Ice Beam",
        (Fighting, Category::Physical) => "Close Combat",
        (Fighting, Category::Special) => "Aura Sphere",
        (Poison, Category::Physical) => "Poison Jab",
        (Poison, Category::Special) => "Sludge Bomb",
        (Ground, Category::Physical) => "Earthquake",
        (Ground, Category::Special) => "Earth Power",
        (Flying, Category::Physical) => "Brave Bird",
        (Flying, Category::Special) => "Air Slash",
        (Psychic, Category::Physical) => "Zen Headbutt",
        (Psychic, Category::Special) => "Psychic",
        (Bug, Category::Physical) => "X-Scissor",
        (Bug, Category::Special) => "Bug Buzz",
        (Rock, Category::Physical) => "Rock Slide",
        (Rock, Category::Special) => "Power Gem",
        (Ghost, Category::Physical) => "Shadow Claw",
        (Ghost, Category::Special) => "Shadow Ball",
        (Dragon, Category::Physical) => "Dragon Claw",
        (Dragon, Category::Special) => "Dragon Pulse",
        (Dark, Category::Physical) => "Crunch",
        (Dark, Category::Special) => "Dark Pulse",
        (Steel, Category::Physical) => "Iron Head",
        (Steel, Category::Special) => "Flash Cannon",
        (Fairy, Category::Physical) => "Play Rough",
        (Fairy, Category::Special) => "Moonblast",
    }
}

// ---------------------------------------------------------------------------
// Log types
// ---------------------------------------------------------------------------

#[derive(Serialize, Debug, Clone)]
pub struct MonSnapshot {
    pub slot: usize,
    pub pokemon_id: i64,
    pub name: String,
    pub shiny: bool,
    pub type_1: String,
    pub type_2: Option<String>,
    pub max_hp: i64,
}

#[derive(Serialize, Debug, Clone)]
pub struct HpSnapshot {
    pub a: Vec<i64>,
    pub b: Vec<i64>,
}

#[derive(Serialize, Debug, Clone)]
pub struct TurnEvent {
    pub n: usize,
    pub actor: &'static str,
    pub mon: i64,
    pub mon_name: String,
    pub move_name: &'static str,
    pub move_type: String,
    pub target: i64,
    pub target_name: String,
    pub dmg: i64,
    pub eff: f64,
    pub crit: bool,
    pub missed: bool,
    pub fainted: bool,
    /// Pre-rendered line so the bot can play the battle back without knowing
    /// any of the rules.
    pub text: String,
    pub hp_after: HpSnapshot,
}

#[derive(Serialize, Debug, Clone)]
pub struct BattleLog {
    pub battlefield: &'static str,
    pub battlefield_name: &'static str,
    pub battlefield_emoji: &'static str,
    pub battlefield_summary: String,
    pub team_a: Vec<MonSnapshot>,
    pub team_b: Vec<MonSnapshot>,
    pub turns: Vec<TurnEvent>,
    pub winner: &'static str,
    pub total_turns: usize,
    /// True when the turn cap decided it rather than a knockout.
    pub decided_on_hp: bool,
}

// ---------------------------------------------------------------------------
// Simulation
// ---------------------------------------------------------------------------

fn snapshot(team: &[Combatant]) -> Vec<MonSnapshot> {
    team.iter()
        .enumerate()
        .map(|(i, c)| MonSnapshot {
            slot: i,
            pokemon_id: c.pokemon_id,
            name: c.name.clone(),
            shiny: c.shiny,
            type_1: c.type_1.name().to_string(),
            type_2: c.type_2.map(|t| t.name().to_string()),
            max_hp: c.max_hp.round() as i64,
        })
        .collect()
}

fn hp_snapshot(a: &[Combatant], b: &[Combatant]) -> HpSnapshot {
    HpSnapshot {
        a: a.iter().map(|c| c.hp_pct()).collect(),
        b: b.iter().map(|c| c.hp_pct()).collect(),
    }
}

fn active_index(team: &[Combatant]) -> Option<usize> {
    team.iter().position(|c| !c.fainted())
}

/// How well a move lands on this defender. Typeless moves are always neutral.
fn move_effectiveness(mv: &Move, defender: &Combatant) -> f64 {
    match mv.move_type {
        Some(t) => type_chart::effectiveness_against(t, defender.type_1, defender.type_2),
        None => 1.0,
    }
}

/// Raw damage before the random roll and crit.
fn base_damage(attacker: &Combatant, defender: &Combatant, mv: &Move, bf: &Battlefield) -> f64 {
    let (a, d) = match mv.category {
        Category::Physical => (attacker.atk, defender.def),
        Category::Special => (attacker.spa, defender.spd),
    };

    let raw = (((2.0 * LEVEL / 5.0 + 2.0) * mv.power * a / d) / 50.0) + 2.0;

    // Struggle is typeless: no same-type bonus, no matchup, no weather.
    let (stab, field) = match mv.move_type {
        Some(t) => (
            if attacker.has_type(t) { STAB } else { 1.0 },
            bf.multiplier(t),
        ),
        None => (1.0, 1.0),
    };

    raw * stab * move_effectiveness(mv, defender) * field
}

/// Picks this turn's move.
///
/// A dual type chooses which of its two types to attack with at random, rather
/// than settling on whichever scores fractionally higher. Always taking the
/// arithmetic best meant a Grass/Poison threw Sludge Bomb every single turn and
/// never once used Energy Ball, which reads as broken even though the numbers
/// were right -- and it meant a Water type could ignore its Water move in a
/// Downpour. Randomising the type keeps both halves of a pokemon in play and
/// lets the battlefield matter.
///
/// Types the defender is outright immune to are dropped first, so the roll
/// never wastes a turn on something that cannot land. When nothing at all
/// connects, that is what Struggle is for.
fn choose_move(
    attacker: &Combatant,
    defender: &Combatant,
    _bf: &Battlefield,
    rng: &mut Rng,
) -> Move {
    let usable: Vec<&Move> = attacker
        .moves
        .iter()
        .filter(|m| move_effectiveness(m, defender) > 0.0)
        .collect();

    if usable.is_empty() {
        return struggle(attacker.preferred);
    }

    // Distinct types still on the table, in a stable order so the draw is
    // reproducible from the seed.
    let mut types: Vec<Type> = Vec::new();
    for mv in usable.iter() {
        if let Some(t) = mv.move_type {
            if !types.contains(&t) {
                types.push(t);
            }
        }
    }

    let chosen = types[rng.below(types.len() as u64) as usize];

    // Within the chosen type, pick between the reliable move and the heavy one.
    // This is a coin flip rather than a comparison on purpose: at 80 power with
    // full accuracy against 110 at 70%, expected damage is near enough
    // identical that a comparison would always land on the same one and leave
    // the other permanently unused.
    let of_type: Vec<&Move> = usable
        .iter()
        .filter(|m| m.move_type == Some(chosen))
        .copied()
        .collect();

    *of_type[rng.below(of_type.len() as u64) as usize]
}

fn render_line(
    attacker_name: &str,
    target_name: &str,
    mv: &Move,
    dmg: i64,
    eff: f64,
    crit: bool,
    missed: bool,
    fainted: bool,
) -> String {
    let attacker = title_case(attacker_name);
    let target = title_case(target_name);

    if missed {
        return format!("{} used {}! It missed!", attacker, mv.name);
    }

    let mut line = format!("{} used {}!", attacker, mv.name);

    if let Some(text) = type_chart::effectiveness_text(eff) {
        line.push(' ');
        line.push_str(text);
    }

    if eff == 0.0 {
        return line;
    }

    if crit {
        line.push_str(" A critical hit!");
    }

    line.push_str(&format!(" {} took {} damage.", target, dmg));

    if fainted {
        line.push_str(&format!(" {} fainted!", target));
    }

    line
}

/// Dataset names are lowercase and hyphenated ("mr-mime"). The bot has richer
/// display formatting; this is just so the stored log reads properly on its own.
fn title_case(name: &str) -> String {
    name.split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn team_hp_fraction(team: &[Combatant]) -> f64 {
    let total: f64 = team.iter().map(|c| c.max_hp).sum();
    if total <= 0.0 {
        return 0.0;
    }
    team.iter().map(|c| c.hp.max(0.0)).sum::<f64>() / total
}

/// Runs a full battle. `team_a` is the challenger.
pub fn simulate(
    mut team_a: Vec<Combatant>,
    mut team_b: Vec<Combatant>,
    bf: &'static Battlefield,
    seed: u64,
) -> BattleLog {
    let mut rng = Rng::new(seed);

    let snap_a = snapshot(&team_a);
    let snap_b = snapshot(&team_b);
    let mut turns: Vec<TurnEvent> = Vec::new();

    let mut turn_number = 0usize;
    let mut decided_on_hp = false;

    let winner = loop {
        if active_index(&team_a).is_none() {
            break Side::B;
        }
        if active_index(&team_b).is_none() {
            break Side::A;
        }

        if turn_number >= MAX_TURNS {
            decided_on_hp = true;
            let fa = team_hp_fraction(&team_a);
            let fb = team_hp_fraction(&team_b);
            break if fa > fb {
                Side::A
            } else if fb > fa {
                Side::B
            } else if rng.chance(0.5) {
                // Exact ties are all but impossible with float HP, but the
                // model has no room for a draw, so break it deterministically.
                Side::A
            } else {
                Side::B
            };
        }

        turn_number += 1;

        // Turn order: faster active mon goes first, seeded coin flip on a tie.
        let ia = active_index(&team_a).unwrap();
        let ib = active_index(&team_b).unwrap();
        let first = if team_a[ia].spe > team_b[ib].spe {
            Side::A
        } else if team_b[ib].spe > team_a[ia].spe {
            Side::B
        } else if rng.chance(0.5) {
            Side::A
        } else {
            Side::B
        };

        for side in [first, first.other()].iter() {
            // The side that moved first may have just knocked the other out.
            if active_index(&team_a).is_none() || active_index(&team_b).is_none() {
                break;
            }

            let (attacker_idx, defender_idx) = match side {
                Side::A => (active_index(&team_a).unwrap(), active_index(&team_b).unwrap()),
                Side::B => (active_index(&team_b).unwrap(), active_index(&team_a).unwrap()),
            };

            let (attacker, defender) = match side {
                Side::A => (team_a[attacker_idx].clone(), team_b[defender_idx].clone()),
                Side::B => (team_b[attacker_idx].clone(), team_a[defender_idx].clone()),
            };

            let mv = choose_move(&attacker, &defender, bf, &mut rng);

            let eff = move_effectiveness(&mv, &defender);
            let missed = !rng.chance(mv.accuracy);

            let (dmg, crit) = if missed || eff == 0.0 {
                (0.0, false)
            } else {
                let crit = rng.chance(CRIT_CHANCE);
                let roll = rng.range(ROLL_LOW, ROLL_HIGH);
                let d = base_damage(&attacker, &defender, &mv, bf)
                    * if crit { CRIT_MULT } else { 1.0 }
                    * roll;
                (d.max(1.0), crit)
            };

            // Apply.
            let target_team = match side {
                Side::A => &mut team_b,
                Side::B => &mut team_a,
            };
            target_team[defender_idx].hp -= dmg;
            let fainted = target_team[defender_idx].fainted();
            if fainted {
                target_team[defender_idx].hp = 0.0;
            }

            let dmg_i = dmg.round() as i64;
            let text = render_line(
                &attacker.name,
                &defender.name,
                &mv,
                dmg_i,
                eff,
                crit,
                missed,
                fainted,
            );

            turns.push(TurnEvent {
                n: turn_number,
                actor: side.key(),
                mon: attacker.pokemon_id,
                mon_name: attacker.name.clone(),
                move_name: mv.name,
                move_type: mv
                    .move_type
                    .map(|t| t.name().to_string())
                    .unwrap_or_else(|| "typeless".to_string()),
                target: defender.pokemon_id,
                target_name: defender.name.clone(),
                dmg: dmg_i,
                eff,
                crit,
                missed,
                fainted,
                text,
                hp_after: hp_snapshot(&team_a, &team_b),
            });
        }
    };

    BattleLog {
        battlefield: bf.key,
        battlefield_name: bf.name,
        battlefield_emoji: bf.emoji,
        battlefield_summary: bf.summary(),
        team_a: snap_a,
        team_b: snap_b,
        turns,
        winner: winner.key(),
        total_turns: turn_number,
        decided_on_hp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::battlefield;

    fn stat(pokemon_id: i64, name: &str, s: [i32; 6], t1: &str, t2: Option<&str>) -> PokemonStat {
        PokemonStat {
            pokemon_id,
            name: name.to_string(),
            hp: s[0],
            atk: s[1],
            def: s[2],
            spa: s[3],
            spd: s[4],
            spe: s[5],
            type_1: t1.to_string(),
            type_2: t2.map(|x| x.to_string()),
            bst: s.iter().sum(),
        }
    }

    fn charizard() -> PokemonStat {
        stat(6, "charizard", [78, 84, 78, 109, 85, 100], "fire", Some("flying"))
    }

    fn blastoise() -> PokemonStat {
        stat(9, "blastoise", [79, 83, 100, 85, 105, 78], "water", None)
    }

    fn magikarp() -> PokemonStat {
        stat(129, "magikarp", [20, 10, 55, 15, 20, 80], "water", None)
    }

    fn mewtwo() -> PokemonStat {
        stat(150, "mewtwo", [106, 110, 90, 154, 90, 130], "psychic", None)
    }

    fn field() -> &'static Battlefield {
        battlefield::by_key("town_square").unwrap()
    }

    #[test]
    fn normalization_compresses_the_power_range() {
        let weak = Combatant::new(&magikarp(), false, 1.0);
        let strong = Combatant::new(&mewtwo(), false, 1.0);

        let weak_total = weak.atk + weak.def + weak.spa + weak.spd + weak.spe;
        let strong_total = strong.atk + strong.def + strong.spa + strong.spd + strong.spe;

        // Raw BSTs differ by 3.4x; after normalisation the gap must be small.
        let ratio = strong_total / weak_total;
        assert!(
            ratio > 1.0 && ratio < 1.35,
            "expected a modest gap after normalisation, got {}",
            ratio
        );
    }

    #[test]
    fn normalization_preserves_stat_shape() {
        // Magikarp's defining trait is that speed is its best stat by far.
        let karp = Combatant::new(&magikarp(), false, 1.0);
        assert!(karp.spe > karp.atk);
        assert!(karp.spe > karp.spa);
        assert!(karp.def > karp.atk);
    }

    #[test]
    fn shiny_is_a_small_edge() {
        let plain = Combatant::new(&charizard(), false, 1.0);
        let shiny = Combatant::new(&charizard(), true, 1.0);
        assert!(shiny.atk > plain.atk);
        let ratio = shiny.atk / plain.atk;
        assert!(ratio > 1.03 && ratio < 1.07, "got {}", ratio);
    }

    #[test]
    fn moveset_is_entirely_on_type() {
        // Fire/Flying: two moves per type, and nothing off-type.
        let zard = Combatant::new(&charizard(), false, 1.0);
        assert_eq!(zard.moves.len(), 4);
        assert!(zard.moves.iter().any(|m| m.move_type == Some(Type::Fire)));
        assert!(zard.moves.iter().any(|m| m.move_type == Some(Type::Flying)));
        assert!(
            zard.moves
                .iter()
                .all(|m| m.move_type == Some(Type::Fire) || m.move_type == Some(Type::Flying)),
            "a pokemon should only ever throw its own types"
        );

        // Mono-typed: one type, still two moves, still nothing borrowed.
        let karp = Combatant::new(&magikarp(), false, 1.0);
        assert_eq!(karp.moves.len(), 2);
        assert!(karp.moves.iter().all(|m| m.move_type == Some(Type::Water)));
    }

    /// The behaviour that prompted this: a Grass/Poison must not settle on one
    /// half of its typing and never touch the other.
    #[test]
    fn a_dual_type_uses_both_of_its_types() {
        let bulbasaur = stat(1, "bulbasaur", [45, 49, 49, 65, 65, 45], "grass", Some("poison"));
        let electivire = stat(466, "electivire", [75, 123, 67, 95, 85, 95], "electric", None);

        let attacker = Combatant::new(&bulbasaur, false, 1.0);
        let defender = Combatant::new(&electivire, false, 1.0);

        let mut rng = Rng::new(7);
        let mut seen_grass = false;
        let mut seen_poison = false;

        for _ in 0..100 {
            match choose_move(&attacker, &defender, field(), &mut rng).move_type {
                Some(Type::Grass) => seen_grass = true,
                Some(Type::Poison) => seen_poison = true,
                other => panic!("bulbasaur threw an off-type move: {:?}", other),
            }
        }

        assert!(
            seen_grass && seen_poison,
            "expected both types over 100 turns (grass: {}, poison: {})",
            seen_grass,
            seen_poison
        );
    }

    /// A type the defender is immune to is dropped before the roll, so the turn
    /// is never wasted on something that cannot land.
    #[test]
    fn an_immune_type_is_never_chosen() {
        // Gengar is Ghost/Poison. Steel is immune to Poison, so a Gengar facing
        // a Steel type has only its Ghost half available.
        let gengar = stat(94, "gengar", [60, 65, 60, 130, 75, 110], "ghost", Some("poison"));
        let steelix = stat(208, "steelix", [75, 85, 200, 55, 65, 30], "steel", Some("ground"));

        let attacker = Combatant::new(&gengar, false, 1.0);
        let defender = Combatant::new(&steelix, false, 1.0);

        let mut rng = Rng::new(11);
        for _ in 0..100 {
            let mv = choose_move(&attacker, &defender, field(), &mut rng);
            assert_eq!(
                mv.move_type,
                Some(Type::Ghost),
                "poison cannot touch steel and should never be rolled"
            );
        }
    }

    /// With movesets restricted to a pokemon's own types, a fully blocked
    /// matchup has to fall through to Struggle -- otherwise it would be fifty
    /// turns of nothing.
    #[test]
    fn a_fully_blocked_pokemon_struggles() {
        // Snorlax is pure Normal; Gengar's Ghost half blocks Normal outright.
        let gengar = stat(94, "gengar", [60, 65, 60, 130, 75, 110], "ghost", Some("poison"));
        let snorlax = stat(143, "snorlax", [160, 110, 65, 65, 110, 30], "normal", None);

        let attacker = Combatant::new(&snorlax, false, 1.0);
        let defender = Combatant::new(&gengar, false, 1.0);

        let mut rng = Rng::new(1);
        let chosen = choose_move(&attacker, &defender, field(), &mut rng);

        assert_eq!(chosen.name, "Struggle");
        assert_eq!(chosen.move_type, None);
        assert!(
            move_effectiveness(&chosen, &defender) > 0.0,
            "struggle must always be able to land"
        );
    }

    #[test]
    fn battles_are_reproducible() {
        let a = vec![
            Combatant::new(&charizard(), false, 1.0),
            Combatant::new(&mewtwo(), false, 1.0),
            Combatant::new(&magikarp(), false, 1.0),
        ];
        let b = vec![
            Combatant::new(&blastoise(), false, 1.0),
            Combatant::new(&magikarp(), true, 1.0),
            Combatant::new(&charizard(), false, 1.0),
        ];

        let one = simulate(a.clone(), b.clone(), field(), 12345);
        let two = simulate(a, b, field(), 12345);

        assert_eq!(one.winner, two.winner);
        assert_eq!(one.turns.len(), two.turns.len());
        for (x, y) in one.turns.iter().zip(two.turns.iter()) {
            assert_eq!(x.text, y.text);
            assert_eq!(x.dmg, y.dmg);
        }
    }

    #[test]
    fn different_seeds_can_diverge() {
        let a = vec![
            Combatant::new(&charizard(), false, 1.0),
            Combatant::new(&blastoise(), false, 1.0),
            Combatant::new(&magikarp(), false, 1.0),
        ];
        let b = vec![
            Combatant::new(&blastoise(), false, 1.0),
            Combatant::new(&charizard(), false, 1.0),
            Combatant::new(&mewtwo(), false, 1.0),
        ];

        // Turn counts are too coarse to detect this: the damage roll only spans
        // 0.85-1.0, which often is not enough to change how many hits a KO
        // takes. Compare the damage actually dealt instead.
        let totals: Vec<i64> = (0..8)
            .map(|s| {
                simulate(a.clone(), b.clone(), field(), s)
                    .turns
                    .iter()
                    .map(|t| t.dmg)
                    .sum()
            })
            .collect();

        assert!(
            totals.windows(2).any(|w| w[0] != w[1]),
            "eight seeds produced identical damage: {:?}",
            totals
        );
    }

    /// Whatever a pokemon is up against, the move it actually throws has to be
    /// able to land -- on-type where possible, Struggle where not. Exhaustive
    /// over every attacker and defender typing in the chart.
    #[test]
    fn every_matchup_produces_a_move_that_connects() {
        use crate::domain::type_chart::ALL_TYPES;

        let mut rng = Rng::new(4242);

        for atk_1 in ALL_TYPES.iter() {
            // Both mono-typed and dual-typed attackers.
            let attacker_types: Vec<(Type, Option<Type>)> = std::iter::once((*atk_1, None))
                .chain(ALL_TYPES.iter().filter(|t| *t != atk_1).map(|t2| (*atk_1, Some(*t2))))
                .collect();

            for (t1, t2) in attacker_types {
                let attacker = Combatant::new(
                    &stat(1, "tester", [80, 80, 80, 80, 80, 80], t1.name(), t2.map(|t| t.name())),
                    false,
                    1.0,
                );

                for def_1 in ALL_TYPES.iter() {
                    let def_seconds = ALL_TYPES
                        .iter()
                        .copied()
                        .map(Some)
                        .chain(std::iter::once(None));

                    for def_2 in def_seconds {
                        let defender = Combatant::new(
                            &stat(
                                2,
                                "target",
                                [80, 80, 80, 80, 80, 80],
                                def_1.name(),
                                def_2.map(|t| t.name()),
                            ),
                            false,
                            1.0,
                        );

                        let mv = choose_move(&attacker, &defender, field(), &mut rng);

                        assert!(
                            move_effectiveness(&mv, &defender) > 0.0,
                            "{}{} threw {} at {}{} and it could not land",
                            t1.name(),
                            t2.map(|t| format!("/{}", t.name())).unwrap_or_default(),
                            mv.name,
                            def_1.name(),
                            def_2.map(|t| format!("/{}", t.name())).unwrap_or_default(),
                        );

                        // On-type whenever anything on-type can reach.
                        if let Some(mt) = mv.move_type {
                            assert!(
                                attacker.has_type(mt),
                                "{}{} threw an off-type {}",
                                t1.name(),
                                t2.map(|t| format!("/{}", t.name())).unwrap_or_default(),
                                mt.name(),
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn battles_terminate_and_produce_a_winner() {
        // Two Shuckles: maximum stall, so this exercises the turn cap.
        let shuckle = stat(213, "shuckle", [20, 10, 230, 10, 230, 5], "bug", Some("rock"));
        let a = vec![
            Combatant::new(&shuckle, false, 1.0),
            Combatant::new(&shuckle, false, 1.0),
            Combatant::new(&shuckle, false, 1.0),
        ];
        let b = a.clone();

        let log = simulate(a, b, field(), 999);
        assert!(log.total_turns <= MAX_TURNS);
        assert!(log.winner == "a" || log.winner == "b");
    }

    #[test]
    fn a_battle_never_exceeds_the_turn_cap() {
        let a = vec![Combatant::new(&charizard(), false, 1.0)];
        let b = vec![Combatant::new(&blastoise(), false, 1.0)];
        let log = simulate(a, b, field(), 7);
        assert!(log.total_turns <= MAX_TURNS);
        assert!(!log.turns.is_empty());
    }

    #[test]
    fn battlefield_boosts_matching_moves() {
        let downpour = battlefield::by_key("downpour").unwrap();
        let neutral = battlefield::by_key("town_square").unwrap();

        let attacker = Combatant::new(&blastoise(), false, 1.0);
        let defender = Combatant::new(&mewtwo(), false, 1.0);

        let surf = attacker
            .moves
            .iter()
            .find(|m| m.move_type == Some(Type::Water) && m.power == 80.0)
            .unwrap();

        let wet = base_damage(&attacker, &defender, surf, downpour);
        let dry = base_damage(&attacker, &defender, surf, neutral);

        assert!(wet > dry);
        assert!((wet / dry - 1.30).abs() < 0.001);
    }

    #[test]
    fn log_lines_read_properly() {
        let a = vec![Combatant::new(&charizard(), false, 1.0)];
        let b = vec![Combatant::new(&blastoise(), false, 1.0)];
        let log = simulate(a, b, field(), 3);

        let first = &log.turns[0];
        assert!(first.text.contains("used"), "got: {}", first.text);
        assert!(
            first.text.starts_with("Charizard") || first.text.starts_with("Blastoise"),
            "got: {}",
            first.text
        );
    }

    #[test]
    fn title_case_handles_hyphenated_names() {
        assert_eq!(title_case("mr-mime"), "Mr Mime");
        assert_eq!(title_case("pikachu"), "Pikachu");
        assert_eq!(title_case("iron-crown"), "Iron Crown");
    }

    /// Zero percent has to mean fainted and only fainted, because that is what
    /// the bot keys its faint markers off.
    #[test]
    fn zero_percent_means_fainted_and_nothing_else() {
        let mut mon = Combatant::new(&charizard(), false, 1.0);

        // Barely alive still reports at least 1%.
        mon.hp = mon.max_hp * 0.004;
        assert!(!mon.fainted());
        assert_eq!(mon.hp_pct(), 1, "a living pokemon must never report 0%");

        mon.hp = 0.0;
        assert!(mon.fainted());
        assert_eq!(mon.hp_pct(), 0);

        mon.hp = -50.0;
        assert_eq!(mon.hp_pct(), 0);
    }

    /// Same invariant, but across a whole battle rather than a hand-set value.
    #[test]
    fn hp_snapshots_only_report_zero_for_knockouts() {
        let a = vec![
            Combatant::new(&charizard(), false, 1.0),
            Combatant::new(&mewtwo(), false, 1.0),
        ];
        let b = vec![
            Combatant::new(&blastoise(), false, 1.0),
            Combatant::new(&magikarp(), false, 1.0),
        ];

        let log = simulate(a, b, field(), 31337);

        // A slot that reads 0 must stay 0 -- nothing comes back from fainted.
        let mut dead_a = vec![false; log.team_a.len()];
        for turn in log.turns.iter() {
            for (i, pct) in turn.hp_after.a.iter().enumerate() {
                if dead_a[i] {
                    assert_eq!(*pct, 0, "slot {} revived after fainting", i);
                }
                if *pct == 0 {
                    dead_a[i] = true;
                }
            }
        }
    }

    #[test]
    fn hp_snapshots_track_every_slot() {
        let a = vec![
            Combatant::new(&charizard(), false, 1.0),
            Combatant::new(&mewtwo(), false, 1.0),
            Combatant::new(&magikarp(), false, 1.0),
        ];
        let b = vec![
            Combatant::new(&blastoise(), false, 1.0),
            Combatant::new(&charizard(), false, 1.0),
            Combatant::new(&magikarp(), false, 1.0),
        ];

        let log = simulate(a, b, field(), 55);
        for t in log.turns.iter() {
            assert_eq!(t.hp_after.a.len(), 3);
            assert_eq!(t.hp_after.b.len(), 3);
            assert!(t.hp_after.a.iter().all(|p| *p >= 0 && *p <= 100));
            assert!(t.hp_after.b.iter().all(|p| *p >= 0 && *p <= 100));
        }
    }
}
