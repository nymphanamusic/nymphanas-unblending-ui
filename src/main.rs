#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![expect(rustdoc::missing_crate_level_docs)]

use crate::unblending::unblending::BlendMode;
use eframe::{egui, Frame};
use egui::{Color32, Vec2};
use layer::Layer;
use rfd::FileDialog;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

mod layer;
mod unblending;

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Nymphana's Unblending UI",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<NymphanasUnblendingUI>::default())
        }),
    )
}

struct NymphanasUnblendingUI<'a> {
    image_path: Option<PathBuf>,
    image: Option<Box<egui::Image<'a>>>,
    layers: Vec<Layer>,
}

impl Default for NymphanasUnblendingUI<'_> {
    fn default() -> Self {
        Self {
            image_path: None,
            image: None,
            layers: vec![],
        }
    }
}

impl eframe::App for NymphanasUnblendingUI<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        ctx.style_mut(|style| {
            style.spacing.interact_size = Vec2::new(60.0, 30.0);
            style.spacing.item_spacing = Vec2::new(10.0, 10.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Nymphana's Unblending UI");

            if ui.button("Run unblending").clicked() {}

            ui.horizontal(|ui| {
                if ui.button("Select file").clicked() {
                    let files = FileDialog::new()
                        .add_filter("Image", &["bmp", "jpeg", "jpg", "png"])
                        .set_directory("~")
                        .pick_file();
                    files.inspect(|x| self.image_path = Some(x.into()));
                    self.load_image();
                }
            });

            // Image
            ui.label(format!(
                "Selected file: {}",
                match &self.image_path {
                    Some(y) => y.to_str().unwrap_or("None"),
                    None => "None",
                }
            ));
            if let Some(image) = &self.image {
                ui.add((*image.clone()).max_size([200.0, 400.0].into()));
            };

            // Layers
            if ui.button("Add layer").clicked() {
                self.layers.push(Layer {
                    blend_mode: BlendMode::Normal,
                    color: Color32::from_gray(255),
                    variance: 0.5,
                });
            }
            for (idx, layer) in self.layers.iter_mut().enumerate() {
                ui.push_id(idx, |ui| layer.draw(ui));
            }
            for (idx, layer) in self.layers.iter().enumerate() {
                ui.label(format!(
                    "Layer \"{layer_idx}\" has color {color}",
                    layer_idx = idx,
                    color = layer.color.to_hex()
                ));
            }
        });
    }
}

impl NymphanasUnblendingUI<'_> {
    fn load_image(&mut self) {
        if let Some(path) = &self.image_path
            && let Ok(mut file) = File::open(path)
        {
            let mut bytes = Vec::new();
            if file.read_to_end(&mut bytes).is_ok() {
                self.image = Some(Box::new(egui::Image::from_bytes(
                    format!["bytes://{}", path.to_str().unwrap()],
                    bytes,
                )));
            }
        }
    }
}
