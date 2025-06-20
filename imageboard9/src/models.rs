// ===== Board Settings =====
pub const THREAD_PREVIEW_LENGTH: usize = 160;
pub const REPLY_PREVIEW_LENGTH: usize = 80;
pub const THREADS_PER_PAGE: usize = 8;
pub const REPLIES_TO_SHOW: usize = 3;
pub const RESET_DB_ON_START: bool = true;
// ===== End Board Settings =====

use mysql::*;
use mysql::prelude::*;
use serde::{Deserialize, Serialize};
use chrono;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use dotenvy::dotenv;
use std::env;

pub static DB: Lazy<Mutex<PooledConn>> = Lazy::new(|| {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env or environment");
    let pool = Pool::new(db_url.as_str()).expect("Failed to connect to MariaDB/MySQL");
    Mutex::new(pool.get_conn().expect("Failed to get DB connection"))
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: i64,
    pub board: String,
    pub subject: String,
    pub message: String,
    pub media: Option<String>,
    pub preview: String,
    pub bumped: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: i64,
    pub thread_id: i64,
    pub message: String,
    pub media: Option<String>,
    pub preview: String,
}

pub fn init_db() {
    let mut db = DB.lock().unwrap();
    if RESET_DB_ON_START {
        println!("RESET_DB_ON_START is true. Dropping all posts and threads tables.");
        db.query_drop("DROP TABLE IF EXISTS posts").unwrap();
        db.query_drop("DROP TABLE IF EXISTS threads").unwrap();
    }
    db.query_drop(
        "CREATE TABLE IF NOT EXISTS threads (
            id BIGINT PRIMARY KEY AUTO_INCREMENT,
            board VARCHAR(32) NOT NULL,
            subject TEXT NOT NULL,
            message TEXT NOT NULL,
            media TEXT,
            bumped BIGINT NOT NULL
        )"
    ).unwrap();
    db.query_drop(
        "CREATE TABLE IF NOT EXISTS posts (
            id BIGINT PRIMARY KEY AUTO_INCREMENT,
            thread_id BIGINT NOT NULL,
            message TEXT NOT NULL,
            media TEXT,
            FOREIGN KEY(thread_id) REFERENCES threads(id) ON DELETE CASCADE
        )"
    ).unwrap();
}

pub fn get_threads_paged(board: &str, threads_per_page: usize, page: usize) -> Vec<Thread> {
    let mut db = DB.lock().unwrap();
    db.exec_map(
        "SELECT id, board, subject, message, media, bumped FROM threads WHERE board = :board ORDER BY bumped DESC LIMIT :limit OFFSET :offset",
        params! {
            "board" => board,
            "limit" => threads_per_page as u64,
            "offset" => (threads_per_page * page) as u64
        },
        |(id, board, subject, message, media, bumped): (i64, String, String, String, Option<String>, i64)| {
            let preview = if message.len() > THREAD_PREVIEW_LENGTH {
                format!("{}...", &message[..THREAD_PREVIEW_LENGTH])
            } else {
                message.clone()
            };
            Thread { id, board, subject, message, media, preview, bumped }
        }
    ).unwrap_or_default()
}

pub fn get_total_thread_count(board: &str) -> usize {
    let mut db = DB.lock().unwrap();
    db.exec_first::<u64, _, _>(
        "SELECT COUNT(*) FROM threads WHERE board = :board",
        params! { "board" => board }
    ).unwrap_or(Some(0)).unwrap_or(0) as usize
}

pub fn get_last_n_replies(thread_id: i64, n: usize) -> Vec<Post> {
    let mut db = DB.lock().unwrap();
    let mut posts: Vec<Post> = db.exec_map(
        "SELECT id, thread_id, message, media FROM posts WHERE thread_id = :tid ORDER BY id DESC LIMIT :n",
        params! { "tid" => thread_id, "n" => n as u64 },
        |(id, thread_id, message, media): (i64, i64, String, Option<String>)| {
            let preview = if message.len() > REPLY_PREVIEW_LENGTH {
                format!("{}...", &message[..REPLY_PREVIEW_LENGTH])
            } else {
                message.clone()
            };
            Post { id, thread_id, message, media, preview }
        }
    ).unwrap_or_default();
    posts.reverse();
    posts
}

pub fn get_thread(id: i64) -> Option<(Thread, Vec<Post>)> {
    let mut db = DB.lock().unwrap();
    let thread = db.exec_first(
        "SELECT id, board, subject, message, media, bumped FROM threads WHERE id = :id",
        params! { "id" => id }
    ).unwrap_or(None)
     .map(|(id, board, subject, message, media, bumped): (i64, String, String, String, Option<String>, i64)| {
        let preview = if message.len() > THREAD_PREVIEW_LENGTH {
            format!("{}...", &message[..THREAD_PREVIEW_LENGTH])
        } else {
            message.clone()
        };
        Thread { id, board, subject, message, media, preview, bumped }
    })?;

    let posts = db.exec_map(
        "SELECT id, thread_id, message, media FROM posts WHERE thread_id = :id ORDER BY id ASC",
        params! { "id" => id },
        |(id, thread_id, message, media): (i64, i64, String, Option<String>)| {
            Post { id, thread_id, message: message.clone(), media, preview: message }
        }
    ).unwrap_or_default();

    Some((thread, posts))
}

pub fn insert_thread(board: &str, subject: &str, message: &str, media: Option<&str>) {
    let mut db = DB.lock().unwrap();
    let now = chrono::Utc::now().timestamp();
    db.exec_drop(
        "INSERT INTO threads (board, subject, message, media, bumped) VALUES (:board, :subject, :message, :media, :bumped)",
        params! { "board" => board, "subject" => subject, "message" => message, "media" => media, "bumped" => now },
    ).unwrap();
}

pub fn insert_post(thread_id: i64, message: &str, media: Option<&str>) {
    let mut db = DB.lock().unwrap();
    db.exec_drop(
        "INSERT INTO posts (thread_id, message, media) VALUES (:tid, :message, :media)",
        params! { "tid" => thread_id, "message" => message, "media" => media },
    ).unwrap();
    let now = chrono::Utc::now().timestamp();
    db.exec_drop(
        "UPDATE threads SET bumped = :bumped WHERE id = :id",
        params! { "bumped" => now, "id" => thread_id },
    ).unwrap();
}

pub fn get_post_count(thread_id: i64) -> usize {
    let mut db = DB.lock().unwrap();
    db.exec_first::<u64, _, _>(
        "SELECT COUNT(*) FROM posts WHERE thread_id = :tid",
        params! { "tid" => thread_id }
    ).unwrap_or(Some(0)).unwrap_or(0) as usize
}
