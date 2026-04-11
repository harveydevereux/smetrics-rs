use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};


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

/// Obtain IG media ids for a user.
///
/// Note the access token is enough to identify the user 'me'.
/// See https://developers.facebook.com/docs/instagram-platform/reference/instagram-media
pub async fn get_media(key: &str, id: &String) -> Post {
    let req = reqwest::get(format!("https://graph.instagram.com/v25.0/{}?fields=permalink,comments_count,like_count,timestamp,caption&access_token={}", id, key)).await.unwrap();
    req.json::<Post>().await.unwrap()
}

/// Obtain Posts for a given IG user.
pub fn get_user_feed(key: &str) -> impl AsyncFn(&str) -> Vec<Box<dyn crate::Post>> {
    async move |user: &str| -> Vec<Box<dyn crate::Post>> {

        let media = get_user_media(user, key).await;

        let fut_posts = media.iter().map(|id| get_media(key, id));
        let posts = futures::future::join_all(fut_posts).await;

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

}