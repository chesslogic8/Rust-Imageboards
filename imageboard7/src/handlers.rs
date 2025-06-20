use axum::{
    extract::{Multipart, Path},
    response::{Html, Redirect, IntoResponse, Response},
};
use askama::Template;
use uuid::Uuid;
use crate::models;
use crate::templates::{Board, ThreadView, ErrorPage};
use crate::models::{THREADS_PER_PAGE, REPLIES_TO_SHOW};

const MAX_MESSAGE_LEN: usize = 50_000;
const MAX_UPLOAD_SIZE: usize = 50 * 1024 * 1024; // 50MB

fn ext_from_mime(mime: &str) -> Option<&'static str> {
    match mime {
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/gif" => Some("gif"),
        "image/webp" => Some("webp"),
        "image/bmp" => Some("bmp"),
        "video/mp4" => Some("mp4"),
        _ => None
    }
}

pub async fn board() -> Html<String> {
    board_page(Path(0)).await
}

pub async fn board_page(Path(page): Path<usize>) -> Html<String> {
    let threads = models::get_threads_paged(THREADS_PER_PAGE, page);
    let total_threads = models::get_total_thread_count();
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
        threads,
        thread_reply_counts,
        last_replies,
        page,
        page_count,
    };
    Html(tmpl.render().unwrap())
}

pub async fn thread_view(Path(id): Path<i64>) -> Html<String> {
    if let Some((thread, posts)) = models::get_thread(id) {
        let tmpl = ThreadView { thread, posts };
        Html(tmpl.render().unwrap())
    } else {
        Html("<h2>Thread not found</h2>".to_string())
    }
}

async fn save_media(field: axum::extract::multipart::Field<'_>) -> Result<Option<String>, String> {
    let content_type = field.content_type().map(|m| m.to_string()).unwrap_or_default();
    let ext = match ext_from_mime(&content_type) {
        Some(e) => e,
        None => {
            return Err("Unsupported file type. Allowed: jpg, png, gif, webp, bmp, mp4.".to_string());
        }
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

pub async fn new_thread(mut multipart: Multipart) -> Response {
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
                            Ok(opt) => {
                                media = opt;
                            }
                            Err(reason) => {
                                return Html(
                                    ErrorPage {
                                        message: format!("Upload error: {}", reason),
                                        back_url: "/".to_string(),
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
                back_url: "/".to_string(),
            }
            .render()
            .unwrap(),
        ).into_response();
    }
    if message.len() > MAX_MESSAGE_LEN {
        return Html(
            ErrorPage {
                message: "Message is too long! (Max 50,000 bytes)".to_string(),
                back_url: "/".to_string(),
            }
            .render()
            .unwrap(),
        ).into_response();
    }
    models::insert_thread(&subject, &message, media.as_deref());
    return Redirect::to("/").into_response();
}

pub async fn reply(Path(id): Path<i64>, mut multipart: Multipart) -> Response {
    let mut message = String::new();
    let mut media: Option<String> = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        match field.name() {
            Some("message") => message = field.text().await.unwrap_or_default(),
            Some("media") => {
                if let Some(filename) = field.file_name() {
                    if !filename.is_empty() {
                        match save_media(field).await {
                            Ok(opt) => {
                                media = opt;
                            }
                            Err(reason) => {
                                return Html(
                                    ErrorPage {
                                        message: format!("Upload error: {}", reason),
                                        back_url: format!("/thread/{}", id),
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
                back_url: format!("/thread/{}", id),
            }
            .render()
            .unwrap(),
        ).into_response();
    }
    if message.len() > MAX_MESSAGE_LEN {
        return Html(
            ErrorPage {
                message: "Message is too long! (Max 50,000 bytes)".to_string(),
                back_url: format!("/thread/{}", id),
            }
            .render()
            .unwrap(),
        ).into_response();
    }
    models::insert_post(id, &message, media.as_deref());
    return Redirect::to(&format!("/thread/{}", id)).into_response();
}
