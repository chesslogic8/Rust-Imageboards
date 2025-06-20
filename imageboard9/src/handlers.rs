use crate::boards::{BOARDS, BoardDef};
use axum::{
    extract::{Multipart, Path},
    response::{Html, Redirect, IntoResponse, Response},
};
use askama::Template;
use uuid::Uuid;
use crate::models;
use crate::templates::{Board, ThreadView, ErrorPage};
use crate::models::{THREADS_PER_PAGE, REPLIES_TO_SHOW};
use tokio::fs;

const MAX_MESSAGE_LEN: usize = 50_000;
const MAX_UPLOAD_SIZE: usize = 50 * 1024 * 1024; // 50MB

// Serve the static landing page
pub async fn landing_page() -> Html<String> {
    let html = fs::read_to_string("static/landing.html").await.unwrap_or_else(|_| {
        "<h2>Landing page not found</h2>".to_string()
    });
    Html(html)
}

fn get_board(slug: &str) -> Option<&'static BoardDef> {
    BOARDS.iter().find(|b| b.slug == slug)
}

pub async fn board_page(Path(board_slug): Path<String>) -> Response {
    board_page_with_page(Path((board_slug, 0))).await
}

pub async fn board_page_with_page(Path((board_slug, page)): Path<(String, usize)>) -> Response {
    let board = match get_board(&board_slug) {
        Some(b) => b,
        None => {
            return Html(format!("<h2>Board '{board_slug}' not found</h2>")).into_response();
        }
    };
    let threads = models::get_threads_paged(&board.slug, THREADS_PER_PAGE, page);
    let total_threads = models::get_total_thread_count(&board.slug);
    let page_count = if total_threads == 0 {
        1
    } else {
        (total_threads + THREADS_PER_PAGE - 1) / THREADS_PER_PAGE
    };

    let mut thread_reply_counts = std::collections::HashMap::new();
    let mut last_replies = std::collections::HashMap::new();

    for thread in &threads {
        let count = models::get_post_count(thread.id);
        thread_reply_counts.insert(thread.id, count);

        let replies = models::get_last_n_replies(thread.id, REPLIES_TO_SHOW);
        last_replies.insert(thread.id, replies);
    }

    let tmpl = Board {
        board: board.clone(),
        threads,
        thread_reply_counts,
        last_replies,
        page,
        page_count,
    };
    Html(tmpl.render().unwrap()).into_response()
}

// Save an uploaded file, check type and size
async fn save_media(field: axum::extract::multipart::Field<'_>) -> Result<Option<String>, String> {
    let content_type = field.content_type().map(|m| m.to_string()).unwrap_or_default();
    let ext = match content_type.as_str() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/bmp" => "bmp",
        "video/mp4" => "mp4",
        _ => return Err("Unsupported file type. Allowed: jpg, png, gif, webp, bmp, mp4.".to_string()),
    };
    let uuid = Uuid::new_v4().to_string();
    let fname = format!("{}.{}", uuid, ext);
    let path = format!("uploads/{}", fname);

    let data = field.bytes().await.map_err(|_| "Failed to read file data.".to_string())?;
    if data.len() > MAX_UPLOAD_SIZE {
        return Err("File too large. Max allowed size: 50MB.".to_string());
    }
    tokio::fs::create_dir_all("uploads").await.ok();
    tokio::fs::write(&path, &data).await.map_err(|_| "Failed to save file.".to_string())?;
    Ok(Some(fname))
}

pub async fn new_thread(Path(board_slug): Path<String>, mut multipart: Multipart) -> Response {
    let board = match get_board(&board_slug) {
        Some(b) => b,
        None => {
            return Html(format!("<h2>Board '{board_slug}' not found</h2>")).into_response();
        }
    };

    let mut subject = String::new();
    let mut message = String::new();
    let mut media: Option<String> = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        match field.name() {
            Some("subject") => subject = field.text().await.unwrap_or_default(),
            Some("message") => message = field.text().await.unwrap_or_default(),
            Some("media") => {
                if let Some(filename) = field.file_name() {
                    if !filename.is_empty() {
                        match save_media(field).await {
                            Ok(opt) => media = opt,
                            Err(reason) => {
                                return Html(
                                    ErrorPage {
                                        message: format!("Upload error: {}", reason),
                                        back_url: format!("/{}/", board.slug),
                                    }
                                    .render()
                                    .unwrap(),
                                ).into_response();
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    if subject.trim().is_empty() || message.trim().is_empty() {
        return Html(
            ErrorPage {
                message: "Subject and message are required.".to_string(),
                back_url: format!("/{}/", board.slug),
            }
            .render()
            .unwrap(),
        ).into_response();
    }
    if message.len() > MAX_MESSAGE_LEN {
        return Html(
            ErrorPage {
                message: "Message is too long! (Max 50,000 bytes)".to_string(),
                back_url: format!("/{}/", board.slug),
            }
            .render()
            .unwrap(),
        ).into_response();
    }
    models::insert_thread(&board.slug, &subject, &message, media.as_deref());
    Redirect::to(&format!("/{}/", board.slug)).into_response()
}

pub async fn thread_view(Path((board_slug, id)): Path<(String, i64)>) -> Response {
    let board = match get_board(&board_slug) {
        Some(b) => b,
        None => {
            return Html(format!("<h2>Board '{board_slug}' not found</h2>")).into_response();
        }
    };

    if let Some((thread, posts)) = models::get_thread(id) {
        if thread.board != board.slug {
            return Html("<h2>Thread not found in this board.</h2>".to_string()).into_response();
        }
        let tmpl = ThreadView { thread, posts };
        Html(tmpl.render().unwrap()).into_response()
    } else {
        Html("<h2>Thread not found</h2>".to_string()).into_response()
    }
}

pub async fn reply(
    Path((board_slug, id)): Path<(String, i64)>,
    mut multipart: Multipart
) -> Response {
    let board = match get_board(&board_slug) {
        Some(b) => b,
        None => {
            return Html(format!("<h2>Board '{board_slug}' not found</h2>")).into_response();
        }
    };

    let mut message = String::new();
    let mut media: Option<String> = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        match field.name() {
            Some("message") => message = field.text().await.unwrap_or_default(),
            Some("media") => {
                if let Some(filename) = field.file_name() {
                    if !filename.is_empty() {
                        match save_media(field).await {
                            Ok(opt) => media = opt,
                            Err(reason) => {
                                return Html(
                                    ErrorPage {
                                        message: format!("Upload error: {}", reason),
                                        back_url: format!("/{}/thread/{}", board.slug, id),
                                    }
                                    .render()
                                    .unwrap(),
                                ).into_response();
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    if message.trim().is_empty() {
        return Html(
            ErrorPage {
                message: "Message is required.".to_string(),
                back_url: format!("/{}/thread/{}", board.slug, id),
            }
            .render()
            .unwrap(),
        ).into_response();
    }
    if message.len() > MAX_MESSAGE_LEN {
        return Html(
            ErrorPage {
                message: "Message is too long! (Max 50,000 bytes)".to_string(),
                back_url: format!("/{}/thread/{}", board.slug, id),
            }
            .render()
            .unwrap(),
        ).into_response();
    }
    models::insert_post(id, &message, media.as_deref());
    Redirect::to(&format!("/{}/thread/{}", board.slug, id)).into_response()
}
