use std::{collections::HashMap, fs::File, io::Write, path::Path};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{PostData, TimedEngagement, util::read_file_utf8};

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
/// Partial data from Bluesky on a Post.
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
/// The response data from Bluesky.
struct Response {
    pub feed: Vec<PostView>
}

/// Obtain PostViews from a given users feed.
///
/// See https://docs.bsky.app/docs/api/app-bsky-feed-get-author-feed
pub async fn get_user_feed(user: &str) -> Vec<Box<dyn crate::Post>> {
    let req = reqwest::get(format!("https://public.api.bsky.app/xrpc/app.bsky.feed.getAuthorFeed?actor={}&limit=100", user)).await.unwrap();
    let response: Response = req.json().await.unwrap();

    let mut posts: Vec<Box<dyn crate::Post>> = Vec::new();

    for post in response.feed {
        posts.push(Box::new(post.post));
    }

    posts
}

impl crate::Post for Post {

    fn hashtags(&self) -> Vec<&str> {
        self.record.text.split_whitespace().filter(|x|x.starts_with('#')).collect()
    }

    fn uri(&self) -> &str {
        &self.uri
    }

    fn creation_time(&self) -> DateTime<Utc> {
        self.record.created_at
    }

    fn engagement(&self) -> u64 {
        self.bookmark_count+self.like_count+2*self.quote_count+2*self.reply_count+self.repost_count
    }

}