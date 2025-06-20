use askama::Template;
use askama_web::WebTemplate;

/// A single thread’s data for the index view
pub struct ThreadInfo {
    pub id: i64,
    pub subject: String,
    pub message: String,
    /// Plain filename or empty string
    pub filename: String,
    pub reply_count: i64,
    /// Plain text preview or empty string
    pub preview: String,
}

/// Main board page
#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub title: String,
    pub threads: Vec<ThreadInfo>,
    pub page: usize,
    pub total_pages: usize,
    pub per_page: usize,
}

/// Full thread view
#[derive(Template, WebTemplate)]
#[template(path = "thread.html")]
pub struct ThreadTemplate {
    pub title: String,
    pub id: i64,
    pub subject: String,
    pub message: String,
    /// Plain filename or empty string
    pub filename: String,
    pub replies: Vec<String>,
}

/// “New Thread” form
#[derive(Template, WebTemplate)]
#[template(path = "post_form.html")]
pub struct PostFormTemplate {
    pub title: String,
}
