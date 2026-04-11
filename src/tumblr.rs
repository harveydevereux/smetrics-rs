use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
struct Post {
    note_count: u64,
    post_url: String,
    tags: Vec<String>,
    timestamp: u64
}

#[derive(Serialize, Deserialize, Debug)]
struct Body {
    posts: Vec<Post>,
    total_posts: Value
}

#[derive(Serialize, Deserialize, Debug)]
/// HTTP code and plaintext message.
struct Meta {
    status: u16,
    msg: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    meta: Meta,
    response: Body
}

/// Obtain Posts from a given blogs feed.
pub fn get_user_feed(key: &str) -> impl AsyncFn(&str) -> Vec<Box<dyn crate::Post>> {
    async move |user: &str| -> Vec<Box<dyn crate::Post>> {
        let req = reqwest::get(format!("https://api.tumblr.com/v2/blog/{}.tumblr.com/posts?api_key={}", user, key)).await.unwrap();
        let request: Response = req.json().await.unwrap();

        let mut posts: Vec<Box<dyn crate::Post>> = Vec::new();

        for post in request.response.posts {
            posts.push(Box::new(post));
        }

        posts
    }
}

impl crate::Post for Post {

    fn hashtags(&self) -> Vec<&str> {
        self.tags.iter().map(|s|s.as_str()).collect()
    }

    fn uri(&self) -> &str {
        &self.post_url
    }

    fn creation_time(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp_secs(self.timestamp as i64).unwrap()
    }

    fn engagement(&self) -> u64 {
        self.note_count
    }

}