#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![expect(rustdoc::missing_crate_level_docs)]

use eframe::{egui, Frame};

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
    name: String,
    age: u32,
}

impl Default for NymphanasUnblendingUI {
    fn default() -> Self {
        Self {
            name: "Nymphana".to_owned(),
            age: 26,
        }
    }
}

impl eframe::App for NymphanasUnblendingUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Nymphana's Unblending UI");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
                ui.add(egui::Slider::new(&mut self.age, 0..=100).text("age"));
                if ui.button("Increment").clicked() {
                    self.age += 1;
                }
                ui.label(format!(
                    "Hey there {0} the {1} year old!",
                    self.name, self.age
                ));
            });
        });
    }
}
