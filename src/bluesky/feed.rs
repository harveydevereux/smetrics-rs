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

async fn get_user_feed(user: &str) -> Vec<Box<dyn crate::Post>> {
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

    fn time(&self) -> DateTime<Utc> {
        self.record.created_at
    }

    fn engagement(&self) -> u64 {
        self.bookmark_count+self.like_count+2*self.quote_count+2*self.reply_count+self.repost_count
    }

}

pub async fn scrape(user: &str, path: &Path, max_watch_days: u64, min_interval_ms: u64, pretty: bool) {
    let fut_posts = get_user_feed(user);

    let now = Utc::now();
    let file = path.join("bluesky.json");
    let mut data: HashMap<String, HashMap<String, PostData>> = if file.exists() {
        match read_file_utf8(&file) {
            Some(data) => {
                match serde_yaml::from_str(&data) {
                    Ok(data) => {
                        data
                    }
                    Err(_) => HashMap::new()
                }
            },
            None => HashMap::new()
        }
    }
    else { HashMap::new() };

    if !data.contains_key(user) {
        data.insert(user.to_string(), HashMap::new());
    }

    for post in fut_posts.await {
        if !data[user].contains_key(post.uri()) {
            if let Some(val) = data.get_mut(user) {
                val.insert (
                    post.uri().to_string(),
                    PostData {
                        hashtags: post.hashtags().iter().map(|s|s.to_string()).collect(),
                        time: now,
                        engagement: vec![TimedEngagement { engagement: post.engagement(), time: now }]
                    }
                );
            };
        }
        else {

            if ((now - post.time()).num_days() as u64) > max_watch_days {
                continue
            }

            if let Some(val) = data.get_mut(user) {
                match val.get_mut(post.uri()).unwrap().engagement.last() {
                    Some(data) => {
                        if ((now-data.time).num_milliseconds() as u64) < min_interval_ms {
                            continue
                        }
                    },
                    None => {}
                }
                val.get_mut(post.uri()).unwrap().engagement.push(
                    TimedEngagement { engagement: post.engagement(), time: now }
                );
                val.get_mut(post.uri()).unwrap().engagement.sort_by(|a,b| a.time.cmp(&b.time));
            }
        }
    }


    let json = match pretty {
        true => serde_json::to_string_pretty(&data).unwrap(),
        false => serde_json::to_string(&data).unwrap()
    };

    let mut file = File::create(file).unwrap();
    file.write_all(json.as_bytes()).unwrap();


}