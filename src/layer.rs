use crate::unblending::blend_mode::BlendMode;
use egui::ecolor::Hsva;
use egui::{Color32, TextureHandle, Widget};
use std::hash::{Hash, Hasher};
use strum::IntoEnumIterator;
use uuid::Uuid;

#[derive(Clone)]
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

impl Layer {
    pub fn draw(&mut self, ui: &mut egui::Ui, processed: Option<&TextureHandle>) {
        if let Some(processed_texture) = processed {
            ui.add(egui::Image::new(processed_texture).max_size([200.0, 400.0].into()));
        };

        egui::ComboBox::from_label("Blend mode")
            .selected_text(format!("{:?}", self.blend_mode.get_name()))
            .show_ui(ui, |ui| {
                BlendMode::iter().for_each(|x| {
                    ui.selectable_value(
                        &mut self.blend_mode,
                        x.clone(),
                        (&mut x.clone()).get_name(),
                    );
                });
            });

        let mut hsva = self.color.into();
        ui.color_edit_button_hsva(&mut hsva);
        self.color = Hsva { a: 1.0, ..hsva }.into();

        ui.label("Variance");
        let mut variance = self.variance * 100.0;
        egui::Slider::new(&mut variance, 0.0..=100.0)
            .suffix("%")
            .ui(ui);
        self.variance = variance / 100.0;
    }
}
