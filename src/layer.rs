use crate::ui::PainterFrame;
use crate::unblending::blend_mode::BlendMode;
use crate::utils::Flag;
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
        let total_layers: usize;
        let should_delete = Flag::new_rc();
        {
            total_layers = Rc::clone(&layers).borrow().len();
            if row.index() >= total_layers {
                // This is getting hit after deleting a layer that's not at the end. Maybe because of
                // multi-renders?
                return;
            }
            // Layers later in the list composite on top of previous layers
            row_index = total_layers - 1 - row.index();
            layer_uuid = Rc::clone(&layers).borrow()[row_index].uuid;
            layer_index = Rc::clone(&layers)
                .borrow_mut()
                .iter()
                .position(|x| x.uuid == layer_uuid)
                .unwrap();
        }

        // Layer order buttons
        let layers_cp = Rc::clone(&layers);
        let should_delete_cp = Rc::clone(&should_delete);
        {
            let layer_index_cp = layer_index.clone();
            let shift_layer = |offset| {
                let new_index = layer_index_cp as isize + offset;
                if new_index >= 0 && new_index < layers.borrow().len() as isize {
                    layers.borrow_mut().swap(layer_index_cp, new_index as usize);
                }
            };
            let add_layer = |forward| {
                let new_index = if forward {
                    layer_index_cp as isize + 1
                } else {
                    layer_index as isize
                };
                if new_index >= 0 && new_index <= layers.borrow().len() as isize {
                    layers
                        .borrow_mut()
                        .insert(new_index as usize, Layer::default());
                }
            };

            row.col(|ui| {
                ui.vertical_centered(|ui| {
                    let is_top = row_index == layers_cp.borrow().len() - 1;
                    let is_bottom = row_index == 0;

                    let size = 16.0;
                    let paint_stroke =
                        Stroke::new(1.5, ui.style().visuals.widgets.active.fg_stroke.color);
                    let add_button_color = Color32::from_rgb(43, 63, 116);

                    // Move up button
                    if !is_top {
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

                    // Add above button
                    PainterFrame::new(move |painter, rect, hover_progress| {
                        let center = rect.center()
                            + Vec2::new(0.0, egui::lerp(0.0..=-size / 8.0, hover_progress));
                        paint_plus(&painter, center, size, paint_stroke.clone());
                    })
                    .bg_fill(add_button_color)
                    .on_click(|| {
                        add_layer(false);
                    })
                    .ui(ui);

                    // Delete button
                    if total_layers != 1 {
                        PainterFrame::new(move |painter, rect, _hover_progress| {
                            let center = rect.center();
                            paint_x(&painter, center, size, paint_stroke.clone());
                        })
                        .bg_fill(Color32::from_rgb(95, 44, 44))
                        .on_click(|| {
                            should_delete_cp.borrow_mut().set();
                        })
                        .ui(ui);
                    }

                    // Add below button
                    PainterFrame::new(move |painter, rect, hover_progress| {
                        let center = rect.center()
                            + Vec2::new(0.0, egui::lerp(0.0..=size / 8.0, hover_progress));
                        paint_plus(&painter, center, size, paint_stroke.clone());
                    })
                    .bg_fill(add_button_color)
                    .on_click(|| {
                        add_layer(true);
                    })
                    .ui(ui);

                    // Move down button
                    if !is_bottom {
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

        if should_delete.borrow().is_set && total_layers != 1 {
            layers.borrow_mut().remove(layer_index);
        }
    }
}

fn paint_plus(painter: &egui::Painter, center: egui::Pos2, size: f32, stroke: Stroke) {
    const RATIO: f32 = 0.25;
    painter.line_segment(
        [
            center + [-size * RATIO, 0.0].into(),
            center + [size * RATIO, 0.0].into(),
        ],
        stroke,
    );
    painter.line_segment(
        [
            center + [0.0, -size * RATIO].into(),
            center + [0.0, size * RATIO].into(),
        ],
        stroke,
    );
}

fn paint_x(painter: &egui::Painter, center: egui::Pos2, size: f32, stroke: Stroke) {
    const RATIO: f32 = 0.25;
    painter.line_segment(
        [
            center + [-size * RATIO, -size * RATIO].into(),
            center + [size * RATIO, size * RATIO].into(),
        ],
        stroke,
    );
    painter.line_segment(
        [
            center + [-size * RATIO, size * RATIO].into(),
            center + [size * RATIO, -size * RATIO].into(),
        ],
        stroke,
    );
}
