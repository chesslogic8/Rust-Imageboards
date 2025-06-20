#[allow(unused_imports)]

use askama::Template;
use askama_web::WebTemplate;

pub struct ThreadInfo {
    pub id: i64,
    pub subject: String,
    pub message: String,
    pub filename: String,
    pub reply_count: i64,
    pub preview: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "board.html")]
pub struct BoardTemplate {
    pub board: String,
    pub threads: Vec<ThreadInfo>,
    pub page: usize,
    pub total_pages: usize,
    pub per_page: usize,
}

#[derive(Template, WebTemplate)]
#[template(path = "thread.html")]
pub struct ThreadTemplate {
    pub board: String,
    pub id: i64,
    pub subject: String,
    pub message: String,
    pub filename: String,
    pub replies: Vec<String>,
}

#[derive(Template, WebTemplate)]
#[template(path = "post_form.html")]
pub struct PostFormTemplate {
    pub board: String,
}
