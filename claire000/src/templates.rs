use askama::Template;
use crate::models::{Thread, Post};
use std::collections::HashMap;

#[derive(Template)]
#[template(path = "board.html")]
pub struct Board {
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
