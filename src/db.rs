use rocket::fairing::{self, AdHoc};
use rocket::response::status::Created;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::{Build, Rocket};
use rocket_db_pools::{sqlx, Connection, Database};
use sqlx::types::chrono::NaiveDateTime;

#[derive(Database)]
#[database("fridge-door")]
struct Db(sqlx::SqlitePool);

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

const DEFAULT_LIST_COUNT: u32 = 20;
const DEFAULT_LIST_SINCE_ID: u32 = 0;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Message {
    #[serde(skip_deserializing)]
    id: i64,
    text: String,
    #[serde(skip_deserializing)]
    created_at: NaiveDateTime,
    expires_at: Option<NaiveDateTime>,
}

#[post("/", data = "<message>")]
async fn create(mut db: Connection<Db>, message: Json<Message>) -> Result<Created<Json<Message>>> {
    let result = (match message.expires_at {
        Some(_) => sqlx::query!(
            "insert into messages (text, expires_at) values (?, ?)",
            message.text,
            message.expires_at
        ),
        None => sqlx::query!("insert into messages (text) values (?)", message.text),
    })
    .execute(&mut *db)
    .await?;

    // Would be really odd if reading the message we just created failed.
    // If we got here, though, it got created, so still return 201 and the message as sent.
    Ok(
        Created::new("/").body(match read(db, result.last_insert_rowid()).await {
            Ok(created) => created.unwrap_or(message),
            Err(_) => message,
        }),
    )
}

#[get("/?<count>&<since_id>&<include_expired>")]
async fn list(
    mut db: Connection<Db>,
    count: Option<u32>,
    since_id: Option<u32>,
    include_expired: Option<u32>,
) -> Result<Json<Vec<Message>>> {
    let count = count.unwrap_or(DEFAULT_LIST_COUNT);
    let since_id = since_id.unwrap_or(DEFAULT_LIST_SINCE_ID);
    let include_expired = include_expired.unwrap_or(0);

    let messages = sqlx::query_as!(
        Message,
        "select * from messages where id > ? and (expires_at is null or ? = 1 or date('now') < expires_at) limit ?",
        since_id,
        include_expired,
        count
    )
    .fetch_all(&mut *db)
    .await?;

    Ok(Json(messages))
}

#[get("/<id>")]
async fn read(mut db: Connection<Db>, id: i64) -> Result<Option<Json<Message>>> {
    let message = sqlx::query_as!(Message, "select * from messages where id = ?", id)
        .fetch_one(&mut *db)
        .await?;

    Ok(Some(Json(message)))
}

#[get("/random")]
async fn random(mut db: Connection<Db>) -> Result<Option<Json<Message>>> {
    let message = sqlx::query_as!(
        Message,
        "select * from messages where date('now') < expires_at order by random() limit 1"
    )
    .fetch_one(&mut *db)
    .await?;

    Ok(Some(Json(message)))
}

#[delete("/<id>")]
async fn delete(mut db: Connection<Db>, id: i64) -> Result<()> {
    // Update the message to have an expiration date of now.
    sqlx::query!(
        "update messages set expires_at = date('now') where id = ?",
        id
    )
    .execute(&mut *db)
    .await?;

    Ok(())
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
            .mount("/messages", routes![create, list, read, delete, random])
    })
}
