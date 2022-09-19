use aws_sdk_s3::model::{BucketLocationConstraint, CreateBucketConfiguration};
use aws_sdk_s3::types::ByteStream;
use aws_sdk_s3::{Client, Error};
use std::path::Path;
use tokio::fs::File;
use tokio::io;

// TODO: get real save path.
const SAVE_PATH: &str = "test.txt";
const BUCKET_NAME: &str = "darksouls-cloud-save";
// TODO: document this on readme.md.
const BUCKET_REGION: &str = "sa-east-1";

#[tokio::main]
async fn main() -> Result<(), Error> {
    // TODO: document this on readme.md.
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    create_bucket_if_not_exists(&client).await;

    println!("What do you want to do?\n1. Upload my save file.\n2. Download my save file.");

    let mut input = String::new();

    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let input: String = input
        .trim()
        .parse()
        .expect("This is not an available option");

    match input.as_str() {
        "1" => {
            let _ = upload_object(&client)
                .await
                .expect("Error while uploading the save file!");
        }
        "2" => {
            let _ = download_object(&client)
                .await
                .expect("Error while downloading the save file!");
        }
        _ => panic!("This is not an available option, aborting."),
    }

    Ok(())
}

async fn create_bucket_if_not_exists(client: &Client) {
    let bucket_exists = bucket_exists(&client)
        .await
        .expect("Error while reading the AWS S3 Bucket!");
    if !bucket_exists {
        println!("AWS S3 bucket don't exists. Creating one...");
        let _ = create_bucket(&client).await;
    }
}

async fn create_bucket(client: &Client) -> Result<(), Error> {
    let constraint = BucketLocationConstraint::from(BUCKET_REGION);
    let cfg = CreateBucketConfiguration::builder()
        .location_constraint(constraint)
        .build();
    client
        .create_bucket()
        .create_bucket_configuration(cfg)
        .bucket(BUCKET_NAME)
        .send()
        .await
        .expect("Oops, we couldn't create the bucket!");
    Ok(())
}

async fn bucket_exists(client: &Client) -> Result<bool, Error> {
    let bucket_list = list_buckets(client)
        .await
        .expect("Oops, something went wrong while we verified the AWS S3 bucket existence!");
    for x in bucket_list {
        if x == BUCKET_NAME {
            return Ok(true);
        }
    }
    Ok(false)
}

async fn upload_object(client: &Client) -> Result<(), Error> {
    let body = ByteStream::from_path(Path::new(&SAVE_PATH))
        .await
        .expect("Error while reading the save file from disk!");
    let _ = &client
        .put_object()
        .bucket(BUCKET_NAME)
        // TODO: get file name from path.
        .key(BUCKET_NAME)
        .body(body)
        .send()
        .await
        .expect("Error while uploading the save file!");
    println!("Successfully uploaded the file.");
    Ok(())
}

pub async fn download_object(client: &Client) -> Result<(), Error> {
    let resp = client
        .get_object()
        .bucket(BUCKET_NAME)
        // TODO: get file name from path.
        .key(BUCKET_NAME)
        .send()
        .await
        .expect("Error while downloading the save file!");
    let mut body = resp.body.into_async_read();
    let mut file = File::create(SAVE_PATH).await.expect("");
    io::copy(&mut body, &mut file).await.expect("");
    Ok(())
}

async fn list_buckets(client: &Client) -> Result<Vec<String>, Error> {
    let response = &client.list_buckets().send().await?;
    let mut buffer = vec![];
    for bucket in response.buckets().unwrap_or_default() {
        buffer.push(String::from(bucket.name().unwrap_or_default()));
    }
    Ok(buffer)
}
