use actix_files::Files;
use actix_multipart::Multipart;
use actix_web::{web, App, HttpServer, Error};
use actix_web::error::ErrorInternalServerError;
use chrono::Utc;
use futures_util::StreamExt;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::sync::Mutex;
use uuid::Uuid;

mod templates;
use templates::{IndexTemplate, ThreadTemplate, PostFormTemplate, ThreadInfo};

/// Whether to delete the DB and uploads directory on each start (for dev)
const RESET_ON_START: bool = true;
/// The title that appears in the header
const APP_TITLE: &str = "Chessboard Messageboard";

/// Shared application state
struct AppState {
    db: Mutex<Connection>,
}

/// Form data for replies
#[derive(Deserialize)]
struct ReplyForm {
    post_id: i64,
    message: String,
}

/// Query parameters for pagination
#[derive(Deserialize)]
struct PageQuery {
    page: Option<usize>,
    per_page: Option<usize>,
}

/// Initialize or reset the SQLite schema
fn init_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(r#"
        PRAGMA foreign_keys = OFF;
        DROP TABLE IF EXISTS replies;
        DROP TABLE IF EXISTS posts;
        PRAGMA foreign_keys = ON;
        BEGIN;
        CREATE TABLE posts (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            subject       TEXT NOT NULL,
            message       TEXT NOT NULL,
            filename      TEXT,
            created_at    TEXT NOT NULL,
            last_activity TEXT NOT NULL
        );
        CREATE TABLE replies (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            post_id     INTEGER NOT NULL,
            message     TEXT NOT NULL,
            created_at  TEXT NOT NULL,
            FOREIGN KEY(post_id) REFERENCES posts(id) ON DELETE CASCADE
        );
        COMMIT;
    "#)?;
    Ok(())
}

/// GET `/` → paginated index page
async fn index(
    query: web::Query<PageQuery>,
    data: web::Data<AppState>,
) -> Result<IndexTemplate, Error> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.filter(|n| [10, 25, 50].contains(n)).unwrap_or(10);
    let offset = (page - 1) * per_page;

    let conn = data.db.lock().unwrap();

    // Total number of posts
    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM posts", [], |r| r.get(0))
        .map_err(ErrorInternalServerError)?;
    let total_pages = ((total as usize + per_page - 1) / per_page).max(1);

    // Fetch the requested page of threads
    let mut stmt = conn
        .prepare(&format!(
            "SELECT
                id,
                subject,
                message,
                filename,
                (SELECT COUNT(*) FROM replies WHERE post_id = posts.id),
                (SELECT message FROM replies WHERE post_id = posts.id ORDER BY created_at DESC LIMIT 1)
             FROM posts
             ORDER BY last_activity DESC
             LIMIT {limit} OFFSET {off}",
            limit = per_page,
            off = offset,
        ))
        .map_err(ErrorInternalServerError)?;

    let threads = stmt
        .query_map([], |row| {
            Ok(ThreadInfo {
                id: row.get(0)?,
                subject: row.get(1)?,
                message: row.get(2)?,
                filename: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                reply_count: row.get(4)?,
                preview: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
            })
        })
        .map_err(ErrorInternalServerError)?
        .collect::<Result<_, _>>()
        .map_err(ErrorInternalServerError)?;

    Ok(IndexTemplate {
        title: APP_TITLE.to_string(),
        threads,
        page,
        total_pages,
        per_page,
    })
}

