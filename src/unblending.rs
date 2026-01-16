use autocxx::prelude::*;

include_cpp! {
    #include "unblending/unblending.hpp"
    safety!(unsafe_ffi)
    generate!("unblending::BlendMode")
    generate!("unblending::ColorImage")
    generate!("unblending::ColorModelPtr")
    generate!("unblending::CompOp")
    generate!("unblending::LayerInfo")
    generate!("unblending::compute_color_unmixing")
    generate!("unblending::perform_matte_refinement")
}

use crate::unblending::unblending::BlendMode;

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

#[allow(unused_imports)]
pub use ffi::*;
