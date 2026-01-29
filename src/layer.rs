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
use std::rc::Rc;
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
        let layers = Rc::new(RefCell::new(layers));

        let row_index: usize;
        let layer_uuid: Uuid;
        let layer_index: usize;
        {
            row_index = row.index();
            layer_uuid = Rc::clone(&layers).borrow()[row_index].uuid;
            layer_index = Rc::clone(&layers)
                .borrow_mut()
                .iter()
                .position(|x| x.uuid == layer_uuid)
                .unwrap();
        }

        // Layer order buttons
        let layers_3 = Rc::clone(&layers);
        {
            let layer_index_2 = layer_index.clone();
            let shift_layer = |offset| {
                let new_index = layer_index_2 as isize + offset;
                if new_index >= 0 && new_index < layers.borrow().len() as isize {
                    layers.borrow_mut().swap(layer_index_2, new_index as usize);
                }
            };

            row.col(|ui| {
                ui.vertical_centered(move |ui| {
                    let size = 16.0;
                    let paint_stroke =
                        Stroke::new(3.0, ui.style().visuals.widgets.active.fg_stroke.color);

                    // Move up button
                    if row_index != 0 {
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
                        .on_click(|| {
                            shift_layer(-1);
                        })
                        .ui(ui);
                    }

                    // Move down button
                    if row_index != layers_3.borrow().len() - 1 {
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
                        .on_click(|| shift_layer(1))
                        .ui(ui);
                    }
                });
            });
        }

        // Controls
        let layers_2 = Rc::clone(&layers);
        {
            let layer = &mut (layers_2.borrow_mut()[row.index()]);
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