/// GET `/thread/{id}` → full thread view
async fn thread_view(
    path: web::Path<i64>,
    data: web::Data<AppState>,
) -> Result<ThreadTemplate, Error> {
    let id = path.into_inner();
    let conn = data.db.lock().unwrap();

    // Fetch original post
    let mut stmt = conn
        .prepare("SELECT subject, message, filename FROM posts WHERE id = ?1")
        .map_err(ErrorInternalServerError)?;
    let (subject, message, raw_fname): (String, String, Option<String>) =
        stmt.query_row([id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .map_err(ErrorInternalServerError)?;
    let filename = raw_fname.unwrap_or_default();

    // Fetch all replies
    let mut stmt2 = conn
        .prepare("SELECT message FROM replies WHERE post_id = ?1 ORDER BY created_at ASC")
        .map_err(ErrorInternalServerError)?;
    let replies = stmt2
        .query_map([id], |r| Ok(r.get::<_, String>(0)?))
        .map_err(ErrorInternalServerError)?
        .collect::<Result<_, _>>()
        .map_err(ErrorInternalServerError)?;

    Ok(ThreadTemplate {
        title: APP_TITLE.to_string(),
        id,
        subject,
        message,
        filename,
        replies,
    })
}

/// GET `/post_form` → new thread form
async fn post_form() -> Result<PostFormTemplate, Error> {
    Ok(PostFormTemplate {
        title: APP_TITLE.to_string(),
    })
}

/// POST `/post` → create a new thread
async fn post_handler(
    mut payload: Multipart,
    data: web::Data<AppState>,
) -> Result<actix_web::HttpResponse, Error> {
    let conn = data.db.lock().unwrap();
    let mut subject = String::new();
    let mut message = String::new();
    let mut filename: Option<String> = None;

    // Parse multipart fields
    while let Some(field_res) = payload.next().await {
        let mut field = field_res.map_err(ErrorInternalServerError)?;
        if let Some(cd) = field.content_disposition() {
            if let Some(name) = cd.get_name() {
                match name {
                    "subject" => {
                        while let Some(chunk) = field.next().await {
                            let data = chunk.map_err(ErrorInternalServerError)?;
                            subject.push_str(std::str::from_utf8(&data).map_err(ErrorInternalServerError)?);
                        }
                    }
                    "message" => {
                        while let Some(chunk) = field.next().await {
                            let data = chunk.map_err(ErrorInternalServerError)?;
                            message.push_str(std::str::from_utf8(&data).map_err(ErrorInternalServerError)?);
                        }
                    }
                    "file" => {
                        if let Some(orig) = cd.get_filename() {
                            let ext = orig.rsplit('.').next().unwrap_or("");
                            if ["jpg","jpeg","png","gif","webp","mp4"].contains(&ext) {
                                fs::create_dir_all("uploads").unwrap();
                                let safe_name = format!("{}-{}.{}", Utc::now().timestamp(), Uuid::new_v4(), ext);
                                let mut f = fs::File::create(format!("uploads/{}", safe_name))
                                    .map_err(ErrorInternalServerError)?;
                                while let Some(chunk) = field.next().await {
                                    let data = chunk.map_err(ErrorInternalServerError)?;
                                    f.write_all(&data).map_err(ErrorInternalServerError)?;
                                }
                                filename = Some(safe_name);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Insert into DB
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO posts (subject, message, filename, created_at, last_activity)
         VALUES (?1, ?2, ?3, ?4, ?4)",
        params![subject, message, filename, now],
    )
    .map_err(ErrorInternalServerError)?;

    Ok(actix_web::HttpResponse::SeeOther()
        .append_header(("Location", "/"))
        .finish())
}

/// POST `/reply` → add a reply and bump the thread
async fn reply_handler(
    form: web::Form<ReplyForm>,
    data: web::Data<AppState>,
) -> Result<actix_web::HttpResponse, Error> {
    let conn = data.db.lock().unwrap();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO replies (post_id, message, created_at) VALUES (?1, ?2, ?3)",
        params![form.post_id, form.message.clone(), now],
    )
    .map_err(ErrorInternalServerError)?;
    conn.execute(
        "UPDATE posts SET last_activity = ?1 WHERE id = ?2",
        params![now, form.post_id],
    )
    .map_err(ErrorInternalServerError)?;

    Ok(actix_web::HttpResponse::SeeOther()
        .append_header(("Location", "/"))
        .finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if RESET_ON_START {
        let _ = fs::remove_file("db.sqlite");
        let _ = fs::remove_dir_all("uploads");
    }
    fs::create_dir_all("uploads").unwrap();

    let conn = Connection::open("db.sqlite").expect("Failed to open DB");
    init_db(&conn).expect("Failed to init schema");

    let state = web::Data::new(AppState { db: Mutex::new(conn) });

    println!("Server running at http://0.0.0.0:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(Files::new("/uploads", "uploads").use_last_modified(true))
            .route("/", web::get().to(index))
            .route("/post_form", web::get().to(post_form))
            .route("/post", web::post().to(post_handler))
            .route("/thread/{id}", web::get().to(thread_view))
            .route("/reply", web::post().to(reply_handler))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

