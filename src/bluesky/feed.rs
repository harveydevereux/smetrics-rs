use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    #[serde(rename = "$type")]
    pub record_type: String,
    pub created_at: DateTime<Utc>,
    pub text: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub thumb: String,
    pub fullsize: String,
    pub alt: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Embed {
    pub images: Vec<Image>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub uri: String,
    pub record: Record,
    pub embed: Option<Embed>,
    pub bookmark_count: u64,
    pub reply_count: u64,
    pub repost_count: u64,
    pub like_count: u64,
    pub quote_count: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PostView {
    pub post: Post
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub feed: Vec<PostView>
}

pub async fn get_user_feed(user: &str) -> Response {
    let req = reqwest::get(format!("https://public.api.bsky.app/xrpc/app.bsky.feed.getAuthorFeed?actor={}&limit=100", user)).await.unwrap();
    req.json().await.unwrap()
}

impl crate::Post for Post {

    fn hashtags(&self) -> Vec<&str> {
        self.record.text.split_whitespace().filter(|x|x.starts_with('#')).collect()
    }

}