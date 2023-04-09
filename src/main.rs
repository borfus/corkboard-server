#![feature(plugin, decl_macro, proc_macro_hygiene)]

extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket_contrib;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

mod domain;
mod route;
mod config;
mod schema;

mod prelude {
    pub use crate::domain::*;
    pub use crate::route::*;
    pub use crate::config::*;
    pub use crate::schema::*;
    pub use crate::schema::event::*;
    pub use crate::schema::pin::*;
    pub use crate::schema::faq::*;
    pub use crate::schema::luckymon_history::*;
}

use dotenv::dotenv;
use std::env;

fn rocket() -> rocket::Rocket {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("set DATABASE_URL");

    let pool = config::db::init_pool(database_url);
    rocket::ignite()
        .manage(pool)
        .mount(
            "/api/v1/",
            routes![
                route::event::get_all,
                route::event::get_all_current,
                route::event::new_event,
                route::event::get_one,
                route::event::update,
                route::event::delete,
                route::pin::get_all,
                route::pin::new_pin,
                route::pin::get_one,
                route::pin::update,
                route::pin::delete,
                route::faq::get_all,
                route::faq::new_faq,
                route::faq::get_one,
                route::faq::update,
                route::faq::delete,
                route::luckymon_history::get_all,
                route::luckymon_history::new_hist,
                route::luckymon_history::get_one,
                route::luckymon_history::update,
                route::luckymon_history::delete
            ]
        )
}

fn main() {
    rocket().launch();
}

