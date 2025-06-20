use axum::{
    extract::{Multipart, Path},
    response::{Html, Redirect},
};
use askama::Template;
use uuid::Uuid;
use crate::models;
use crate::templates::{Board, ThreadView};

const THREADS_PER_PAGE: usize = 8;
const REPLIES_TO_SHOW: usize = 3;

pub async fn board() -> Html<String> {
    board_page(Path(0)).await
}

pub async fn board_page(Path(page): Path<usize>) -> Html<String> {
    let threads = models::get_threads_paged(THREADS_PER_PAGE, page).unwrap_or_default();
    let total_threads = models::get_total_thread_count().unwrap_or(0);
    let page_count = if total_threads == 0 {
        1
    } else {
        (total_threads + THREADS_PER_PAGE - 1) / THREADS_PER_PAGE
    };

    let mut thread_reply_counts = std::collections::HashMap::new();
    let mut last_replies = std::collections::HashMap::new();

    for thread in &threads {
        let count = models::get_post_count(thread.id).unwrap_or(0);
        thread_reply_counts.insert(thread.id, count);

        let replies = models::get_last_n_replies(thread.id, REPLIES_TO_SHOW).unwrap_or_default();
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
    if let Ok((thread, posts)) = models::get_thread(id) {
        let tmpl = ThreadView { thread, posts };
        Html(tmpl.render().unwrap())
    } else {
        Html("<h2>Thread not found</h2>".to_string())
    }
}

async fn save_media(field: axum::extract::multipart::Field<'_>) -> Option<String> {
    let content_type = field.content_type().map(|m| m.to_string()).unwrap_or_default();
    if !["image/png", "image/jpeg", "image/gif", "video/mp4"].contains(&content_type.as_str()) {
        return None;
    }
    let ext = match content_type.as_str() {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "video/mp4" => "mp4",
        _ => "",
    };
    let uuid = Uuid::new_v4().to_string();
    let fname = format!("{}.{}", uuid, ext);
    let path = format!("uploads/{}", fname);

    let data = field.bytes().await.ok()?;
    tokio::fs::write(&path, &data).await.ok()?;
    Some(fname)
}

pub async fn new_thread(mut multipart: Multipart) -> Redirect {
    let mut subject = String::new();
    let mut message = String::new();
    let mut media: Option<String> = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        match field.name() {
            Some("subject") => subject = field.text().await.unwrap_or_default(),
            Some("message") => message = field.text().await.unwrap_or_default(),
            Some("media") => {
                if let Some(filename) = save_media(field).await {
                    media = Some(filename);
                }
            }
            _ => {}
        }
    }
    // Server-side validation (even if HTML required is bypassed)
    if subject.trim().is_empty() || message.trim().is_empty() {
        return Redirect::to("/");
    }
    let _ = models::insert_thread(&subject, &message, media.as_deref());
    Redirect::to("/")
}

pub async fn reply(Path(id): Path<i64>, mut multipart: Multipart) -> Redirect {
    let mut message = String::new();
    let mut media: Option<String> = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        match field.name() {
            Some("message") => message = field.text().await.unwrap_or_default(),
            Some("media") => {
                if let Some(filename) = save_media(field).await {
                    media = Some(filename);
                }
            }
            _ => {}
        }
    }
    if message.trim().is_empty() {
        return Redirect::to(&format!("/thread/{}", id));
    }
    let _ = models::insert_post(id, &message, media.as_deref());
    Redirect::to(&format!("/thread/{}", id))
}
