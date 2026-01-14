#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![expect(rustdoc::missing_crate_level_docs)]

use crate::layer::LayerBlending;
use eframe::{egui, Frame};
use egui::Color32;
use layer::Layer;
use std::ops::{Deref, DerefMut};
use std::path::Path;

mod layer;

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

struct NymphanasUnblendingUI {
    image: Option<Box<Path>>,
    layers: Vec<Layer>,
}

impl Default for NymphanasUnblendingUI {
    fn default() -> Self {
        Self {
            image: None,
            layers: vec![],
        }
    }
}

impl eframe::App for NymphanasUnblendingUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Nymphana's Unblending UI");
            if ui.button("Add layer").clicked() {
                self.layers.push(Layer {
                    type_: LayerBlending::Normal,
                    color: Color32::from_gray(255).into(),
                    variance: 0.5,
                });
            }
            for layer in &mut self.layers {
                layer.draw(ui)
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
