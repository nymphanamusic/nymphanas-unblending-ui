use crate::unblending::unblending::BlendMode;
use egui::ecolor::Hsva;
use egui::{Color32, Widget};

pub struct Layer {
    pub blend_mode: BlendMode,
    pub color: Color32,
    pub variance: f32,
}

impl Layer {
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("Take your pick")
            .selected_text(format!("{:?}", self.blend_mode.get_name()))
            .show_ui(ui, |ui| {
                BlendMode::ALL.iter().for_each(|x| {
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
