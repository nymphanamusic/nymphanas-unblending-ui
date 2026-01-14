use egui::Color32;
use std::ops::Deref;

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

#[derive(Clone, Copy)]
pub struct Color32Wrapper(Color32);

impl Deref for Color32Wrapper {
    type Target = Color32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<[f32; 3]> for Color32Wrapper {
    fn into(self) -> [f32; 3] {
        [self.r(), self.g(), self.b()].map(|x| f32::from(x) / 255.0)
    }
}

impl From<Color32> for Color32Wrapper {
    fn from(value: Color32) -> Self {
        Color32Wrapper(value)
    }
}

impl From<&[f32; 3]> for Color32Wrapper {
    fn from(rgb: &[f32; 3]) -> Self {
        let [r, g, b] = rgb.map(|x| (x * 255.0).floor() as u8);
        Color32Wrapper(Color32::from_rgb(r, g, b))
    }
}

pub struct Layer {
    pub type_: LayerBlending,
    pub color: Color32Wrapper,
    pub variance: f32,
}

impl Layer {
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        let mut rgb = self.color.into();
        ui.color_edit_button_rgb(&mut rgb);
        self.color = Color32Wrapper::from(&rgb);
    }
}
