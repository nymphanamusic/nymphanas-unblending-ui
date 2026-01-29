use crate::unblending::common::{Mat3, Scalar, Vec4};
use egui::Color32;
use nalgebra::matrix;

const s: Scalar = 3.2;
const offset: Scalar = 1e-03;

pub fn get_sigma(variance: Scalar) -> Mat3 {
    assert!(variance >= 0.0 && variance <= 1.0);
    matrix!(
        variance.powf(s) + offset, 0.0, 0.0;
        0.0, variance.powf(s) + offset, 0.0;
        0.0, 0.0, variance.powf(s) + offset
    )
}

pub fn get_sigma_full(
    s1: Scalar,
    s2: Scalar,
    s3: Scalar,
    s4: Scalar,
    s5: Scalar,
    s6: Scalar,
) -> Mat3 {
    assert!(s1 >= 0.0 && s1 <= 1.0);
    assert!(s2 >= 0.0 && s2 <= 1.0);
    assert!(s3 >= 0.0 && s3 <= 1.0);
    assert!(s4 >= 0.0 && s4 <= 1.0);
    assert!(s5 >= 0.0 && s5 <= 1.0);
    assert!(s6 >= 0.0 && s6 <= 1.0);

    matrix!(
        s1.powf(s) + offset, 0.1 * (s4 - 0.5), 0.0;
        0.0, s2.powf(s) + offset, 0.1 * (s5 - 0.5);
        0.1 * (s6 - 0.5), 0.0, s3.powf(s) + offset
    )
}

pub fn color_from_vec4(v: &Vec4) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (v.x * 255.0) as u8,
        (v.y * 255.0) as u8,
        (v.z * 255.0) as u8,
        (v.w * 255.0) as u8,
    )
}

pub fn vec4_from_color(color: &Color32) -> Vec4 {
    Vec4::new(
        color.r() as f64 / 255.0,
        color.g() as f64 / 255.0,
        color.b() as f64 / 255.0,
        color.a() as f64 / 255.0,
    )
}
