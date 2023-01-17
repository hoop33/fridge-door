use rocket::fairing::{self, AdHoc};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::{Build, Rocket};
use rocket_db_pools::{sqlx, Connection, Database};
use sqlx::types::chrono::NaiveDateTime;

#[derive(Database)]
#[database("fridge-door")]
struct Db(sqlx::SqlitePool);

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

const DEFAULT_QUERY_LIMIT: u32 = 20;
const DEFAULT_QUERY_OFFSET: u32 = 0;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Message {
    // #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    id: i64,
    text: String,
    font_color: Option<String>,
    font_family: Option<String>,
    created_at: NaiveDateTime,
    expires_at: NaiveDateTime,
}

#[get("/?<limit>&<offset>&<include_expired>")]
async fn list(
    mut db: Connection<Db>,
    limit: Option<u32>,
    offset: Option<u32>,
    include_expired: Option<u32>,
) -> Result<Json<Vec<Message>>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT);
    let offset = offset.unwrap_or(DEFAULT_QUERY_OFFSET);
    let include_expired = include_expired.unwrap_or(0);

    let messages = sqlx::query_as!(
        Message,
        "select * from messages where id > ? and (? = 1 or date('now') < expires_at) limit ?",
        offset,
        include_expired,
        limit
    )
    .fetch_all(&mut *db)
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
        rocket
            .attach(Db::init())
            .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
            .mount("/messages", routes![list])
    })
}
