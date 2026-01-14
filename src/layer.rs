use egui::ecolor::Hsva;
use egui::Color32;

pub enum LayerBlending {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    LinearDodge,
}

pub struct Layer {
    pub type_: LayerBlending,
    pub color: Color32,
    pub variance: f32,
}

impl Layer {
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        let mut hsva = self.color.into();
        ui.color_edit_button_hsva(&mut hsva);
        self.color = Hsva { a: 1.0, ..hsva }.into();
    }
}
