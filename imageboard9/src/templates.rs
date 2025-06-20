use askama::Template;
use crate::models::{Thread, Post};
use crate::boards::BoardDef;
use std::collections::HashMap;

#[derive(Template)]
#[template(path = "board.html")]
pub struct Board {
    pub board: BoardDef,
    pub threads: Vec<Thread>,
    pub thread_reply_counts: HashMap<i64, usize>,
    pub last_replies: HashMap<i64, Vec<Post>>,
    pub page: usize,
    pub page_count: usize,
}

#[derive(Template)]
#[template(path = "thread.html")]
pub struct ThreadView {
    pub thread: Thread,
    pub posts: Vec<Post>,
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorPage {
    pub message: String,
    pub back_url: String,
}
