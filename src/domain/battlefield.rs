//! The daily battlefield: one rotating condition that nudges move damage by
//! type, shared by every battle fought that day.
//!
//! There are exactly eighteen, one per type, so coverage is guaranteed by
//! construction rather than by remembering to check. The multiplier applies to
//! the *move's* type, not the attacker's -- a Fire type swinging its Normal
//! basic gets no sun bonus, so you are rewarded for actually playing your
//! typing rather than for having rolled the right species.

use chrono::{Datelike, NaiveDate};

use super::rng::Rng;
use super::type_chart::Type;

#[derive(Serialize, Debug, Clone, Copy)]
pub struct Battlefield {
    /// Stable identifier written to `luckymon_battle.battlefield`.
    pub key: &'static str,
    pub name: &'static str,
    pub emoji: &'static str,
    pub primary: Type,
    pub secondary: Type,
    pub suppressed: Type,
    pub primary_mult: f64,
    pub secondary_mult: f64,
    pub suppressed_mult: f64,
}

impl Battlefield {
    /// Damage multiplier this battlefield applies to a move of `move_type`.
    pub fn multiplier(&self, move_type: Type) -> f64 {
        if move_type == self.primary {
            self.primary_mult
        } else if move_type == self.secondary {
            self.secondary_mult
        } else if move_type == self.suppressed {
            self.suppressed_mult
        } else {
            1.0
        }
    }

    /// One-line summary for the battle embed, e.g.
    /// "Water +30%, Electric +15%, Fire -25%".
    pub fn summary(&self) -> String {
        format!(
            "{} +{}%, {} +{}%, {} -{}%",
            self.primary.display(),
            pct_up(self.primary_mult),
            self.secondary.display(),
            pct_up(self.secondary_mult),
            self.suppressed.display(),
            pct_down(self.suppressed_mult),
        )
    }
}

fn pct_up(m: f64) -> i64 {
    ((m - 1.0) * 100.0).round() as i64
}

fn pct_down(m: f64) -> i64 {
    ((1.0 - m) * 100.0).round() as i64
}

const UP_PRIMARY: f64 = 1.30;
const UP_SECONDARY: f64 = 1.15;
const DOWN_HARD: f64 = 0.75;
const DOWN_SOFT: f64 = 0.80;

