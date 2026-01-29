use crate::ui::PainterFrame;
use crate::unblending::blend_mode::BlendMode;
use crate::ProcessedLayers;
use eframe::emath::Vec2;
use egui::ecolor::Hsva;
use egui::style::HandleShape;
use egui::{Color32, ComboBox, Stroke, Widget};
use egui_extras::{Column, TableBuilder, TableRow};
use std::cell::RefCell;
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
            .column(Column::exact(30.0))
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
                Layer::draw(layers, &mut row, processed_layers);
            })
        });
    }

    pub fn draw<'a>(
        layers: &mut Vec<Self>,
        row: &mut TableRow,
        processed_layers: &ProcessedLayers,
    ) {
        let layers = RefCell::new(layers);

        // Layer order buttons
        {
            let shift_layer = |offset, uuid: Uuid| {
                let index = layers
                    .borrow_mut()
                    .iter()
                    .position(|x| x.uuid == uuid)
                    .unwrap();
                let new_index = index as isize + offset;
                if new_index >= 0 && new_index < layers.borrow().len() as isize {
                    layers.borrow_mut().swap(index, new_index as usize);
                }
            };

            let layer_uuid = layers.borrow()[row.index()].uuid;
            row.col(move |ui| {
                ui.vertical_centered(move |ui| {
                    let size = 16.0;
                    let paint_stroke =
                        Stroke::new(3.0, ui.style().visuals.widgets.active.fg_stroke.color);

                    // Move up button
                    PainterFrame::new(move |painter, rect, hover_progress| {
                        let center = rect.center()
                            + Vec2::new(0.0, egui::lerp(0.0..=-size / 8.0, hover_progress));
                        painter.line(
                            vec![
                                center + [-size / 4.0, size / 6.0].into(),
                                center + [0.0, -size / 6.0].into(),
                                center + [size / 4.0, size / 6.0].into(),
                            ],
                            paint_stroke.clone(),
                        );
                    })
                    .on_click(move || {
                        shift_layer(-1, layer_uuid);
                    })
                    .ui(ui);

                    // Move down button
                    PainterFrame::new(move |painter, rect, hover_progress| {
                        let center = rect.center()
                            + Vec2::new(0.0, egui::lerp(0.0..=size / 8.0, hover_progress));
                        painter.line(
                            vec![
                                center + [-size / 4.0, -size / 6.0].into(),
                                center + [0.0, size / 6.0].into(),
                                center + [size / 4.0, -size / 6.0].into(),
                            ],
                            paint_stroke.clone(),
                        );
                    })
                    .on_click(move || shift_layer(1, layer_uuid))
                    .ui(ui);
                });
            });
        }

        // Controls
        {
            let layer = &mut (layers.borrow_mut()[row.index()]);
            row.col(|ui| {
                ui.vertical(|ui| {
                    ComboBox::from_label("Blend mode")
                        .selected_text(<BlendMode as Into<&str>>::into(layer.blend_mode))
                        .show_ui(ui, |ui| {
                            BlendMode::iter().for_each(|x| {
                                ui.selectable_value(
                                    &mut layer.blend_mode,
                                    x.clone(),
                                    <BlendMode as Into<&str>>::into(x),
                                );
                            });
                        });

                    let mut hsva = layer.color.into();
                    ui.color_edit_button_hsva(&mut hsva);
                    layer.color = Hsva { a: 1.0, ..hsva }.into();

                    ui.horizontal(|ui| {
                        let mut variance = layer.variance * 100.0;
                        ui.label("Variance");
                        egui::Slider::new(&mut variance, 0.0..=100.0)
                            .suffix("%")
                            .handle_shape(HandleShape::Rect { aspect_ratio: 0.35 })
                            .ui(ui);
                        layer.variance = variance / 100.0;
                    });
                });
            });
        }

        // Preview image
        {
            let layer = &mut (layers.borrow_mut()[row.index()]);
            let processed_layer = processed_layers.get(&layer.uuid);
            row.col(|ui| {
                if let Some(processed_texture) = processed_layer {
                    ui.add(egui::Image::new(processed_texture).fit_to_fraction([1.0, 1.0].into()));
                };
            });
        }
    }
}
