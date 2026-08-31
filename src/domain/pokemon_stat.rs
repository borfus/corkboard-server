//! Read-only access to the seeded `pokemon_stat` table.
//!
//! The table is populated by migration and never written at runtime, so there
//! is no `New...` insertable counterpart here.

use diesel::pg::PgConnection;
use diesel::prelude::*;

use crate::schema::pokemon_stat;
use crate::schema::pokemon_stat::dsl::pokemon_stat as all_pokemon_stat;

use super::type_chart::Type;

#[derive(Serialize, Deserialize, Queryable, Debug, Clone)]
pub struct PokemonStat {
    pub pokemon_id: i64,
    pub name: String,
    pub hp: i32,
    pub atk: i32,
    pub def: i32,
    pub spa: i32,
    pub spd: i32,
    pub spe: i32,
    pub type_1: String,
    pub type_2: Option<String>,
    pub bst: i32,
}

impl PokemonStat {
    /// Primary typing. Falls back to Normal if the row somehow holds a type
    /// the chart does not know, which keeps a bad row from panicking a battle.
    pub fn primary_type(&self) -> Type {
        Type::from_name(&self.type_1).unwrap_or(Type::Normal)
    }

    pub fn secondary_type(&self) -> Option<Type> {
        self.type_2.as_ref().and_then(|t| Type::from_name(t))
    }

    /// Bulk lookup. Order is not guaranteed to match `ids`; callers that care
    /// should index the result themselves.
    pub fn get_by_ids(ids: &[i64], conn: &PgConnection) -> Vec<PokemonStat> {
        if ids.is_empty() {
            return Vec::new();
        }
        all_pokemon_stat
            .filter(pokemon_stat::pokemon_id.eq_any(ids))
            .load::<PokemonStat>(conn)
            .unwrap_or_default()
    }

    /// Everything with `t` in either slot. Backs NPC trainer teams, which are
    /// themed by type.
    pub fn get_by_type(t: Type, conn: &PgConnection) -> Vec<PokemonStat> {
        let name = t.name();
        all_pokemon_stat
            .filter(
                pokemon_stat::type_1
                    .eq(name)
                    .or(pokemon_stat::type_2.eq(name)),
            )
            .order(pokemon_stat::pokemon_id)
            .load::<PokemonStat>(conn)
            .unwrap_or_default()
    }

    pub fn get_all(conn: &PgConnection) -> Vec<PokemonStat> {
        all_pokemon_stat
            .order(pokemon_stat::pokemon_id)
            .load::<PokemonStat>(conn)
            .unwrap_or_default()
    }
}
