use actix_files::Files;
use actix_multipart::Multipart;
use actix_web::{
    error::{ErrorBadRequest, ErrorInternalServerError},
    middleware::{DefaultHeaders, Logger},
    web, App, Error, HttpResponse, HttpServer,
};
use chrono::Utc;
use config::Config;
use futures_util::StreamExt;
use infer::Infer;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, OpenFlags};
use serde::Deserialize;
use std::{fs, io::Write, path::PathBuf};
use uuid::Uuid;

/// ─────────────────────────────────────────────────────────────────────────────
/// Re‑export Askama filters so its derive macro can resolve `filters::…`
pub mod filters {
    pub use askama::filters::*;
}

/// templates
mod templates;
use templates::{IndexTemplate, PostFormTemplate, ThreadInfo, ThreadTemplate};

/// 50 MiB nginx limit mirrored at the app layer
const MAX_FILE_BYTES: usize = 50 * 1024 * 1024;

/// ─────────────────────────────────────────────────────────────────────────────
/// Configuration

#[derive(Deserialize, Clone)]
struct Settings {
    bind: String,           // "0.0.0.0:8080"
    db_path: String,        // "db.sqlite"
    uploads_dir: PathBuf,   // "uploads"
    title: String,          // board title
    reset_on_start: bool,   // dev convenience
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            bind:            "0.0.0.0:8080".into(),
            db_path:         "db.sqlite".into(),
            uploads_dir:     "uploads".into(),
            title:           "Chessboard Messageboard".into(),
            reset_on_start:  true,
        }
    }
}

fn load_settings() -> Settings {
    Config::builder()
        .add_source(config::Environment::default().separator("__"))
        .add_source(config::File::with_name("Config").required(false))
        .build()
        .ok()
        .and_then(|c| c.try_deserialize().ok())
        .unwrap_or_default()
}

/// ─────────────────────────────────────────────────────────────────────────────
/// Database helpers

type SqlitePool = Pool<SqliteConnectionManager>;

fn make_pool(path: &str) -> SqlitePool {
    let mgr = SqliteConnectionManager::file(path)
        .with_flags(
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_FULL_MUTEX,
        )
        .with_init(|c| {
            c.execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA foreign_keys = ON;",
            )
        });

    Pool::builder()
        .max_size(8)
        .build(mgr)
        .expect("create pool")
}

fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        BEGIN;
        CREATE TABLE IF NOT EXISTS posts (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            subject       TEXT NOT NULL,
            message       TEXT NOT NULL,
            filename      TEXT,
            created_at    INTEGER NOT NULL,
            last_activity INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS replies (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            post_id     INTEGER NOT NULL,
            message     TEXT NOT NULL,
            created_at  INTEGER NOT NULL,
            FOREIGN KEY(post_id) REFERENCES posts(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_posts_last ON posts(last_activity DESC);
        CREATE INDEX IF NOT EXISTS idx_replies_post ON replies(post_id);
        COMMIT;
    "#,
    )
}

/// ─────────────────────────────────────────────────────────────────────────────
/// Shared state

struct AppState {
    pool: SqlitePool,
    uploads_dir: PathBuf,
    title: String,
}

/// ─────────────────────────────────────────────────────────────────────────────
/// Pagination helpers

#[derive(Deserialize)]
struct PageQuery {
    page: Option<usize>,
    per_page: Option<usize>,
}

fn page_params(q: &PageQuery) -> (usize, usize, usize) {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.filter(|n| [10, 25, 50].contains(n)).unwrap_or(10);
    let offset = (page - 1) * per_page;
    (page, per_page, offset)
}

/// ─────────────────────────────────────────────────────────────────────────────
/// Handlers

async fn index(
    query: web::Query<PageQuery>,
    data: web::Data<AppState>,
) -> Result<IndexTemplate, Error> {
    let (page, per_page, offset) = page_params(&query);
    let conn = data.pool.get().map_err(ErrorInternalServerError)?;

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM posts", [], |r| r.get(0))
        .map_err(ErrorInternalServerError)?;
    let total_pages = ((total as usize + per_page - 1) / per_page).max(1);

    let mut stmt = conn
        .prepare(
            "SELECT id,
                    subject,
                    message,
                    filename,
                    (SELECT COUNT(*) FROM replies WHERE post_id = posts.id),
                    (SELECT message FROM replies WHERE post_id = posts.id ORDER BY created_at DESC LIMIT 1)
             FROM posts
             ORDER BY last_activity DESC
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(ErrorInternalServerError)?;

    let threads = stmt
        .query_map(params![per_page as i64, offset as i64], |row| {
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
        title: data.title.clone(),
        threads,
        page,
        total_pages,
        per_page,
    })
}

async fn thread_view(
    path: web::Path<i64>,
    data: web::Data<AppState>,
) -> Result<ThreadTemplate, Error> {
    let id = path.into_inner();
    let conn = data.pool.get().map_err(ErrorInternalServerError)?;

    let (subject, message, filename): (String, String, String) = conn
        .query_row(
            "SELECT subject, message, COALESCE(filename,'')
             FROM posts WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map_err(ErrorInternalServerError)?;

    let mut stmt = conn
        .prepare(
            "SELECT message FROM replies
             WHERE post_id = ?1
             ORDER BY created_at ASC",
        )
        .map_err(ErrorInternalServerError)?;
    let replies = stmt
        .query_map([id], |r| r.get::<_, String>(0))
        .map_err(ErrorInternalServerError)?
        .collect::<Result<_, _>>()
        .map_err(ErrorInternalServerError)?;

    Ok(ThreadTemplate {
        title: data.title.clone(),
        id,
        subject,
        message,
        filename,
        replies,
    })
}

async fn post_form(data: web::Data<AppState>) -> Result<PostFormTemplate, Error> {
    Ok(PostFormTemplate {
        title: data.title.clone(),
    })
}

async fn post_handler(
    mut payload: Multipart,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let conn = data.pool.get().map_err(ErrorInternalServerError)?;
    let mut subject = String::new();
    let mut message = String::new();
    let mut filename: Option<String> = None;

    let type_sniffer = Infer::new();

    while let Some(field) = payload.next().await {
        let mut field = field.map_err(ErrorInternalServerError)?;
        if let Some(cd) = field.content_disposition() {
            match cd.get_name() {
                Some("subject") => {
                    while let Some(chunk) = field.next().await {
                        subject
                            .push_str(std::str::from_utf8(&chunk?).map_err(ErrorInternalServerError)?);
                    }
                }
                Some("message") => {
                    while let Some(chunk) = field.next().await {
                        message
                            .push_str(std::str::from_utf8(&chunk?).map_err(ErrorInternalServerError)?);
                    }
                }
                Some("file") => {
                    if let Some(orig) = cd.get_filename() {
                        let ext = orig
                            .rsplit('.')
                            .next()
                            .unwrap_or("")
                            .to_ascii_lowercase();
                        let allowed = ["jpg", "jpeg", "png", "gif", "webp", "mp4"];
                        if allowed.contains(&ext.as_str()) {
                            fs::create_dir_all(&data.uploads_dir).unwrap();
                            let safe_name =
                                format!("{}-{}.{}", Utc::now().timestamp(), Uuid::new_v4(), ext);
                            let dest_path = data.uploads_dir.join(&safe_name);
                            let mut file =
                                fs::File::create(&dest_path).map_err(ErrorInternalServerError)?;

                            let mut size = 0usize;
                            let mut first_chunk: Option<Vec<u8>> = None;

                            while let Some(chunk) = field.next().await {
                                let bytes = chunk.map_err(ErrorInternalServerError)?;
                                size += bytes.len();
                                if size > MAX_FILE_BYTES {
                                    return Err(ErrorBadRequest("File exceeds 50 MB"));
                                }
                                if first_chunk.is_none() {
                                    first_chunk = Some(bytes.clone().into());
                                }
                                file.write_all(&bytes).map_err(ErrorInternalServerError)?;
                            }

                            if let Some(head) = first_chunk {
                                if let Some(kind) = type_sniffer.get(&head) {
                                    let ok = matches!(
                                        kind.mime_type(),
                                        "image/jpeg"
                                            | "image/png"
                                            | "image/gif"
                                            | "image/webp"
                                            | "video/mp4"
                                    );
                                    if !ok {
                                        let _ = fs::remove_file(dest_path);
                                        return Err(ErrorBadRequest("Unsupported file type"));
                                    }
                                }
                            }

                            filename = Some(safe_name);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let now = Utc::now().timestamp();
    conn.execute(
        "INSERT INTO posts (subject, message, filename, created_at, last_activity)
         VALUES (?1, ?2, ?3, ?4, ?4)",
        params![subject, message, filename, now],
    )
    .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::SeeOther()
        .append_header(("Location", "/"))
        .finish())
}

#[derive(Deserialize)]
struct ReplyForm {
    post_id: i64,
    message: String,
}

async fn reply_handler(
    form: web::Form<ReplyForm>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let conn = data.pool.get().map_err(ErrorInternalServerError)?;
    let now = Utc::now().timestamp();

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

    Ok(HttpResponse::SeeOther()
        .append_header(("Location", "/"))
        .finish())
}

/// ─────────────────────────────────────────────────────────────────────────────
/// Main

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let settings = load_settings();

    if settings.reset_on_start {
        let _ = fs::remove_file(&settings.db_path);
        let _ = fs::remove_dir_all(&settings.uploads_dir);
    }
    fs::create_dir_all(&settings.uploads_dir).unwrap();

    // create schema before pool to avoid race + “database locked”
    {
        let conn = Connection::open(&settings.db_path).expect("open db");
        init_schema(&conn).expect("schema init");
    }

    let pool = make_pool(&settings.db_path);

    let state = web::Data::new(AppState {
        pool,
        uploads_dir: settings.uploads_dir.clone(),
        title: settings.title.clone(),
    });

    println!("Server running at http://{}", settings.bind);
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Logger::default())
            .wrap(
                DefaultHeaders::new()
                    .add(("X-Content-Type-Options", "nosniff"))
                    .add(("X-Frame-Options", "DENY"))
                    .add(("Referrer-Policy", "same-origin")),
            )
            .service(
                Files::new("/uploads", &settings.uploads_dir)
                    .use_last_modified(true)
                    .prefer_utf8(true),
            )
            .route("/", web::get().to(index))
            .route("/post_form", web::get().to(post_form))
            .route("/post", web::post().to(post_handler))
            .route("/thread/{id}", web::get().to(thread_view))
            .route("/reply", web::post().to(reply_handler))
    })
    .bind(&settings.bind)?
    .run()
    .await
}
