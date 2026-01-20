#[allow(dead_code)]
use autocxx::prelude::*;
use egui::Color32;
use ffi::unblending::{AbstractImage, BlendMode, ColorImage};
use ffi::unblending_helpers::make_vector4d;
use std::pin::Pin;

#[allow(unused_imports)]
pub use ffi::unblending;
pub use ffi::unblending_helpers;
pub use ffi::Eigen;

// Manual bindings

#[cxx::bridge(namespace = "Eigen")]
pub mod ffi_manual {
    unsafe extern "C++" {
        include!("Eigen/Core");
        include!("Eigen/src/Core/Matrix.h");

        type Vector3d;
        fn x(self: &Vector3d) -> f64;
        fn y(self: &Vector3d) -> f64;
        fn z(self: &Vector3d) -> f64;
        #[Self = "Vector3d"]
        fn Zero() -> UniquePtr<Vector3d>;

        type Vector4d;
        // fn x(self: &Vector4d) -> f64;
        // fn y(self: &Vector4d) -> f64;
        // fn z(self: &Vector4d) -> f64;
        // fn w(self: &Vector4d) -> f64;
        #[Self = "Vector4d"]
        fn Zero() -> UniquePtr<Vector4d>;

        type Matrix3d;
        #[Self = "Matrix3d"]
        fn Zero() -> UniquePtr<Matrix3d>;
    }
}

// Autocxx

include_cpp! {
    #include "unblending/unblending.hpp"
    #include "unblending_helpers.hpp"
    safety!(unsafe)

    extern_cpp_opaque_type!("Eigen::Vector3d", crate::unblending_ffi::ffi_manual::Vector3d)
    extern_cpp_opaque_type!("Eigen::Vector4d", crate::unblending_ffi::ffi_manual::Vector4d)
    extern_cpp_opaque_type!("Eigen::Matrix3d", crate::unblending_ffi::ffi_manual::Matrix3d)

    generate!("unblending::BlendMode")
    generate!("unblending::AbstractImage")
    generate!("unblending::ColorImage")
    generate!("unblending::ColorModelPtr")
    generate!("unblending::CompOp")
    generate!("unblending::LayerInfo")
    generate!("unblending::compute_color_unmixing")
    generate!("unblending::perform_matte_refinement")

    // unblending_helpers.cpp
    generate!("unblending_helpers::make_matrix3d_s")
    generate!("unblending_helpers::make_matrix3d")
    generate!("unblending_helpers::make_vector3d")
    generate!("unblending_helpers::make_vector4d")
    generate!("unblending_helpers::make_layer_info")
    generate!("unblending_helpers::push_layer_info")
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
        let mut color = unblending_helpers::make_vector4d(
            color.r() as f64 / 255.0,
            color.g() as f64 / 255.0,
            color.b() as f64 / 255.0,
            color.a() as f64 / 255.0,
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

/// Source: https://github.com/google/autocxx/issues/592#issuecomment-1936306347
unsafe fn upcast<D, S>(derived: Pin<&mut D>) -> Pin<&mut S> {
    let subclass_obs_ptr = Pin::into_inner_unchecked(derived) as *mut D;
    std::pin::Pin::new_unchecked(&mut *subclass_obs_ptr.cast::<S>())
}

pub fn pixels_from_color_image(mut image: &ColorImage) -> Vec<Color32> {
    // let abstract_image;
    // unsafe {
    //     abstract_image = upcast::<ColorImage, AbstractImage>(image);
    // }
    let width: i32 = image.as_ref().width().into();
    let height: i32 = image.as_ref().height().into();

    let mut out = Vec::with_capacity((width as u16 * height as u16).into());
    for idx in 0..(width * height) {
        let x = idx % width;
        let y = ((idx as f32) / height as f32).floor() as i32;
        // let color = image.get_rgba(c_int(x), c_int(y));
        // out.push(Color32::from_rgba_premultiplied(
        //     (color.x() * 255.0).floor() as u8,
        //     (color.y() * 255.0).floor() as u8,
        //     (color.z() * 255.0).floor() as u8,
        //     (color.w() * 255.0).floor() as u8,
        // ));
    }

    out
}
