
use std::{path::PathBuf, env};
use aws_config::{meta::region::RegionProviderChain, Region};
use aws_sdk_s3::Client;
use indicatif::ProgressBar;
use trackable_s3_stream::TrackableBodyStream;

#[tokio::main(flavor="current_thread")]
async fn main() {
    dotenv::dotenv().ok();

    let mut body = TrackableBodyStream::try_from(PathBuf::from("./examples/sample.jpeg")).map_err(|e| {
        panic!("Could not open sample file: {}", e);
    }).unwrap();

    let bar = ProgressBar::new(body.content_length() as u64);

    body.set_callback(move |tot_size: u64, sent: u64, cur_buf: u64| {
        bar.inc(cur_buf as u64);
        if sent == tot_size {
            bar.finish();
        }
    });

    let region_provider = RegionProviderChain::default_provider().or_else(Region::new("us-east-1"));
    let endpoint = String::from("https://s3.us-west-004.backblazeb2.com");
    let bucket = env::var("AWS_BUCKET").expect("AWS_BUCKET environment variable not set");
    let config = aws_config::from_env()
        .region(region_provider)
        .endpoint_url(endpoint)
        .load()
        .await;

    let client = Client::new(&config);

    println!("Uploading to {}", bucket);
    match client.put_object()
                    .bucket(bucket)
                    .key("tracked_sample.jpeg")
                    .content_length(body.content_length())
                    .body(body.to_s3_stream())
                    .send().await {
                        Ok(_) => {
                            println!("Upload complete");
                        },
                        Err(e) => {
                            panic!("Failed to upload object: {}", e);
                        }
                    }
}
