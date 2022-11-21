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
                route::event::new_event,
                route::event::get_one,
                route::pin::get_all,
                route::pin::new_pin,
                route::pin::get_one,
                route::faq::get_all,
                route::faq::new_faq,
                route::faq::get_one
            ]
        )
}

fn main() {
    rocket().launch();
}

