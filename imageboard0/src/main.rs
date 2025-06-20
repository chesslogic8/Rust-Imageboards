
use actix_files;
use actix_multipart::Multipart;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use futures_util::StreamExt as _;
use serde::{Deserialize, Serialize};
use sled::{Db, IVec};
use std::{fs, io::Write, path::Path};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Post {
    title: String,
    message: String,
    filename: String,
    is_video: bool,
}

struct AppState {
    db: Db,
}

const STYLE: &str = r#"
body { font-family: Arial, sans-serif; background: #f0f0f0; padding: 20px; }
.form-container { background: #fff; padding: 20px; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); max-width: 600px; margin: auto; }
input, textarea, button { width: 100%; padding: 10px; margin-bottom: 10px; border: 1px solid #ddd; border-radius: 4px; }
button { background: #007bff; color: #fff; border: none; cursor: pointer; }
button:hover { background: #0056b3; }
.post { background: #fff; padding: 20px; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); margin: 20px auto; max-width: 600px; }
.post-title { font-size: 1.2em; margin-bottom: 10px; }
.post-message { margin-bottom: 10px; }
"#;

async fn index(data: web::Data<AppState>) -> impl Responder {
    let tree = data.db.open_tree("posts").unwrap();
    let mut html = String::new();
    html.push_str(&format!("<html><head><meta charset=\"utf-8\"><title>Imageboard</title><style>{}</style></head><body>", STYLE));
    html.push_str("<div class=\"form-container\">\n<form action=\"/post\" method=\"post\" enctype=\"multipart/form-data\">\n<input type=\"text\" name=\"title\" placeholder=\"Title\" required>\n<textarea name=\"message\" placeholder=\"Message\" rows=\"4\" required></textarea>\n<input type=\"file\" name=\"file\" accept=\"image/png,image/gif,image/webp,image/jpeg,video/mp4\" required>\n<button type=\"submit\">Post</button>\n</form>\n</div>");

    // iterate in reverse (newest first)
    for item in tree.iter().rev() {
        let (_key, value): (IVec, IVec) = item.unwrap();
        if let Ok(post) = serde_json::from_slice::<Post>(&value) {
            html.push_str("<div class=\"post\">");
            html.push_str(&format!("<div class=\"post-title\">{}</div>", post.title));
            html.push_str(&format!("<div class=\"post-message\">{}</div>", post.message));
            if post.is_video {
                html.push_str(&format!("<video controls style=\"max-width:100%;\"><source src=\"/uploads/{}\"></video>", post.filename));
            } else {
                html.push_str(&format!("<img src=\"/uploads/{}\" style=\"max-width:100%;\">", post.filename));
            }
            html.push_str("</div>");
        }
    }
    html.push_str("</body></html>");
    HttpResponse::Ok().body(html)
}

async fn post(mut payload: Multipart, data: web::Data<AppState>) -> impl Responder {
    let mut title = String::new();
    let mut message = String::new();
    let mut filename = String::new();
    let mut is_video = false;

    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();
        let content_disposition = field.content_disposition();
        let name = content_disposition.get_name().unwrap();

        if name == "title" {
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                title.push_str(std::str::from_utf8(&data).unwrap());
            }
        } else if name == "message" {
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                message.push_str(std::str::from_utf8(&data).unwrap());
            }
        } else if name == "file" {
            let original = content_disposition.get_filename().unwrap();
            let ext = Path::new(original)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            let allowed = ["png", "jpg", "jpeg", "gif", "webp", "mp4"];
            if !allowed.contains(&ext.as_str()) {
                return HttpResponse::BadRequest().body("Invalid file type");
            }
            let id = Uuid::new_v4().to_string();
            filename = format!("{}.{}", id, ext);
            is_video = ext == "mp4";
            let filepath = format!("uploads/{}", filename);
            let mut f = fs::File::create(&filepath).unwrap();

            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                f.write_all(&data).unwrap();
            }
        }
    }

    // store in sled
    let tree = data.db.open_tree("posts").unwrap();
    let id = data.db.generate_id().unwrap();
    let key = id.to_be_bytes();
    let post = Post { title, message, filename, is_video };
    let serialized = serde_json::to_vec(&post).unwrap();
    tree.insert(key, serialized).unwrap();
    tree.flush().unwrap();

    HttpResponse::SeeOther()
        .insert_header(("Location", "/"))
        .finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    fs::create_dir_all("uploads").unwrap();

    // initialize sled DB
    let db = sled::open("simple_imageboard_db").unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { db: db.clone() }))
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/post").route(web::post().to(post)))
            .service(actix_files::Files::new("/uploads", "uploads").show_files_listing())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
