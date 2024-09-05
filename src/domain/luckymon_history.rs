use chrono::DateTime;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::Utc;
use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::luckymon_history;
use crate::schema::luckymon_history::dsl::luckymon_history as all_luckymon_history;

#[derive(Serialize, Queryable)]
pub struct LuckymonHistory {
    pub id: Uuid,
    pub last_modified_date: NaiveDateTime,
    pub user_id: Option<i64>,
    pub date_obtained: Option<NaiveDate>,
    pub pokemon_id: Option<i64>,
    pub shiny: Option<bool>,
    pub pokemon_name: Option<String>,
    pub traded: Option<bool>,
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "luckymon_history"]
pub struct NewLuckymonHistory {
    pub user_id: Option<i64>,
    pub date_obtained: Option<NaiveDate>,
    pub pokemon_id: Option<i64>,
    pub shiny: Option<bool>,
    pub pokemon_name: Option<String>,
    pub traded: Option<bool>,
}

impl LuckymonHistory {
    pub fn get_all_hist_by_user(user_id: i64, conn: &PgConnection) -> Vec<LuckymonHistory> {
        all_luckymon_history
            .filter(luckymon_history::user_id.eq(Some(user_id)))
            .order(luckymon_history::pokemon_id)
            .load::<LuckymonHistory>(conn)
            .expect("error!")
    }

    pub fn insert_hist(
        hist: NewLuckymonHistory,
        conn: &PgConnection,
        trade: bool,
    ) -> LuckymonHistory {
        // Check to see if the user already ran the .luckymon command today.
        // If so, don't add another entry.
        if !trade {
            if let Ok(existing_hist) = all_luckymon_history
                .filter(luckymon_history::user_id.eq(hist.user_id))
                .filter(luckymon_history::pokemon_id.eq(hist.pokemon_id))
                .get_result::<LuckymonHistory>(conn)
            {
                if hist.shiny.unwrap() && !existing_hist.shiny.unwrap() {
                    return Self::update_by_id(&existing_hist.id.to_string(), hist, conn);
                }
                return existing_hist;
            }
        }

        let new_hist = diesel::insert_into(luckymon_history::table)
            .values(&hist)
            .get_result::<LuckymonHistory>(conn)
            .unwrap();

        new_hist
    }

    pub fn get_hist_by_id(hist_id: &str, conn: &PgConnection) -> LuckymonHistory {
        all_luckymon_history
            .filter(luckymon_history::id.eq(Uuid::parse_str(hist_id).unwrap()))
            .get_result::<LuckymonHistory>(conn)
            .expect(format!("Unable to find luckymon_history with ID {}!", hist_id).as_str())
    }

    pub fn update_by_id(
        hist_id: &str,
        new_hist: NewLuckymonHistory,
        conn: &PgConnection,
    ) -> LuckymonHistory {
        let updated_hist = diesel::update(
            all_luckymon_history.filter(
                luckymon_history::id
                    .eq(Uuid::parse_str(hist_id).expect("Invalid luckymon_history ID!")),
            ),
        )
        .set((
            luckymon_history::last_modified_date.eq(DateTime::from_timestamp_millis(
                Utc::now().timestamp_millis(),
            )
            .unwrap()
            .naive_utc()),
            luckymon_history::user_id.eq(new_hist.user_id),
            luckymon_history::date_obtained.eq(new_hist.date_obtained),
            luckymon_history::pokemon_id.eq(new_hist.pokemon_id),
            luckymon_history::shiny.eq(new_hist.shiny),
            luckymon_history::pokemon_name.eq(new_hist.pokemon_name),
            luckymon_history::traded.eq(new_hist.traded),
        ))
        .get_result::<LuckymonHistory>(conn)
        .expect(format!("Unabled to update luckymon_history with ID {}", hist_id).as_str());

        updated_hist
    }

    pub fn update_as_traded(hist_id: &str, conn: &PgConnection) -> LuckymonHistory {
        let to_update = all_luckymon_history
            .filter(luckymon_history::id.eq(Uuid::parse_str(hist_id).unwrap()))
            .get_result::<LuckymonHistory>(conn)
            .expect(format!("Unable to find luckymon_history with ID {}!", hist_id).as_str());

        let updated_hist = diesel::update(
            all_luckymon_history.filter(
                luckymon_history::id
                    .eq(Uuid::parse_str(hist_id).expect("Invalid luckymon_history ID!")),
            ),
        )
        .set((
            luckymon_history::last_modified_date.eq(DateTime::from_timestamp_millis(
                Utc::now().timestamp_millis(),
            )
            .unwrap()
            .naive_utc()),
            luckymon_history::user_id.eq(to_update.user_id),
            luckymon_history::date_obtained.eq(to_update.date_obtained),
            luckymon_history::pokemon_id.eq(to_update.pokemon_id),
            luckymon_history::shiny.eq(to_update.shiny),
            luckymon_history::pokemon_name.eq(to_update.pokemon_name),
            luckymon_history::traded.eq(true),
        ))
        .get_result::<LuckymonHistory>(conn)
        .expect(format!("Unabled to update luckymon_history with ID {}", hist_id).as_str());

        updated_hist
    }

    pub fn delete_by_id(hist_id: &str, conn: &PgConnection) -> LuckymonHistory {
        let deleted_hist = diesel::delete(all_luckymon_history.filter(
            luckymon_history::id.eq(Uuid::parse_str(hist_id).expect("Invalid LuckymonHistory ID!")),
        ))
        .get_result(conn)
        .expect(format!("Unabled to delete LuckymonHistory with ID {}", hist_id).as_str());

        deleted_hist
    }
}
