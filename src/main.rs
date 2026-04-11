use smetrics_rs::bluesky::feed::get_user_feed;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Bluesky handle to analyse.
    #[arg(short, long)]
    bluesky: String,
}

#[tokio::main]
async fn main(){
    let args = Args::parse();

    let json = get_user_feed(&args.bluesky).await;
    println!("{:?}", json);
}