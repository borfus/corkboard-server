use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;
use chrono::NaiveDateTime;

use crate::schema::faq;
use crate::schema::faq::dsl::faq as all_faqs;

#[derive(Serialize, Queryable)]
pub struct Faq {
    pub id: Uuid,
    pub last_modified_date: NaiveDateTime,
    pub question: Option<String>,
    pub answer: Option<String>
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "faq"]
pub struct NewFaq {
    pub question: Option<String>,
    pub answer: Option<String>
}

impl Faq {
    pub fn get_all_faqs(conn: &PgConnection) -> Vec<Faq> {
        all_faqs
            .order(faq::id.desc())
            .load::<Faq>(conn)
            .expect("error!")
    }

    pub fn insert_faq(faq: NewFaq, conn: &PgConnection) -> bool {
        diesel::insert_into(faq::table)
            .values(&faq)
            .execute(conn)
            .is_ok()
    }

    pub fn get_faq_by_id(faq_id: &str, conn: &PgConnection) -> Vec<Faq> {
        all_faqs
            .filter(faq::id.eq(Uuid::parse_str(faq_id).unwrap()))
            .load::<Faq>(conn)
            .expect("error!")
    }
}

