use std::{collections::HashMap, fs::File, io::Write, path::Path};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::util::read_file_utf8;

const CACHE_PATH: &str = ".instagram_cache.json";


#[derive(Serialize, Deserialize, Debug)]
pub struct MediaId {
    id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MediaList {
    data: Vec<MediaId>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MediaResponse {
    media: MediaList
}

/// Obtain IG media ids for a user.
///
/// Note the access token is enough to identify the user 'me'.
/// See https://developers.facebook.com/docs/instagram-platform/instagram-graph-api/reference/ig-user
pub async fn get_user_media(user: &str, key: &str) -> Vec<String> {
    let req = reqwest::get(format!("https://graph.instagram.com/v25.0/{}?fields=media&access_token={}", user, key)).await.unwrap();
    req.json::<MediaResponse>().await.unwrap().media.data.
        iter().
        map(|m|m.id.clone()).
        collect()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Post {
    like_count: u64,
    comments_count: u64,
    timestamp: DateTime<Utc>,
    permalink: String,
    id: String,
    caption: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cache {
    seen: HashMap<String, DateTime<Utc>>
}

fn load_cache(path: &Path) -> Cache {
    let cache = path.join(CACHE_PATH);
    if cache.exists() {
        match read_file_utf8(&cache) {
            Some(data) => {
                match serde_yaml::from_str::<Cache>(&data) {
                    Ok(cache) => {
                        return cache
                    }
                    Err(_) => {  }
                }
            },
            None => {  }
        }
    }
    return Cache { seen: HashMap::new() }
}

/// Filter out already seen ids, and too old ids, to conserve API calls.
///
/// get_user_media will return all ids, but not with any dates.
///
/// The Graph API can batch calls but each (Media) call in the batch still counts
/// to the rate limit.
fn filter_seen(media: Vec<String>, path: &Path, max_watch_days: u64) -> Vec<String> {
    let mut filtered: Vec<String> = vec![];
    let cache = load_cache(path);
    let now = Utc::now();

    for id in media {
        if cache.seen.contains_key(&id) {
            if ((now-cache.seen[&id]).num_days() as u64) <= max_watch_days {
                filtered.push(id);
            }
        }
        else {
            filtered.push(id);
        }
    }
    filtered
}

/// Cache seen ids with there creation dates.
fn update_cache(posts: &Vec<Post>, path: &Path) {
    let mut cache = load_cache(path);
    for post in posts {
        cache.seen.insert(post.id.clone(), post.timestamp);
    }
    let json = serde_json::to_string(&cache).unwrap();

    let mut file = File::create(path.join(CACHE_PATH)).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}

/// Obtain IG media ids for a user.
///
/// Note the access token is enough to identify the user 'me'.
/// See https://developers.facebook.com/docs/instagram-platform/reference/instagram-media
pub async fn get_media(key: &str, id: &String) -> Post {
    let req = reqwest::get(format!("https://graph.instagram.com/v25.0/{}?fields=permalink,comments_count,like_count,timestamp,caption&access_token={}", id, key)).await.unwrap();
    req.json::<Post>().await.unwrap()
}

/// Obtain Posts for a given IG user.
pub fn get_user_feed(key: &str, path: &Path, max_watch_days: u64) -> impl AsyncFn(&str) -> Vec<Box<dyn crate::Post>> {
    async move |user: &str| -> Vec<Box<dyn crate::Post>> {

        let media = filter_seen(get_user_media(user, key).await, path, max_watch_days);

        let fut_posts = media.iter().map(|id| get_media(key, id));
        let posts = futures::future::join_all(fut_posts).await;

        update_cache(&posts, path);

        let mut boxed_posts: Vec<Box<dyn crate::Post>> = Vec::new();

        for post in posts {
            boxed_posts.push(Box::new(post));
        }

        boxed_posts
    }
}

impl crate::Post for Post {

    fn hashtags(&self) -> Vec<&str> {
        self.caption.split_whitespace().filter(|x|x.starts_with('#')).collect()
    }

    fn uri(&self) -> &str {
        &self.permalink
    }

    fn creation_time(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn engagement(&self) -> u64 {
        self.like_count+2*self.comments_count
    }

    fn title(&self) -> &str {
        match self.caption.split("\n").next() {
            Some(s) => s,
            None => ""
        }
    }

}