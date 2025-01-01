mod downloader_api;

use std::path::Path;
use eframe::{App, NativeOptions, run_simple_native};
use eframe::egui::{CentralPanel, TextBuffer, Vec2};
use egui_file_dialog::{FileDialog};
use egui_modal::Modal;
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
    let mut video_link: String = Default::default();
    let window_options = NativeOptions::default();
    let mut dialog = FileDialog::new().default_size(Vec2::new(200.0,200.0));
    let start_result = run_simple_native("Video Downloader", window_options, move |ctx, _frame| {
        let mut download_path: &Path = Path::new("");

        CentralPanel::default().show(ctx, |ui| {
            let mut modal = Modal::new(&ctx, "error");
            ui.heading("HLS Video Downloader");

            ui.horizontal(|ui| {
                let link_label = ui.label("Video Link: ");
                ui.text_edit_singleline(&mut video_link)
                    .labelled_by(link_label.id);
            });

            if ui.button("Select Folder").clicked() {
                dialog.select_directory();
            }
            if let Some(path) = dialog.update(ctx).selected() {
                download_path = path.strip_prefix("\"").unwrap();
                println!("Path: {}", download_path.to_str().unwrap());
            }
            if ui.button("Download!").clicked() {
                println!("Download path: {}", download_path.to_str().unwrap());
                if download_path.to_str().unwrap().len() == 0 {
                    modal.title(ui, "Path is required!");
                    modal.show(|ui| {});
                    return;
                }
                let formatted_path = String::from(download_path.to_str().unwrap());
                download_wrapper(video_link.clone(), formatted_path);
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