pub const BATTLEFIELDS: [Battlefield; 18] = [
    Battlefield {
        key: "scorched_plains",
        name: "Scorched Plains",
        emoji: "\u{2600}\u{FE0F}",
        primary: Type::Fire,
        secondary: Type::Ground,
        suppressed: Type::Water,
        primary_mult: UP_PRIMARY,
        secondary_mult: UP_SECONDARY,
        suppressed_mult: DOWN_HARD,
    },
    Battlefield {
        key: "downpour",
        name: "Downpour",
        emoji: "\u{1F327}\u{FE0F}",
        primary: Type::Water,
        secondary: Type::Electric,
        suppressed: Type::Fire,
        primary_mult: UP_PRIMARY,
        secondary_mult: UP_SECONDARY,
        suppressed_mult: DOWN_HARD,
    },
    Battlefield {
        key: "overgrowth",
        name: "Overgrowth",
        emoji: "\u{1F33F}",
        primary: Type::Grass,
        secondary: Type::Bug,
        suppressed: Type::Fire,
        primary_mult: UP_PRIMARY,
        secondary_mult: UP_SECONDARY,
        suppressed_mult: DOWN_SOFT,
    },
    Battlefield {
        key: "thunderstorm",
        name: "Thunderstorm",
        emoji: "\u{26A1}",
        primary: Type::Electric,
        secondary: Type::Flying,
        suppressed: Type::Ground,
        primary_mult: UP_PRIMARY,
        secondary_mult: UP_SECONDARY,
        suppressed_mult: DOWN_HARD,
    },
    Battlefield {
        key: "blizzard",
        name: "Blizzard",
        emoji: "\u{2744}\u{FE0F}",
        primary: Type::Ice,
        secondary: Type::Water,
        suppressed: Type::Grass,
        primary_mult: UP_PRIMARY,
        secondary_mult: UP_SECONDARY,
        suppressed_mult: DOWN_SOFT,
    },
    Battlefield {
        key: "sandstorm",
        name: "Sandstorm",
        emoji: "\u{1F3DC}\u{FE0F}",
        primary: Type::Ground,
        secondary: Type::Rock,
        suppressed: Type::Flying,
        primary_mult: UP_PRIMARY,
        secondary_mult: UP_SECONDARY,
        suppressed_mult: DOWN_HARD,
    },
    Battlefield {
        key: "rocky_ridge",
        name: "Rocky Ridge",
        emoji: "\u{1FAA8}",
        primary: Type::Rock,
        secondary: Type::Steel,
        suppressed: Type::Flying,
        primary_mult: UP_PRIMARY,
        secondary_mult: UP_SECONDARY,
        suppressed_mult: DOWN_SOFT,
    },
    Battlefield {
        key: "open_skies",
        name: "Open Skies",
        emoji: "\u{1F54A}\u{FE0F}",
        primary: Type::Flying,
        secondary: Type::Dragon,
        suppressed: Type::Rock,
        primary_mult: UP_PRIMARY,
        secondary_mult: UP_SECONDARY,
        suppressed_mult: DOWN_HARD,
    },
    Battlefield {
        key: "colosseum",
        name: "Colosseum",
        emoji: "\u{1F94A}",
        primary: Type::Fighting,
        secondary: Type::Normal,
        suppressed: Type::Psychic,
        primary_mult: UP_PRIMARY,
        secondary_mult: UP_SECONDARY,
        suppressed_mult: DOWN_HARD,
    },
    Battlefield {
        key: "toxic_swamp",
        name: "Toxic Swamp",
        emoji: "\u{2620}\u{FE0F}",
        primary: Type::Poison,
        secondary: Type::Bug,
        suppressed: Type::Steel,
        primary_mult: UP_PRIMARY,
        secondary_mult: UP_SECONDARY,
        suppressed_mult: DOWN_SOFT,
    },
    Battlefield {
        key: "psychic_field",
        name: "Psychic Field",
        emoji: "\u{1F52E}",
        primary: Type::Psychic,
        secondary: Type::Fairy,
        suppressed: Type::Dark,
        primary_mult: UP_PRIMARY,
        secondary_mult: UP_SECONDARY,
        suppressed_mult: DOWN_HARD,
    },
    Battlefield {
        key: "buzzing_hive",
        name: "Buzzing Hive",
        emoji: "\u{1F41D}",
        primary: Type::Bug,
        secondary: Type::Grass,
        suppressed: Type::Flying,
        primary_mult: UP_PRIMARY,
        secondary_mult: UP_SECONDARY,
        suppressed_mult: DOWN_SOFT,
    },
    Battlefield {
        key: "haunted_ruins",
        name: "Haunted Ruins",
        emoji: "\u{1F47B}",
        primary: Type::Ghost,
        secondary: Type::Dark,
        suppressed: Type::Normal,
        primary_mult: UP_PRIMARY,
        secondary_mult: UP_SECONDARY,
        suppressed_mult: DOWN_HARD,
    },
    Battlefield {
        key: "dragons_den",
        name: "Dragon's Den",
        emoji: "\u{1F409}",
        primary: Type::Dragon,
        secondary: Type::Fire,
        suppressed: Type::Fairy,
        primary_mult: UP_PRIMARY,
        secondary_mult: UP_SECONDARY,
        suppressed_mult: DOWN_SOFT,
    },
    Battlefield {
        key: "moonless_night",
        name: "Moonless Night",
        emoji: "\u{1F311}",
        primary: Type::Dark,
        secondary: Type::Ghost,
        suppressed: Type::Psychic,
        primary_mult: UP_PRIMARY,
        secondary_mult: UP_SECONDARY,
        suppressed_mult: DOWN_SOFT,
    },
    Battlefield {
        key: "iron_fortress",
        name: "Iron Fortress",
        emoji: "\u{2699}\u{FE0F}",
        primary: Type::Steel,
        secondary: Type::Rock,
        suppressed: Type::Fighting,
        primary_mult: UP_PRIMARY,
        secondary_mult: UP_SECONDARY,
        suppressed_mult: DOWN_SOFT,
    },
    Battlefield {
        key: "fairy_ring",
        name: "Fairy Ring",
        emoji: "\u{1F9DA}",
        primary: Type::Fairy,
        secondary: Type::Psychic,
        suppressed: Type::Dragon,
        primary_mult: UP_PRIMARY,
        secondary_mult: UP_SECONDARY,
        suppressed_mult: DOWN_HARD,
    },
    Battlefield {
        key: "town_square",
        name: "Town Square",
        emoji: "\u{1F3D8}\u{FE0F}",
        primary: Type::Normal,
        secondary: Type::Fairy,
        suppressed: Type::Ghost,
        primary_mult: UP_PRIMARY,
        secondary_mult: UP_SECONDARY,
        suppressed_mult: DOWN_HARD,
    },
];

