use std::{fs::create_dir, path::Path, process::exit};

use clap::Parser;
use smetrics_rs::{bluesky, update, tumblr, instagram};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Bluesky handle to analyse.
    #[arg(long)]
    bluesky: String,
    /// Tumblr handle to analyse.
    #[arg(long)]
    tumblr: String,
    /// Tumblr API key.
    #[arg(long)]
    tumblr_api_key: String,
    /// Instagram API key.
    #[arg(long)]
    instagram_api_key: String,
    /// Path of data store directory.
    #[arg(long)]
    data: Option<String>,
    /// Maximum days to watch a posts engagement.
    #[arg(long, default_value_t = 2u64)]
    max_watch_days: u64,
    /// Minimum post engagement watch interval ms.
    #[arg(long, default_value_t = 600000u64)]
    min_interval_ms: u64,
    /// Write JSON prettily
    #[arg(long, default_value_t = false)]
    pretty_json: bool
}

#[tokio::main]
async fn main(){
    let args = Args::parse();

    let path = match &args.data {
        Some(p) => Path::new(p),
        None => Path::new("./data")
    };

    check_path(&path);

    update(
        &args.bluesky,
        bluesky::get_user_feed,
        &path.join("bluesky.json"),
        args.max_watch_days,
        args.min_interval_ms,
        args.pretty_json
    ).await;

    update(
        &args.tumblr,
        tumblr::get_user_feed(&args.tumblr_api_key),
        &path.join("tumblr.json"),
        args.max_watch_days,
        args.min_interval_ms,
        args.pretty_json
    ).await;

    update(
        "me",
        instagram::get_user_feed(&args.instagram_api_key, path, args.max_watch_days),
        &path.join("instagram.json"),
        args.max_watch_days,
        args.min_interval_ms,
        args.pretty_json
    ).await;

}

fn check_path(path: &Path) {

    if !path.exists() {
        match create_dir(path) {
            Ok(_) => {println!("Created new data store: {:?}", path);},
            Err(why) => {println!("Could not create new data store: {}", why); exit(1)}
        }
    }

}