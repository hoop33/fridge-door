#[macro_use]
extern crate rocket;

use std::str::FromStr;

use rocket::fs::FileServer;
use rocket_cors::{AllowedOrigins, CorsOptions};

mod db;

#[get("/cors")]
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

    let rocket = rocket::build();
    let static_dir: String = rocket
        .figment()
        .extract_inner("static_dir")
        .unwrap_or("./static".to_string());

    rocket
        .mount("/", routes![cors])
        .mount("/", FileServer::from(static_dir))
        .attach(cors)
        .attach(db::stage())
}
