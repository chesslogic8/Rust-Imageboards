// ===== Board Settings =====
pub const THREAD_PREVIEW_LENGTH: usize = 160;   // Number of chars shown for OP preview on board
pub const REPLY_PREVIEW_LENGTH: usize = 80;     // Number of chars to show for each reply preview on board
pub const THREADS_PER_PAGE: usize = 8;          // Threads per page on main board
pub const REPLIES_TO_SHOW: usize = 3;           // Last N replies shown for each thread on board
// ===== End Board Settings =====

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use chrono;

pub static DB: Lazy<Mutex<Connection>> = Lazy::new(|| {
    Mutex::new(Connection::open("chess.db").expect("Failed to open DB"))
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: i64,
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

// Initialize DB tables
pub fn init_db() -> Result<()> {
    let db = DB.lock().unwrap();
    db.execute(
        "CREATE TABLE IF NOT EXISTS threads (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            subject TEXT NOT NULL,
            message TEXT NOT NULL,
            media TEXT,
            bumped INTEGER NOT NULL
        )",
        [],
    )?;
    db.execute(
        "CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            thread_id INTEGER NOT NULL,
            message TEXT NOT NULL,
            media TEXT,
            FOREIGN KEY(thread_id) REFERENCES threads(id)
        )",
        [],
    )?;
    Ok(())
}

// Get all threads (latest first, by bumped)
pub fn get_threads() -> Result<Vec<Thread>> {
    let db = DB.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, subject, message, media, bumped FROM threads ORDER BY bumped DESC")?;
    let thread_iter = stmt.query_map([], |row| {
        let message: String = row.get(2)?;
        let preview = if message.len() > THREAD_PREVIEW_LENGTH {
            format!("{}...", &message[..THREAD_PREVIEW_LENGTH])
        } else {
            message.clone()
        };
        Ok(Thread {
            id: row.get(0)?,
            subject: row.get(1)?,
            message: message.clone(),
            media: row.get(3).ok(),
            preview,
            bumped: row.get(4)?,
        })
    })?;
    Ok(thread_iter.filter_map(Result::ok).collect())
}

// Get threads for a page (with pagination, by bumped)
pub fn get_threads_paged(threads_per_page: usize, page: usize) -> Result<Vec<Thread>> {
    let db = DB.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, subject, message, media, bumped FROM threads ORDER BY bumped DESC LIMIT ? OFFSET ?")?;
    let thread_iter = stmt.query_map(params![threads_per_page as i64, (threads_per_page * page) as i64], |row| {
        let message: String = row.get(2)?;
        let preview = if message.len() > THREAD_PREVIEW_LENGTH {
            format!("{}...", &message[..THREAD_PREVIEW_LENGTH])
        } else {
            message.clone()
        };
        Ok(Thread {
            id: row.get(0)?,
            subject: row.get(1)?,
            message: message.clone(),
            media: row.get(3).ok(),
            preview,
            bumped: row.get(4)?,
        })
    })?;
    Ok(thread_iter.filter_map(Result::ok).collect())
}

// Count total threads
pub fn get_total_thread_count() -> Result<usize> {
    let db = DB.lock().unwrap();
    let mut stmt = db.prepare("SELECT COUNT(*) FROM threads")?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        let count: usize = row.get(0)?;
        Ok(count)
    } else {
        Ok(0)
    }
}

// Get the last N replies for a thread (preview field for board only)
pub fn get_last_n_replies(thread_id: i64, n: usize) -> Result<Vec<Post>> {
    let db = DB.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, thread_id, message, media FROM posts WHERE thread_id = ?1 ORDER BY id DESC LIMIT ?2")?;
    let post_iter = stmt.query_map(params![thread_id, n as i64], |row| {
        let message: String = row.get(2)?;
        let preview = if message.len() > REPLY_PREVIEW_LENGTH {
            format!("{}...", &message[..REPLY_PREVIEW_LENGTH])
        } else {
            message.clone()
        };
        Ok(Post {
            id: row.get(0)?,
            thread_id: row.get(1)?,
            message: message.clone(),
            media: row.get(3).ok(),
            preview,
        })
    })?;
    let mut posts: Vec<Post> = post_iter.filter_map(Result::ok).collect();
    posts.reverse();
    Ok(posts)
}

// Get thread and its posts (full message for replies)
pub fn get_thread(id: i64) -> Result<(Thread, Vec<Post>)> {
    let db = DB.lock().unwrap();
    let mut stmt = db.prepare("SELECT id, subject, message, media, bumped FROM threads WHERE id = ?1")?;
    let mut threads = stmt.query_map(params![id], |row| {
        let message: String = row.get(2)?;
        let preview = if message.len() > THREAD_PREVIEW_LENGTH {
            format!("{}...", &message[..THREAD_PREVIEW_LENGTH])
        } else {
            message.clone()
        };
        Ok(Thread {
            id: row.get(0)?,
            subject: row.get(1)?,
            message: message.clone(),
            media: row.get(3).ok(),
            preview,
            bumped: row.get(4)?,
        })
    })?;
    let thread = threads.next().transpose()?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;

    let mut stmt = db.prepare("SELECT id, thread_id, message, media FROM posts WHERE thread_id = ?1 ORDER BY id ASC")?;
    let post_iter = stmt.query_map(params![id], |row| {
        let message: String = row.get(2)?;
        Ok(Post {
            id: row.get(0)?,
            thread_id: row.get(1)?,
            message: message.clone(),
            media: row.get(3).ok(),
            preview: message,
        })
    })?;
    let posts = post_iter.filter_map(Result::ok).collect();
    Ok((thread, posts))
}

// Add thread (set bumped to now)
pub fn insert_thread(subject: &str, message: &str, media: Option<&str>) -> Result<i64> {
    let db = DB.lock().unwrap();
    let now = chrono::Utc::now().timestamp();
    db.execute(
        "INSERT INTO threads (subject, message, media, bumped) VALUES (?1, ?2, ?3, ?4)",
        params![subject, message, media, now],
    )?;
    Ok(db.last_insert_rowid())
}

// Add post (reply) and bump thread
pub fn insert_post(thread_id: i64, message: &str, media: Option<&str>) -> Result<i64> {
    let db = DB.lock().unwrap();
    db.execute(
        "INSERT INTO posts (thread_id, message, media) VALUES (?1, ?2, ?3)",
        params![thread_id, message, media],
    )?;
    // Bump thread
    let now = chrono::Utc::now().timestamp();
    db.execute(
        "UPDATE threads SET bumped = ?1 WHERE id = ?2",
        params![now, thread_id],
    )?;
    Ok(db.last_insert_rowid())
}

// Count replies in a thread
pub fn get_post_count(thread_id: i64) -> Result<usize> {
    let db = DB.lock().unwrap();
    let mut stmt = db.prepare("SELECT COUNT(*) FROM posts WHERE thread_id = ?1")?;
    let mut rows = stmt.query(params![thread_id])?;
    if let Some(row) = rows.next()? {
        let count: usize = row.get(0)?;
        Ok(count)
    } else {
        Ok(0)
    }
}
