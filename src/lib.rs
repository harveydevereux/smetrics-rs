use std::{collections::HashMap, fs::File, io::Write, path::Path};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::util::read_file_utf8;

pub mod bluesky;
pub mod tumblr;
pub mod instagram;
pub mod util;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
/// Represent an engagment score at a particular observation time.
pub struct TimedEngagement {
    pub engagement: u64,
    pub time: DateTime<Utc>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
/// A post's data, hashtahs, the creation time, and engagment timeseries.
pub struct PostData {
    pub hashtags: Vec<String>,
    pub time: DateTime<Utc>,
    pub engagement: Vec<TimedEngagement>
}

pub trait Post {

    fn hashtags(&self) -> Vec<&str>;

    fn uri(&self) -> &str;

    fn creation_time(&self) -> DateTime<Utc>;

    fn engagement(&self) -> u64;

}

/// Obtain new posts and merge them with local data.
///
/// The current engagement values are only store if the post creation time
/// and our current times are within supplied bounds.
pub async fn update<T: AsyncFn(&str) -> Vec<Box<dyn crate::Post>>>(
    user: &str,
    get_user_feed: T,
    path: &Path,
    max_watch_days: u64,
    min_interval_ms: u64,
    pretty: bool
) {
    let fut_posts = get_user_feed(user);

    let now = Utc::now();
    let mut data: HashMap<String, HashMap<String, PostData>> = if path.exists() {
        match read_file_utf8(&path) {
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
        // If unseen always store.
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

            // Don't store if older than max_watch_days.
            if ((now - post.creation_time()).num_days() as u64) > max_watch_days {
                continue
            }

            if let Some(val) = data.get_mut(user) {
                if val[post.uri()].engagement.last().is_some_and
                (
                    |obs| -> bool {
                        if ((now-obs.time).num_milliseconds() as u64) < min_interval_ms { return true }
                        false
                    }
                ) {
                    continue;
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

    let mut file = File::create(path).unwrap();
    file.write_all(json.as_bytes()).unwrap();


}

