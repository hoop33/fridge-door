mod db;

#[macro_use]
extern crate rocket;

use rocket_cors::{AllowedOrigins, CorsOptions};
use std::str::FromStr;

#[get("/")]
fn cors<'a>() -> &'a str {
    "Hello CORS!"
}

#[launch]
fn rocket() -> _ {
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec!["Get", "Post", "Options", "Delete"]
                .into_iter()
                .map(|s| FromStr::from_str(s).unwrap())
                .collect(),
        )
        .allow_credentials(true)
        .to_cors()
        .unwrap();

    rocket::build()
        .mount("/", routes![cors])
        .attach(cors)
        .attach(db::stage())
}
