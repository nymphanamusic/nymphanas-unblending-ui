use crate::unblending::blend_mode::BlendMode;
use crate::ProcessedLayers;
use eframe::emath::Vec2;
use egui::ecolor::Hsva;
use egui::style::HandleShape;
use egui::{Color32, ComboBox, TextureHandle, Widget};
use egui_extras::{Column, TableBuilder, TableRow};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use strum::IntoEnumIterator;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Layer {
    pub uuid: Uuid,
    pub blend_mode: BlendMode,
    pub color: Color32,
    pub variance: f64,
}

impl PartialEq<Self> for Layer {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for Layer {}

impl Hash for Layer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state)
    }

    fn hash_slice<H: Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Default for Layer {
    fn default() -> Self {
        Layer {
            uuid: Uuid::new_v4(),
            blend_mode: BlendMode::Normal,
            color: Color32::from_gray(255),
            variance: 0.5,
        }
    }
}

impl Layer {
    pub fn draw_all(ui: &mut egui::Ui, layers: &mut Vec<Self>, processed_layers: &ProcessedLayers) {
        let style = ui.style_mut();
        style.visuals.handle_shape = HandleShape::Rect { aspect_ratio: 0.35 };
        style.spacing.interact_size = Vec2::new(60.0, 18.0);

        let available_height = ui.available_height();
        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(false)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(
                Column::remainder()
                    .at_least(200.0)
                    .clip(true)
                    .resizable(true),
            )
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height);

        table.body(|body| {
            body.rows(150.0, layers.len(), |mut row| {
                let layer = &mut layers[row.index()];
                let processed = processed_layers.get(&layer.uuid);
                layer.draw(&mut row, processed);
            })
        });
    }

    pub fn draw(&mut self, row: &mut TableRow, processed: Option<&TextureHandle>) {
        row.col(|ui| {
            ui.vertical(|ui| {
                ComboBox::from_label("Blend mode")
                    .selected_text(<BlendMode as Into<&str>>::into(self.blend_mode))
                    .show_ui(ui, |ui| {
                        BlendMode::iter().for_each(|x| {
                            ui.selectable_value(
                                &mut self.blend_mode,
                                x.clone(),
                                <BlendMode as Into<&str>>::into(x),
                            );
                        });
                    });

                let mut hsva = self.color.into();
                ui.color_edit_button_hsva(&mut hsva);
                self.color = Hsva { a: 1.0, ..hsva }.into();

                ui.horizontal(|ui| {
                    let mut variance = self.variance * 100.0;
                    ui.label("Variance");
                    egui::Slider::new(&mut variance, 0.0..=100.0)
                        .suffix("%")
                        .handle_shape(HandleShape::Rect { aspect_ratio: 0.35 })
                        .ui(ui);
                    self.variance = variance / 100.0;
                });
            });
        });

        row.col(|ui| {
            if let Some(processed_texture) = processed {
                ui.add(egui::Image::new(processed_texture).fit_to_fraction([1.0, 1.0].into()));
            };
        });
    }
}
