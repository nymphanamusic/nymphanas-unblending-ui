#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![expect(rustdoc::missing_crate_level_docs)]

use crate::unblending::unblending::BlendMode;
use eframe::wgpu::TextureViewDescriptor;
use eframe::{egui, egui_wgpu, Frame};
use egui::load::TexturePoll;
use egui::{Color32, SizeHint, TextureOptions, Vec2};
use layer::Layer;
use rfd::FileDialog;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

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
    processor_handle: Option<JoinHandle<Result<(), ()>>>,
    start_process_tx: Option<Sender<StartProcess>>,
}

impl Default for NymphanasUnblendingUI<'_> {
    fn default() -> Self {
        Self {
            image_path: None,
            image: None,
            layers: vec![],
            processor_handle: None,
            start_process_tx: None,
        }
    }
}

impl eframe::App for NymphanasUnblendingUI<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        if let Some((start_process_tx, finish_process_rx)) = self.ensure_processor() {}

        ctx.style_mut(|style| {
            style.spacing.interact_size = Vec2::new(60.0, 30.0);
            style.spacing.item_spacing = Vec2::new(10.0, 10.0);
        });

        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.heading("Nymphana's Unblending UI");

            if ui.button("Run unblending").clicked() {
                self.handle_begin_processing(&ctx, &_frame);
            }

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

    fn handle_begin_processing(&mut self, ctx: &egui::Context, frame: &Frame) {
        if let Some(image) = &self.image
            && let Some(start_process_tx) = &self.start_process_tx
            && let Some(render_state) = frame.wgpu_render_state()
        {
            let load_result = image.source(&ctx).load(
                &ctx,
                TextureOptions::default(),
                SizeHint::Scale(1.0.into()),
            );
            if let Ok(load_poll) = load_result {
                start_process_tx
                    .send((load_poll, render_state.renderer.clone()))
                    .unwrap();
            }
        }
    }

    fn ensure_processor(&mut self) -> Option<(Sender<StartProcess>, Receiver<FinishProcess>)> {
        if self.processor_handle.is_some() {
            return None;
        }
        let (start_process_tx, start_process_rx) = mpsc::channel();
        let (finish_process_tx, finish_process_rx) = mpsc::channel();
        self.processor_handle = Some(thread::spawn(move || {
            processor(start_process_rx, finish_process_tx)
        }));

        Some((start_process_tx, finish_process_rx))
    }
}

type StartProcess = (TexturePoll, Arc<egui::mutex::RwLock<egui_wgpu::Renderer>>);
type FinishProcess = ();

fn processor(
    start_process_rx: Receiver<StartProcess>,
    finish_process_tx: Sender<FinishProcess>,
) -> Result<(), ()> {
    // (*_frame.wgpu_render_state().unwrap().renderer).read().texture()
    for (image_poll, renderer) in start_process_rx {
        while image_poll.is_pending() {
            thread::sleep(Duration::from_secs_f32(0.1))
        }
        {
            let read = renderer.read();
            let texture = read.texture(&image_poll.texture_id().unwrap());
            if let Some(texture) = texture
                && let Some(texture) = &texture.texture
            {
                let view = texture.create_view(&TextureViewDescriptor::default());
                view.texture().dimension()
            }
        }
    }
    Ok(())
}
