use autocxx::prelude::*;
use egui::Color32;

include_cpp! {
    #include "unblending/unblending.hpp"
    #include "unblending_helpers.hpp"
    safety!(unsafe_ffi)

    extern_cpp_opaque_type!("Eigen::Vector3d", crate::unblending::ffi_manual::Vector3d)
    extern_cpp_opaque_type!("Eigen::Vector4d", crate::unblending::ffi_manual::Vector4d)
    extern_cpp_opaque_type!("Eigen::Matrix3d", crate::unblending::ffi_manual::Matrix3d)

    generate!("unblending::BlendMode")
    generate!("unblending::ColorImage")
    generate!("unblending::ColorModelPtr")
    generate!("unblending::CompOp")
    generate!("unblending::LayerInfo")
    generate!("unblending::compute_color_unmixing")
    generate!("unblending::perform_matte_refinement")

    // unblending_helpers.cpp
    generate!("unblending::make_matrix3d")
    generate!("unblending::make_vector3d")
    generate!("unblending::make_vector4d")
    generate!("unblending::make_layer_info")
}
#[cxx::bridge(namespace = "Eigen")]
pub mod ffi_manual {
    unsafe extern "C++" {
        include!("Eigen/Core");
        include!("Eigen/src/Core/Matrix.h");

        type Vector3d;
        #[Self = "Vector3d"]
        fn Zero() -> UniquePtr<Vector3d>;

        type Vector4d;
        #[Self = "Vector4d"]
        fn Zero() -> UniquePtr<Vector4d>;

        type Matrix3d;
        #[Self = "Matrix3d"]
        fn Zero() -> UniquePtr<Matrix3d>;
    }
}

use crate::unblending::unblending::{make_vector4d, BlendMode, ColorImage};

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

pub fn color_image_from_vec(v: &Vec<Color32>, size: [usize; 2]) -> UniquePtr<ColorImage> {
    let mut img = ColorImage::new(c_int(size[0] as i32), c_int(size[1] as i32)).within_unique_ptr();

    for (idx, color) in v.iter().enumerate() {
        let mut color = make_vector4d(
            color.r() as f32 / 255.0,
            color.g() as f32 / 255.0,
            color.b() as f32 / 255.0,
            color.a() as f32 / 255.0,
        );
        ColorImage::set_rgba(
            img.pin_mut(),
            c_int((idx % size[0]) as i32),
            c_int((idx / size[1]) as i32),
            &color.as_mut().unwrap(),
        );
    }

    img
}
