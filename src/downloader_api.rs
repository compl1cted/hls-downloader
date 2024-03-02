use std::env;
use std::io::prelude::*;
use std::fs::{create_dir, File, remove_dir_all};
use std::process::Command;
use reqwest::{Client, Response, StatusCode};
use reqwest::header::USER_AGENT;
use futures_util::StreamExt;

async fn download_segment(url: &String, segment_id: i32, directory: &String) -> Option<()> {
    let client: Client = Client::new();
    let request_url: String = format!("{}:hls:seg-{}-v1-a1.ts", url, segment_id);
    let filepath: String = format!("{}/{}.ts", directory, segment_id);

    let response: Response = match client.get(&request_url).header("Header", USER_AGENT).send().await {
        Ok(response) => response,
        Err(error) => {
            println!("Failed to request video segment! Error {error}");
            return None;
        }
    };

    if response.status().is_client_error() {
        return None;
    }

    println!("Starting segment {} download", segment_id);
    let mut stream = response.bytes_stream();

    let mut file: File = match File::create(filepath) {
        Ok(file) => file,
        Err(error) => {
            println!("File creation error: {}", error);
            return None;
        }
    };

    while let Some(byte) = stream.next().await {
        let data = match byte {
            Ok(data) => data,
            Err(error) => {
                println!("Failed to access chunk data stream! Error: {error}");
                return None;
            }
        };

        match file.write_all(&data) {
            Ok(()) => (),
            Err(error) => {
                println!("Failed to write data segment into file! Error: {error}");
                return None;
            }
        };
    };

    println!("Segment {segment_id} downloaded!");

    return Some(());
}

fn merge_segments(directory: &String, list_filename: &String, output_filename: String) {
    let output = Command::new("powershell.exe")
        .arg("ffmpeg")
        .arg(format!("-f concat -safe 0 - {list_filename} -c copy {directory}/{output_filename}.mp4"))
        .output()
        .unwrap();

    println!("{:?}", output);
}

async fn segment_exist(url: &String, segment_id: i32) -> Option<bool> {
    let client: Client = Client::new();
    let request_url: String = format!("{url}:hls:seg-{segment_id}-v1-a1.ts");

    let response: Response = match client.get(&request_url).send().await {
        Ok(response) => response,
        Err(error) => {
            println!("Failed to request video segment! Error: {error}");
            return None;
        }
    };

    if response.status() == StatusCode::NOT_FOUND {
        println!("Segment {segment_id} is not found");
        return Some(false);
    }

    return Some(true);
}

async fn get_video_segment_count(url: &String) -> i32 {
    let mut segment_count: i32 = 1;
    let mut step: i32 = 50;
    let mut direction: i32 = 1;
    let mut last_valid_segment: i32 = 0;
    loop {
        match segment_exist(&url, segment_count).await {
            Some(true) => {
                last_valid_segment = segment_count;
                if direction != 1 {
                    direction = 1;
                    step /= 2
                }
            },
            Some(false) => {
                if segment_count - last_valid_segment == 1 { break }
                if direction != -1 {
                    direction = -1;
                    step /= 2;
                }
            },
            None => {
                println!("Failed to request segment!");
                return 0;
            }
        };
        // println!("Segments count: {segment_count}, Last valid segment: {last_valid_segment}, Step: {step}, Direction: {direction}");
        if step == 0 {step = 1};
        segment_count += step * direction;
    }
    println!("Segment count: {last_valid_segment}");

    return last_valid_segment + 1;
}

pub async fn download_video(url: &String, directory: &String) -> Option<()> {

    match remove_dir_all(&directory) {
        Ok(()) => (),
        Err(error) => {
            println!("Failed to clear segment directory: {error}");
            return None
        }
    };

    match create_dir(&directory) {
        Ok(()) => (),
        Err(error) => {
            println!("Failed to create segment directory: {error}");
            return None
        }
    };

    let record_filepath = String::from("segment_list.txt");
    let mut segment_record_file = match File::create(record_filepath.clone()) {
        Ok(file) => file,
        Err(error) => {
            println!("Failed to create segment record file! Error: {error}",);
            return None;
        }
    };

    let current_dir: String = match env::current_dir() {
        Ok(current_directory) => current_directory.display().to_string(),
        Err(error) => {
            println!("Failed to access current working directory! Error: {error}");
            return None;
        }
    };

    let segment_count = get_video_segment_count(&url).await;

    let mut handles = vec![];

    for segment_id in 1..segment_count {
        let req_url = url.clone();
        let dir = directory.clone();
        let handle = tokio::spawn(async move {
            download_segment(&req_url, segment_id, &dir).await;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    for i in 1..segment_count {
        let record = format!(r#"file '{}\{}\{}.ts'{}"#, current_dir, &directory, i, "\n");
        let _ = segment_record_file.write(record.as_bytes());
    }

    let _ = merge_segments(&directory, &record_filepath, String::from("output"));
    println!("Video downloaded successfully.");
    return Some(())
}