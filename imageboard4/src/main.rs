use actix_files::Files;
use actix_web::{
    error::{ErrorBadRequest, ErrorInternalServerError},
    middleware::{DefaultHeaders, Logger},
    web, App, Error, HttpResponse, HttpServer, Responder,
};
use chrono::Utc;
use futures_util::StreamExt;
use infer::Infer;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, OpenFlags};
use serde::Deserialize;
use std::{fs, io::Write, path::{Path, PathBuf}};
use uuid::Uuid;

mod templates;
use templates::{BoardTemplate, PostFormTemplate};

const MAX_FILE_BYTES: usize = 50 * 1024 * 1024;
const PREVIEW_CHARS: usize = 250; // Change this for longer/shorter previews

type SqlitePool = Pool<SqliteConnectionManager>;

struct BoardPaths {
    dir: PathBuf,
    db: PathBuf,
    uploads: PathBuf,
}

fn get_board_paths(board: &str) -> BoardPaths {
    let safe_board = board.replace('.', "").replace('/', ""); // crude, safe
    let dir = PathBuf::from(format!("chess/{}", safe_board));
    let db = dir.join("db.sqlite");
    let uploads = dir.join("uploads");
    BoardPaths { dir, db, uploads }
}

fn ensure_board_init(paths: &BoardPaths) -> Result<(), std::io::Error> {
    if !paths.dir.exists() {
        fs::create_dir_all(&paths.dir)?;
    }
    if !paths.uploads.exists() {
        fs::create_dir_all(&paths.uploads)?;
    }
    if !paths.db.exists() {
        let conn = Connection::open(&paths.db).expect("create db");
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;
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
            "#,
        ).expect("init schema");
    }
    Ok(())
}

fn make_pool(db_path: &Path) -> SqlitePool {
    let mgr = SqliteConnectionManager::file(db_path)
        .with_flags(
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_FULL_MUTEX,
        )
        .with_init(|c| {
            c.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")
        });
    Pool::builder()
        .max_size(4)
        .build(mgr)
        .expect("create pool")
}

async fn index_page() -> actix_files::NamedFile {
    actix_files::NamedFile::open("static/index.html").expect("static/index.html not found")
}

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

async fn board_view(
    path: web::Path<String>,
    query: web::Query<PageQuery>,
) -> Result<impl Responder, Error> {
    let board = path.into_inner();
    let paths = get_board_paths(&board);
    ensure_board_init(&paths)
        .map_err(|_| ErrorInternalServerError("Could not initialize board directory/files"))?;
    let pool = make_pool(&paths.db);

    let (page, per_page, offset) = page_params(&query);
    let conn = pool.get().map_err(ErrorInternalServerError)?;

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
                    (SELECT COUNT(*) FROM replies WHERE post_id = posts.id)
             FROM posts
             ORDER BY last_activity DESC
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(ErrorInternalServerError)?;

    let threads = stmt
        .query_map(params![per_page as i64, offset as i64], |row| {
            let full_message: String = row.get(2)?;
            let preview = if full_message.chars().count() > PREVIEW_CHARS {
                let truncated: String = full_message.chars().take(PREVIEW_CHARS).collect();
                format!("{}…", truncated)
            } else {
                full_message.clone()
            };
            Ok(templates::ThreadInfo {
                id: row.get(0)?,
                subject: row.get(1)?,
                preview,
                filename: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                reply_count: row.get(4)?,
            })
        })
        .map_err(ErrorInternalServerError)?
        .collect::<Result<_, _>>()
        .map_err(ErrorInternalServerError)?;

    Ok(BoardTemplate {
        board: board.clone(),
        threads,
        page,
        total_pages,
        per_page,
    })
}

async fn post_form(path: web::Path<String>) -> Result<PostFormTemplate, Error> {
    Ok(PostFormTemplate {
        board: path.into_inner(),
    })
}

async fn post_handler(
    path: web::Path<String>,
    mut payload: actix_multipart::Multipart,
) -> Result<HttpResponse, Error> {
    let board = path.into_inner();
    let paths = get_board_paths(&board);
    ensure_board_init(&paths)
        .map_err(|_| ErrorInternalServerError("Could not initialize board directory/files"))?;
    let pool = make_pool(&paths.db);

    let conn = pool.get().map_err(ErrorInternalServerError)?;
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
                            fs::create_dir_all(&paths.uploads).unwrap();
                            let safe_name =
                                format!("{}-{}.{}", Utc::now().timestamp(), Uuid::new_v4(), ext);
                            let dest_path = paths.uploads.join(&safe_name);
                            let mut file =
                                fs::File::create(&dest_path).map_err(ErrorInternalServerError)?;

                            let mut size = 0usize;
                            let mut first_chunk: Option<Vec<u8>> = None;

                            while let Some(chunk) = field.next().await {
                                let bytes = chunk.map_err(ErrorInternalServerError)?;
                                size += bytes.len();
                                if size > MAX_FILE_BYTES {
                                    return Err(ErrorBadRequest("File exceeds 50 MB"));
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
        .append_header(("Location", format!("/chess/{}/", board)))
        .finish())
}

#[derive(Deserialize)]
struct ReplyForm {
    post_id: i64,
    message: String,
}

async fn reply_handler(
    path: web::Path<String>,
    form: web::Form<ReplyForm>,
) -> Result<HttpResponse, Error> {
    let board = path.into_inner();
    let paths = get_board_paths(&board);
    ensure_board_init(&paths)
        .map_err(|_| ErrorInternalServerError("Could not initialize board directory/files"))?;
    let pool = make_pool(&paths.db);

    let conn = pool.get().map_err(ErrorInternalServerError)?;
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
        .append_header(("Location", format!("/chess/{}/", board)))
        .finish())
}

async fn thread_view(
    path: web::Path<(String, i64)>,
) -> Result<impl Responder, Error> {
    let (board, id) = path.into_inner();
    let paths = get_board_paths(&board);
    ensure_board_init(&paths)
        .map_err(|_| ErrorInternalServerError("Could not initialize board directory/files"))?;
    let pool = make_pool(&paths.db);

    let conn = pool.get().map_err(ErrorInternalServerError)?;

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

    Ok(templates::ThreadTemplate {
        board,
        id,
        subject,
        message,
        filename,
        replies,
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    if !Path::new("static/index.html").exists() {
        panic!("Please upload your index.html to static/index.html!");
    }

    println!("Server running at http://127.0.0.1:8080");
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(
                DefaultHeaders::new()
                    .add(("X-Content-Type-Options", "nosniff"))
                    .add(("X-Frame-Options", "DENY"))
                    .add(("Referrer-Policy", "same-origin")),
            )
            .route("/", web::get().to(index_page))
            .route("/chess/{board}/", web::get().to(board_view))
            .route("/chess/{board}/post_form", web::get().to(post_form))
            .route("/chess/{board}/post", web::post().to(post_handler))
            .route("/chess/{board}/thread/{id}", web::get().to(thread_view))
            .route("/chess/{board}/reply", web::post().to(reply_handler))
            .service(Files::new("/static", "static").prefer_utf8(true))
            .service(Files::new("/chess", "chess").prefer_utf8(true))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
