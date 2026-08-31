//! The 18x18 type effectiveness chart (generation 6+), plus the multipliers
//! luckybattle uses in place of the mainline ones.
//!
//! Kept as code rather than a table: it is 324 immutable entries that have not
//! changed since 2013, and a table would add a join to every damage roll.
//!
//! The relationships are written out per attacking type so they can be read
//! against a published chart line by line. Anything not listed is neutral.

use std::fmt;

/// Multiplier for a super effective hit.
///
/// Deliberately above the mainline 2.0: in a 3v3 auto-battle nobody is picking
/// moves turn to turn, so typing is where the drama has to come from. Raising
/// this (and dropping the resisted multiplier below 0.5) makes matchups read
/// clearly in the log instead of blurring into a damage race.
pub const SUPER_EFFECTIVE: f64 = 2.25;

/// Multiplier for a resisted hit. Mirror of `SUPER_EFFECTIVE`, below the
/// mainline 0.5 for the same reason.
pub const NOT_VERY_EFFECTIVE: f64 = 0.45;

pub const NEUTRAL: f64 = 1.0;
pub const IMMUNE: f64 = 0.0;

/// Same-type attack bonus, unchanged from the mainline games.
pub const STAB: f64 = 1.5;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Normal,
    Fire,
    Water,
    Electric,
    Grass,
    Ice,
    Fighting,
    Poison,
    Ground,
    Flying,
    Psychic,
    Bug,
    Rock,
    Ghost,
    Dragon,
    Dark,
    Steel,
    Fairy,
}

/// Every type, for walking the whole chart. Exercised by the exhaustive
/// coverage tests rather than by the simulator, which only ever looks up
/// specific pairs.
#[allow(dead_code)]
pub const ALL_TYPES: [Type; 18] = [
    Type::Normal,
    Type::Fire,
    Type::Water,
    Type::Electric,
    Type::Grass,
    Type::Ice,
    Type::Fighting,
    Type::Poison,
    Type::Ground,
    Type::Flying,
    Type::Psychic,
    Type::Bug,
    Type::Rock,
    Type::Ghost,
    Type::Dragon,
    Type::Dark,
    Type::Steel,
    Type::Fairy,
];

impl Type {
    /// Parses the lowercase identifiers stored in `pokemon_stat.type_1` /
    /// `type_2`, which come straight from the PokeAPI dataset.
    pub fn from_name(name: &str) -> Option<Type> {
        let t = match name {
            "normal" => Type::Normal,
            "fire" => Type::Fire,
            "water" => Type::Water,
            "electric" => Type::Electric,
            "grass" => Type::Grass,
            "ice" => Type::Ice,
            "fighting" => Type::Fighting,
            "poison" => Type::Poison,
            "ground" => Type::Ground,
            "flying" => Type::Flying,
            "psychic" => Type::Psychic,
            "bug" => Type::Bug,
            "rock" => Type::Rock,
            "ghost" => Type::Ghost,
            "dragon" => Type::Dragon,
            "dark" => Type::Dark,
            "steel" => Type::Steel,
            "fairy" => Type::Fairy,
            _ => return None,
        };
        Some(t)
    }

