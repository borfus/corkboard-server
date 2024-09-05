#![feature(decl_macro, proc_macro_hygiene)]

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

mod config;
mod domain;
mod route;
mod schema;

mod prelude {
    pub use crate::domain::*;
}

use dotenv::dotenv;
use rocket::http::Method;
use rocket_cors::{AllowedOrigins, Cors};
use std::{collections::HashSet, env};

fn rocket() -> rocket::Rocket {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("set DATABASE_URL");
    let pool = config::db::init_pool(database_url);

    let allowed_origins = AllowedOrigins::all();

    let mut expose_headers = HashSet::new();
    expose_headers.insert(String::from("Content-Type"));

    let cors_options = Cors {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post, Method::Put, Method::Delete]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: rocket_cors::AllowedHeaders::some(&[
            "Authorization",
            "Accept",
            "Content-Type",
        ]),
        allow_credentials: true,
        expose_headers,
        max_age: Some(3600),
        ..Default::default()
    };

    rocket::ignite().attach(cors_options).manage(pool).mount(
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
            route::luckymon_history::update_traded,
            route::luckymon_history::delete
        ],
    )
}

fn main() {
    rocket().launch();
}
