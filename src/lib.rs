use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod bluesky;
pub mod util;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimedEngagement {
    pub engagement: u64,
    pub time: DateTime<Utc>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PostData {
    pub hashtags: Vec<String>,
    pub time: DateTime<Utc>,
    pub engagement: Vec<TimedEngagement>
}

pub trait Post {

    fn hashtags(&self) -> Vec<&str>;

    fn uri(&self) -> &str;

    fn time(&self) -> DateTime<Utc>;

    fn engagement(&self) -> u64;

}

pub trait Feed {

    fn update(&self, path: &Path);

}