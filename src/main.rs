#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![expect(rustdoc::missing_crate_level_docs)]

use crate::math::get_sigma;
use crate::unblending::blend_mode::BlendMode;
use crate::unblending::color_model::GaussianColorModel;
use crate::unblending::common::{Mat3, Scalar, Vec3};
use crate::unblending::comp_op::CompOp;
use crate::unblending::layer_info::LayerInfo;
use crate::unblending::unblending::compute_color_unmixing;
use eframe::{egui, Frame};
use egui::load::ImagePoll;
use egui::{Color32, ColorImage, SizeHint, TextureHandle, TextureOptions, Vec2};
use itertools::Itertools;
use layer::Layer;
use rfd::FileDialog;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use tracing::subscriber::set_global_default;
use tracing::{debug, error, info, instrument, Level};
use tracing_subscriber::fmt::SubscriberBuilder;
use uuid::Uuid;

mod layer;
mod math;
mod unblending;

type ProcessedLayers = HashMap<Uuid, TextureHandle>;
type StartProcess = (ImagePoll, Vec<Layer>);
type FinishProcess = HashMap<Uuid, ColorImage>;

#[instrument]
fn main() -> eframe::Result {
    let my_collector = SubscriberBuilder::default()
        .with_max_level(Level::DEBUG)
        .finish();
    set_global_default(my_collector).expect("setting tracing default failed");
    info!("Starting application");

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
    processed_layers: ProcessedLayers,

    processor_handle: Option<JoinHandle<Result<(), ()>>>,
    start_process_tx: Option<Sender<StartProcess>>,
    finish_process_rx: Option<Receiver<FinishProcess>>,
}

impl Default for NymphanasUnblendingUI<'_> {
    fn default() -> Self {
        Self {
            image_path: None,
            image: None,
            layers: vec![Layer::default()],
            processed_layers: HashMap::new(),
            processor_handle: None,
            start_process_tx: None,
            finish_process_rx: None,
        }
    }
}

impl Debug for NymphanasUnblendingUI<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NymphanasUnblendingUI")
            .field("image_path", &self.image_path)
            // .field("image", &self.image)
            .field("layers", &self.layers)
            .field("start_process_tx", &self.start_process_tx)
            .field("finish_process_rx", &self.finish_process_rx)
            .finish()
    }
}

impl eframe::App for NymphanasUnblendingUI<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.ensure_processor();
        self.receive_processed_layers(&ctx);

        ctx.style_mut(|style| {
            style.spacing.interact_size = Vec2::new(60.0, 30.0);
            style.spacing.item_spacing = Vec2::new(10.0, 10.0);
        });

        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.heading("Nymphana's Unblending UI");

            if ui.button("Run unblending").clicked() {
                self.handle_begin_processing(&ctx);
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
                self.layers.push(Layer::default());
            }
            for (idx, layer) in self.layers.iter_mut().enumerate() {
                ui.push_id(idx, |ui| {
                    layer.draw(ui, self.processed_layers.get(&layer.uuid))
                });
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
    #[instrument]
    fn load_image(&mut self) {
        info!("Loading image");
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

    #[instrument]
    fn handle_begin_processing(&mut self, ctx: &egui::Context) {
        info!("Begin processing");
        if let Some(image_path) = &self.image_path
            && let Some(start_process_tx) = &self.start_process_tx
        {
            debug!("Loading image");
            match ctx.try_load_image(
                &format!("bytes://{}", image_path.to_str().unwrap()),
                SizeHint::Scale(1.0.into()),
            ) {
                Ok(image_poll) => {
                    debug!("Sending image to processor");
                    start_process_tx
                        .send((image_poll, self.layers.clone()))
                        .unwrap();
                }
                Err(err) => {
                    error!(?err, "Failed to load image.");
                }
            }
        }
    }

    fn ensure_processor(&mut self) {
        if self.processor_handle.is_some() {
            return;
        }
        let (start_process_tx, start_process_rx) = mpsc::channel();
        let (finish_process_tx, finish_process_rx) = mpsc::channel();
        self.processor_handle = Some(thread::spawn(move || {
            processor(start_process_rx, finish_process_tx)
        }));

        self.start_process_tx = Some(start_process_tx);
        self.finish_process_rx = Some(finish_process_rx);
    }

    #[instrument]
    fn receive_processed_layers(&mut self, ctx: &egui::Context) {
        if let Some(finish_process_rx) = &self.finish_process_rx {
            if let Ok(received) = finish_process_rx.try_recv() {
                debug!("Receiving processed layer");
                self.processed_layers =
                    HashMap::from_iter(received.iter().map(|(&uuid, color_image)| {
                        (
                            uuid,
                            ctx.load_texture(
                                "processed.png",
                                color_image.clone(),
                                TextureOptions::LINEAR,
                            ),
                        )
                    }))
            }
        }
    }
}

#[instrument]
fn processor(
    start_process_rx: Receiver<StartProcess>,
    finish_process_tx: Sender<FinishProcess>,
) -> Result<(), ()> {
    for (image_poll, layers) in start_process_rx {
        debug!("Received image process request; loading");
        while let ImagePoll::Pending { size: _ } = image_poll {
            thread::sleep(Duration::from_secs_f32(0.1))
        }
        let ImagePoll::Ready { image } = image_poll else {
            panic!("ImagePoll stopped pending but had no Ready")
        };
        debug!("Beginning image processing");
        let processed = do_process(&image, &layers);
        debug!("Sending processed image back to main thread");
        finish_process_tx.send(processed).unwrap();
    }
    Ok(())
}

#[instrument]
fn do_process<'a>(image: &ColorImage, layers: &Vec<Layer>) -> HashMap<Uuid, ColorImage> {
    debug!("Converting egui image to unblending image");
    let color_image = unblending::color_image::ColorImage {
        width: image.width(),
        height: image.height(),
        pixels: image
            .pixels
            .iter()
            .flat_map(|x| [x.r(), x.g(), x.b(), x.a()])
            .map(|x| x as Scalar)
            .collect_vec(),
    };
    debug!("Collecting layer infos");
    let layer_infos = layers
        .iter()
        .map(|layer| LayerInfo {
            comp_op: CompOp::SourceOver(),
            blend_mode: layer.blend_mode.clone(),
            color_model: GaussianColorModel {
                mu: Vec3::new(
                    layer.color.r() as f64 / 255.0,
                    layer.color.g() as f64 / 255.0,
                    layer.color.b() as f64 / 255.0,
                ),
                sigma_inv: get_sigma(layer.variance).try_inverse().unwrap(),
            },
        })
        .collect_vec();

    HashMap::from_iter(
        layers.iter().map(|x| x.uuid).zip(
            compute_color_unmixing(
                &color_image,
                &layer_infos.iter().by_ref().collect_vec(),
                true,
                None,
            )
            .iter()
            .map(|x| {
                ColorImage::new(
                    image.size,
                    x.iter_rgba()
                        .map(|x| {
                            Color32::from_rgba_premultiplied(
                                x.x as u8, x.y as u8, x.z as u8, x.w as u8,
                            )
                        })
                        .collect_vec(),
                )
            }),
        ),
    )
}
