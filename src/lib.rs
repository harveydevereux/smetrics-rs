pub mod bluesky;

pub trait Post {

    fn hashtags(&self) -> Vec<&str>;

}