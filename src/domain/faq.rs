use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;
use chrono::NaiveDateTime;
use chrono::Utc;

use crate::schema::faq;
use crate::schema::faq::dsl::faq as all_faqs;

#[derive(Serialize, Queryable)]
pub struct Faq {
    pub id: Uuid,
    pub last_modified_date: NaiveDateTime,
    pub question: Option<String>,
    pub answer: Option<String>,
    pub guild_id: Option<i64>
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "faq"]
pub struct NewFaq {
    pub guild_id: Option<i64>,
    pub question: Option<String>,
    pub answer: Option<String>
}

impl Faq {
    pub fn get_all_faqs(guild_id: i64, conn: &PgConnection) -> Vec<Faq> {
        all_faqs
            .filter(faq::guild_id.eq(Some(guild_id)))
            .order(faq::id.desc())
            .load::<Faq>(conn)
            .expect("error!")
    }

    pub fn insert_faq(faq: NewFaq, conn: &PgConnection) -> Faq {
        let new_faq = diesel::insert_into(faq::table)
            .values(&faq)
            .get_result::<Faq>(conn)
            .unwrap();
        new_faq
    }

    pub fn get_faq_by_id(faq_id: &str, guild_id: i64, conn: &PgConnection) -> Vec<Faq> {
        all_faqs
            .filter(faq::id.eq(Uuid::parse_str(faq_id).unwrap()))
            .filter(faq::guild_id.eq(Some(guild_id)))
            .load::<Faq>(conn)
            .expect("error!")
    }

    pub fn update_by_id(faq_id: &str, new_faq: NewFaq, conn: &PgConnection) -> Faq {
        let updated_faq = diesel::update(all_faqs.filter(faq::id.eq(Uuid::parse_str(faq_id).expect("Invalid FAQ ID!"))))
            .set((
                    faq::last_modified_date.eq(NaiveDateTime::from_timestamp_millis(Utc::now().timestamp_millis()).unwrap()),
                    faq::question.eq(new_faq.question),
                    faq::answer.eq(new_faq.answer)
            ))
            .get_result::<Faq>(conn)
            .expect(format!("Unabled to update faq with ID {}", faq_id).as_str());

        updated_faq
    }

    pub fn delete_by_id(faq_id: &str, conn: &PgConnection) -> Faq {
        let deleted_faq = diesel::delete(all_faqs.filter(faq::id.eq(Uuid::parse_str(faq_id).expect("Invalid FAQ ID!"))))
            .get_result(conn)
            .expect(format!("Unabled to delete FAQ with ID {}", faq_id).as_str());

        deleted_faq
    }
}

