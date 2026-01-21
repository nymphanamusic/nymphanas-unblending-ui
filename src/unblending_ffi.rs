use crate::unblending_ffi::unblending::ColorImage;
#[allow(dead_code)]
use autocxx::prelude::*;
use autocxx::subclass::subclass;
use egui::Color32;
#[allow(unused_imports)]
pub use ffi::unblending;
use ffi::unblending::{AbstractImage, BlendMode};
pub use ffi::unblending_helpers;
use std::pin::Pin;
// Autocxx

include_cpp! {
    #include "unblending/unblending.hpp"
    #include "unblending_helpers.hpp"
    safety!(unsafe_ffi)

    generate!("unblending::BlendMode")
    generate!("unblending::AbstractImage")
    generate!("unblending::ColorImage")
    generate!("unblending::ColorModelPtr")
    generate!("unblending::CompOp")
    generate!("unblending::LayerInfo")
    generate!("unblending::compute_color_unmixing")
    generate!("unblending::perform_matte_refinement")

    // unblending_helpers.cpp
    generate_ns!("unblending_helpers")
}

impl BlendMode {
    pub const ALL: [BlendMode; 13] = [
        BlendMode::Normal,
        BlendMode::Multiply,
        BlendMode::Screen,
        BlendMode::Overlay,
        BlendMode::Darken,
        BlendMode::Lighten,
        BlendMode::ColorDodge,
        BlendMode::ColorBurn,
        BlendMode::HardLight,
        BlendMode::SoftLight,
        BlendMode::Difference,
        BlendMode::Exclusion,
        BlendMode::LinearDodge,
    ];
    pub fn get_name(&mut self) -> &str {
        match self {
            BlendMode::Normal => "Normal",
            BlendMode::Multiply => "Multiply",
            BlendMode::Screen => "Screen",
            BlendMode::Overlay => "Overlay",
            BlendMode::Darken => "Darken",
            BlendMode::Lighten => "Lighten",
            BlendMode::ColorDodge => "Color Dodge",
            BlendMode::ColorBurn => "Color Burn",
            BlendMode::HardLight => "Hard Light",
            BlendMode::SoftLight => "Soft Light",
            BlendMode::Difference => "Difference",
            BlendMode::Exclusion => "Exclusion",
            BlendMode::LinearDodge => "Linear Dodge",
        }
    }
}

// Extra stuff

pub fn color_image_from_pixels(v: &Vec<Color32>, size: [usize; 2]) -> UniquePtr<ColorImage> {
    let mut img = ColorImage::new(c_int(size[0] as i32), c_int(size[1] as i32)).within_unique_ptr();

    for (idx, color) in v.iter().enumerate() {
        unblending_helpers::set_rgba(
            img.pin_mut(),
            c_int((idx % size[0]) as i32),
            c_int((idx / size[1]) as i32),
            color.r() as f64 / 255.0,
            color.g() as f64 / 255.0,
            color.b() as f64 / 255.0,
            color.a() as f64 / 255.0,
        );
    }

    img
}

pub fn pixels_from_color_image(mut image: Pin<&mut ColorImage>, size: [usize; 2]) -> Vec<Color32> {
    let width = size[0];
    let height = size[1];
    let mut out = Vec::with_capacity((width as u16 * height as u16).into());
    for idx in 0..(width * height) {
        let x = (idx % width) as i32;
        let y = ((idx as f32) / height as f32).floor() as i32;
        let mut color = unblending_helpers::get_rgba(
            std::pin::Pin::<&mut ColorImage>::new(&mut *image),
            c_int(x),
            c_int(y),
        )
        .within_unique_ptr();
        out.push(Color32::from_rgba_premultiplied(
            ((&mut color).pin_mut().get_r() * 255.0).floor() as u8,
            ((&mut color).pin_mut().get_g() * 255.0).floor() as u8,
            ((&mut color).pin_mut().get_b() * 255.0).floor() as u8,
            ((&mut color).pin_mut().get_a() * 255.0).floor() as u8,
        ));
    }

    out
}
