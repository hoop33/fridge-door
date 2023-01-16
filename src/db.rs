use rocket::{Build, Rocket};
use rocket::fairing::{self, AdHoc};
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use rocket_db_pools::{Connection, Database, sqlx};

#[derive(Database)]
#[database("fridge-door")]
struct Db(sqlx::SqlitePool);

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Message {
    // #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    id: i64,
    text: String,
    font_color: Option<String>,
    font_family: Option<String>,
    // created_at: chrono::NaiveDateTime,
    // expires_at: chrono::NaiveDateTime
}

#[get("/")]
async fn list(mut db: Connection<Db>) -> Result<Json<Vec<Message>>> {
    let messages = sqlx::query_as!(Message, "select * from messages").fetch_all(&mut *db)
        .await?;

    Ok(Json(messages))
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("./migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Database migrations failed: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("SQLx Stage", |rocket| async {
        rocket.attach(Db::init())
            .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
            .mount("/messages", routes![list])
    })
}