/// The battlefield in effect on `date`.
///
/// Deliberately not `hash(date) % 18`: that repeats and leaves gaps, so a type
/// can go a month without its day. Instead the full set is shuffled once per
/// 18-day block, which guarantees every battlefield appears exactly once per
/// cycle while keeping the order unguessable.
pub fn for_date(date: NaiveDate) -> &'static Battlefield {
    let day = date.num_days_from_ce() as i64;
    let len = BATTLEFIELDS.len() as i64;

    // Rust's `/` and `%` truncate toward zero, which would mirror the sequence
    // either side of year 1. div_euclid/rem_euclid keep it monotonic.
    let block = day.div_euclid(len);
    let idx = day.rem_euclid(len) as usize;

    let mut order: Vec<usize> = (0..BATTLEFIELDS.len()).collect();
    Rng::new(block as u64).shuffle(&mut order);

    &BATTLEFIELDS[order[idx]]
}

/// Resolves the key stored in `luckymon_battle.battlefield`. Needed to re-render
/// a stored battle, which the tests exercise ahead of a future replay command.
#[allow(dead_code)]
pub fn by_key(key: &str) -> Option<&'static Battlefield> {
    BATTLEFIELDS.iter().find(|b| b.key == key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::type_chart::ALL_TYPES;

    #[test]
    fn every_type_is_someones_primary() {
        for t in ALL_TYPES.iter() {
            let count = BATTLEFIELDS.iter().filter(|b| b.primary == *t).count();
            assert_eq!(count, 1, "{} should be primary on exactly one field", t.name());
        }
    }

    #[test]
    fn keys_are_unique() {
        let mut keys: Vec<&str> = BATTLEFIELDS.iter().map(|b| b.key).collect();
        keys.sort();
        let before = keys.len();
        keys.dedup();
        assert_eq!(keys.len(), before);
    }

    #[test]
    fn a_field_never_buffs_and_suppresses_the_same_type() {
        for b in BATTLEFIELDS.iter() {
            assert_ne!(b.primary, b.secondary, "{}", b.key);
            assert_ne!(b.primary, b.suppressed, "{}", b.key);
            assert_ne!(b.secondary, b.suppressed, "{}", b.key);
        }
    }

    #[test]
    fn multiplier_only_touches_named_types() {
        let bf = by_key("downpour").unwrap();
        assert_eq!(bf.multiplier(Type::Water), UP_PRIMARY);
        assert_eq!(bf.multiplier(Type::Electric), UP_SECONDARY);
        assert_eq!(bf.multiplier(Type::Fire), DOWN_HARD);
        assert_eq!(bf.multiplier(Type::Normal), 1.0);
    }

    #[test]
    fn every_field_appears_once_per_block() {
        // Walk one full 18-day block and confirm it is a permutation.
        let start = NaiveDate::from_ymd_opt(2026, 8, 31).unwrap();
        let day0 = start.num_days_from_ce() as i64;
        let block_start = day0 - day0.rem_euclid(18);

        let mut seen: Vec<&str> = Vec::new();
        for offset in 0..18 {
            let d = NaiveDate::from_num_days_from_ce_opt((block_start + offset) as i32).unwrap();
            seen.push(for_date(d).key);
        }
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), 18, "an 18-day block must cover all 18 fields");
    }

    #[test]
    fn selection_is_stable_for_a_given_date() {
        let d = NaiveDate::from_ymd_opt(2026, 8, 31).unwrap();
        assert_eq!(for_date(d).key, for_date(d).key);
    }

    #[test]
    fn summary_reads_correctly() {
        let bf = by_key("downpour").unwrap();
        assert_eq!(bf.summary(), "Water +30%, Electric +15%, Fire -25%");
    }
}
