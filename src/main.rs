mod downloader_api;

use eframe::{App, NativeOptions, run_simple_native};
use eframe::egui::{CentralPanel, TextBuffer};
use tokio::{spawn};
use crate::downloader_api::download_video;

fn download_wrapper(link: String, folder: String) {
    spawn(async move {
        let a= link.clone();
        let b= folder.clone();
        download_video(&a, &b).await;
    });
}

#[tokio::main]
async fn main() {
    let mut video_link = "".to_owned();
    let window_options = NativeOptions::default();
    let start_result = run_simple_native("Video Downloader", window_options, move |ctx, _frame| {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("HLS Video Downloader");

            ui.horizontal(|ui| {
                let link_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut video_link)
                    .labelled_by(link_label.id);
            });
            if ui.button("Download!").clicked() {
                download_wrapper(video_link.clone(), String::from("chunks"))
            }
        });
    });

    match start_result {
        Err(error) => panic!("Failed to start app. Error: {}", error),
        Ok(()) => {
            println!("App Closed!");
        }
    }
}