    pub fn name(&self) -> &'static str {
        match self {
            Type::Normal => "normal",
            Type::Fire => "fire",
            Type::Water => "water",
            Type::Electric => "electric",
            Type::Grass => "grass",
            Type::Ice => "ice",
            Type::Fighting => "fighting",
            Type::Poison => "poison",
            Type::Ground => "ground",
            Type::Flying => "flying",
            Type::Psychic => "psychic",
            Type::Bug => "bug",
            Type::Rock => "rock",
            Type::Ghost => "ghost",
            Type::Dragon => "dragon",
            Type::Dark => "dark",
            Type::Steel => "steel",
            Type::Fairy => "fairy",
        }
    }

    /// Title-cased, for move names and log lines.
    pub fn display(&self) -> &'static str {
        match self {
            Type::Normal => "Normal",
            Type::Fire => "Fire",
            Type::Water => "Water",
            Type::Electric => "Electric",
            Type::Grass => "Grass",
            Type::Ice => "Ice",
            Type::Fighting => "Fighting",
            Type::Poison => "Poison",
            Type::Ground => "Ground",
            Type::Flying => "Flying",
            Type::Psychic => "Psychic",
            Type::Bug => "Bug",
            Type::Rock => "Rock",
            Type::Ghost => "Ghost",
            Type::Dragon => "Dragon",
            Type::Dark => "Dark",
            Type::Steel => "Steel",
            Type::Fairy => "Fairy",
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// Effectiveness of `atk` against a single defending type.
///
/// Grouped by attacking type, in the same order as `ALL_TYPES`, so each arm can
/// be checked against a published chart row.
pub fn effectiveness(atk: Type, def: Type) -> f64 {
    use Type::*;

    match (atk, def) {
        // Normal
        (Normal, Rock) | (Normal, Steel) => NOT_VERY_EFFECTIVE,
        (Normal, Ghost) => IMMUNE,

        // Fire
        (Fire, Grass) | (Fire, Ice) | (Fire, Bug) | (Fire, Steel) => SUPER_EFFECTIVE,
        (Fire, Fire) | (Fire, Water) | (Fire, Rock) | (Fire, Dragon) => NOT_VERY_EFFECTIVE,

        // Water
        (Water, Fire) | (Water, Ground) | (Water, Rock) => SUPER_EFFECTIVE,
        (Water, Water) | (Water, Grass) | (Water, Dragon) => NOT_VERY_EFFECTIVE,

        // Electric
        (Electric, Water) | (Electric, Flying) => SUPER_EFFECTIVE,
        (Electric, Electric) | (Electric, Grass) | (Electric, Dragon) => NOT_VERY_EFFECTIVE,
        (Electric, Ground) => IMMUNE,

        // Grass
        (Grass, Water) | (Grass, Ground) | (Grass, Rock) => SUPER_EFFECTIVE,
        (Grass, Fire)
        | (Grass, Grass)
        | (Grass, Poison)
        | (Grass, Flying)
        | (Grass, Bug)
        | (Grass, Dragon)
        | (Grass, Steel) => NOT_VERY_EFFECTIVE,

        // Ice
        (Ice, Grass) | (Ice, Ground) | (Ice, Flying) | (Ice, Dragon) => SUPER_EFFECTIVE,
        (Ice, Fire) | (Ice, Water) | (Ice, Ice) | (Ice, Steel) => NOT_VERY_EFFECTIVE,

        // Fighting
        (Fighting, Normal)
        | (Fighting, Ice)
        | (Fighting, Rock)
        | (Fighting, Dark)
        | (Fighting, Steel) => SUPER_EFFECTIVE,
        (Fighting, Poison)
        | (Fighting, Flying)
        | (Fighting, Psychic)
        | (Fighting, Bug)
        | (Fighting, Fairy) => NOT_VERY_EFFECTIVE,
        (Fighting, Ghost) => IMMUNE,

        // Poison
        (Poison, Grass) | (Poison, Fairy) => SUPER_EFFECTIVE,
        (Poison, Poison) | (Poison, Ground) | (Poison, Rock) | (Poison, Ghost) => {
            NOT_VERY_EFFECTIVE
        }
        (Poison, Steel) => IMMUNE,

        // Ground
        (Ground, Fire) | (Ground, Electric) | (Ground, Poison) | (Ground, Rock)
        | (Ground, Steel) => SUPER_EFFECTIVE,
        (Ground, Grass) | (Ground, Bug) => NOT_VERY_EFFECTIVE,
        (Ground, Flying) => IMMUNE,

        // Flying
        (Flying, Grass) | (Flying, Fighting) | (Flying, Bug) => SUPER_EFFECTIVE,
        (Flying, Electric) | (Flying, Rock) | (Flying, Steel) => NOT_VERY_EFFECTIVE,

        // Psychic
        (Psychic, Fighting) | (Psychic, Poison) => SUPER_EFFECTIVE,
        (Psychic, Psychic) | (Psychic, Steel) => NOT_VERY_EFFECTIVE,
        (Psychic, Dark) => IMMUNE,

        // Bug
        (Bug, Grass) | (Bug, Psychic) | (Bug, Dark) => SUPER_EFFECTIVE,
        (Bug, Fire)
        | (Bug, Fighting)
        | (Bug, Poison)
        | (Bug, Flying)
        | (Bug, Ghost)
        | (Bug, Steel)
        | (Bug, Fairy) => NOT_VERY_EFFECTIVE,

        // Rock
        (Rock, Fire) | (Rock, Ice) | (Rock, Flying) | (Rock, Bug) => SUPER_EFFECTIVE,
        (Rock, Fighting) | (Rock, Ground) | (Rock, Steel) => NOT_VERY_EFFECTIVE,

        // Ghost
        (Ghost, Psychic) | (Ghost, Ghost) => SUPER_EFFECTIVE,
        (Ghost, Dark) => NOT_VERY_EFFECTIVE,
        (Ghost, Normal) => IMMUNE,

        // Dragon
        (Dragon, Dragon) => SUPER_EFFECTIVE,
        (Dragon, Steel) => NOT_VERY_EFFECTIVE,
        (Dragon, Fairy) => IMMUNE,

        // Dark
        (Dark, Psychic) | (Dark, Ghost) => SUPER_EFFECTIVE,
        (Dark, Fighting) | (Dark, Dark) | (Dark, Fairy) => NOT_VERY_EFFECTIVE,

        // Steel
        (Steel, Ice) | (Steel, Rock) | (Steel, Fairy) => SUPER_EFFECTIVE,
        (Steel, Fire) | (Steel, Water) | (Steel, Electric) | (Steel, Steel) => {
            NOT_VERY_EFFECTIVE
        }

        // Fairy
        (Fairy, Fighting) | (Fairy, Dragon) | (Fairy, Dark) => SUPER_EFFECTIVE,
        (Fairy, Fire) | (Fairy, Poison) | (Fairy, Steel) => NOT_VERY_EFFECTIVE,

        _ => NEUTRAL,
    }
}

/// Effectiveness against a defender's full typing. Multiplies both slots, so a
/// dual type can stack to 5.06x or bottom out at 0.2x.
pub fn effectiveness_against(atk: Type, def_1: Type, def_2: Option<Type>) -> f64 {
    let mut mult = effectiveness(atk, def_1);
    if let Some(d2) = def_2 {
        mult *= effectiveness(atk, d2);
    }
    mult
}

/// Log line fragment for a hit, or `None` when the hit was neutral.
pub fn effectiveness_text(mult: f64) -> Option<&'static str> {
    if mult == 0.0 {
        Some("It had no effect!")
    } else if mult > 1.0 {
        Some("It's super effective!")
    } else if mult < 1.0 {
        Some("It's not very effective...")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn immunities_hold() {
        assert_eq!(effectiveness(Type::Normal, Type::Ghost), IMMUNE);
        assert_eq!(effectiveness(Type::Ghost, Type::Normal), IMMUNE);
        assert_eq!(effectiveness(Type::Electric, Type::Ground), IMMUNE);
        assert_eq!(effectiveness(Type::Ground, Type::Flying), IMMUNE);
        assert_eq!(effectiveness(Type::Poison, Type::Steel), IMMUNE);
        assert_eq!(effectiveness(Type::Psychic, Type::Dark), IMMUNE);
        assert_eq!(effectiveness(Type::Dragon, Type::Fairy), IMMUNE);
        assert_eq!(effectiveness(Type::Fighting, Type::Ghost), IMMUNE);
    }

    #[test]
    fn familiar_matchups() {
        assert_eq!(effectiveness(Type::Water, Type::Fire), SUPER_EFFECTIVE);
        assert_eq!(effectiveness(Type::Fire, Type::Grass), SUPER_EFFECTIVE);
        assert_eq!(effectiveness(Type::Grass, Type::Water), SUPER_EFFECTIVE);
        assert_eq!(effectiveness(Type::Fire, Type::Water), NOT_VERY_EFFECTIVE);
        assert_eq!(effectiveness(Type::Normal, Type::Normal), NEUTRAL);
    }

    #[test]
    fn dual_types_stack() {
        // Charizard is Fire/Flying: Rock hits both halves.
        let m = effectiveness_against(Type::Rock, Type::Fire, Some(Type::Flying));
        assert_eq!(m, SUPER_EFFECTIVE * SUPER_EFFECTIVE);

        // ...but Ground is neutral on Fire and cannot touch Flying at all.
        let m = effectiveness_against(Type::Ground, Type::Fire, Some(Type::Flying));
        assert_eq!(m, IMMUNE);
    }

    #[test]
    fn round_trips_dataset_names() {
        for t in ALL_TYPES.iter() {
            assert_eq!(Type::from_name(t.name()), Some(*t));
        }
        assert_eq!(Type::from_name("stellar"), None);
    }

    /// Every type is resisted by something, and every type except Normal beats
    /// something. Normal having no super effective matchup at all is a real
    /// quirk of the mainline chart, not an omission here -- it is asserted
    /// explicitly so a future edit that "fixes" it fails loudly.
    #[test]
    fn every_type_participates() {
        for atk in ALL_TYPES.iter() {
            let beats_something = ALL_TYPES
                .iter()
                .any(|d| effectiveness(*atk, *d) == SUPER_EFFECTIVE);
            let resisted_by_something = ALL_TYPES
                .iter()
                .any(|d| effectiveness(*atk, *d) < NEUTRAL);

            assert!(
                resisted_by_something,
                "{} is resisted by nothing",
                atk.name()
            );

            if *atk == Type::Normal {
                assert!(
                    !beats_something,
                    "Normal is not super effective against anything in gen 6+"
                );
            } else {
                assert!(
                    beats_something,
                    "{} is super effective against nothing",
                    atk.name()
                );
            }
        }
    }
}
