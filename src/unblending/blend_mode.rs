use crate::unblending::common::Vec3;
use strum_macros::{EnumCount, EnumIter, IntoStaticStr};

#[derive(Copy, Clone, PartialEq, IntoStaticStr, EnumCount, EnumIter)]
pub enum BlendMode {
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

impl BlendMode {
    const blend_function_internal_epsilon: f64 = 1e-05;

    pub fn blend_grad_s(self: &Self, s: f64, d: f64, crop: Option<bool>) -> f64 {
        let crop = crop.unwrap_or(false);
        match self {
            BlendMode::Normal => 1.0,
            BlendMode::Multiply => d,
            BlendMode::Screen => 1.0 - d,
            BlendMode::Overlay => {
                if d <= 0.5 {
                    2.0 * d
                } else {
                    2.0 * (1.0 - d)
                }
            }
            BlendMode::Darken => {
                if s < d {
                    1.0
                } else {
                    0.0
                }
            }
            BlendMode::Lighten => {
                if s < d {
                    0.0
                } else {
                    1.0
                }
            }
            BlendMode::ColorDodge => {
                if d < BlendMode::blend_function_internal_epsilon {
                    0.0
                } else if 1.0 - s < BlendMode::blend_function_internal_epsilon {
                    0.0
                } else if 1.0 < d / (1.0 - s) {
                    0.0
                } else {
                    d / ((1.0 - s) * (1.0 - s))
                }
            }
            BlendMode::ColorBurn => {
                if 1.0 - d < BlendMode::blend_function_internal_epsilon {
                    0.0
                } else if s < BlendMode::blend_function_internal_epsilon {
                    0.0
                } else {
                    -(if 1.0 < (1.0 - d) / s {
                        0.0
                    } else {
                        -(1.0 - d) / (s * s)
                    })
                }
            }
            BlendMode::HardLight => {
                if s <= 0.5 {
                    2.0 * d
                } else {
                    2.0 * (1.0 - d)
                }
            }
            BlendMode::SoftLight => {
                if s <= 0.5 {
                    2.0 * d * (1.0 - d)
                } else {
                    2.0 * ((if d <= 0.25 {
                        ((16.0 * d - 12.0) * d + 4.0) * d
                    } else {
                        d.sqrt()
                    }) - d)
                }
            }
            BlendMode::Difference => {
                if s < d {
                    -1.0
                } else {
                    1.0
                }
            }
            BlendMode::Exclusion => 1.0 - 2.0 * d,
            BlendMode::LinearDodge => {
                if crop && s + d > 1.0 {
                    0.0
                } else {
                    1.0
                }
            }
        }
    }

    pub fn blend_grad_d(self: &Self, s: f64, d: f64, crop: Option<bool>) -> f64 {
        let crop = crop.unwrap_or(false);
        match self {
            BlendMode::Normal => 0.0,
            BlendMode::Multiply => s,
            BlendMode::Screen => 1.0 - s,
            BlendMode::Overlay => {
                if d <= 0.5 {
                    2.0 * s
                } else {
                    2.0 * (1.0 - s)
                }
            }
            BlendMode::Darken => {
                if s < d {
                    0.0
                } else {
                    1.0
                }
            }
            BlendMode::Lighten => {
                if s < d {
                    1.0
                } else {
                    0.0
                }
            }
            BlendMode::ColorDodge => {
                if d < BlendMode::blend_function_internal_epsilon {
                    0.0
                } else if 1.0 - s < BlendMode::blend_function_internal_epsilon {
                    0.0
                } else if 1.0 < d / (1.0 - s) {
                    0.0
                } else {
                    1.0 / (1.0 - s)
                }
            }
            BlendMode::ColorBurn => {
                if 1.0 - d < BlendMode::blend_function_internal_epsilon {
                    0.0
                } else if s < BlendMode::blend_function_internal_epsilon {
                    0.0
                } else {
                    -(if 1.0 < (1.0 - d) / s { 0.0 } else { -1.0 / s })
                }
            }
            BlendMode::HardLight => {
                if s <= 0.5 {
                    2.0 * s
                } else {
                    2.0 * (1.0 - s)
                }
            }
            BlendMode::SoftLight => {
                if s <= 0.5 {
                    2.0 * s + 2.0 * d - 4.0 * s * d
                } else {
                    1.0 + (2.0 * s - 1.0)
                        * (if d <= 0.25 {
                            48.0 * d * d - 24.0 * d + 4.0
                        } else {
                            1.0 / (2.0 * d.sqrt())
                        })
                        - (2.0 * s - 1.0)
                }
            }
            BlendMode::Difference => {
                if s < d {
                    1.0
                } else {
                    -1.0
                }
            }
            BlendMode::Exclusion => 1.0 - 2.0 * s,
            BlendMode::LinearDodge => {
                if crop && s + d > 1.0 {
                    0.0
                } else {
                    1.0
                }
            }
        }
    }

    pub fn blend(self: &Self, s: f64, d: f64, crop: Option<bool>) -> f64 {
        let crop = crop.unwrap_or(false);
        return match self {
            BlendMode::Normal => s,
            BlendMode::Multiply => s * d,
            BlendMode::Screen => 1.0 - (1.0 - s) * (1.0 - d),
            BlendMode::Overlay => {
                if d <= 0.5 {
                    2.0 * s * d
                } else {
                    1.0 - 2.0 * (1.0 - s) * (1.0 - d)
                }
            }
            BlendMode::Darken => {
                if s < d {
                    s
                } else {
                    d
                }
            }
            BlendMode::Lighten => {
                if s < d {
                    d
                } else {
                    s
                }
            }
            BlendMode::ColorDodge => {
                if d < BlendMode::blend_function_internal_epsilon {
                    0.0
                } else if 1.0 - s < BlendMode::blend_function_internal_epsilon {
                    1.0
                } else {
                    (d / (1.0 - s)).min(1.0)
                }
            }
            BlendMode::ColorBurn => {
                if 1.0 - d < BlendMode::blend_function_internal_epsilon {
                    1.0
                } else if s < BlendMode::blend_function_internal_epsilon {
                    0.0
                } else {
                    1.0 - ((1.0 - d) / s).min(1.0)
                }
            }
            BlendMode::HardLight => {
                if s <= 0.5 {
                    2.0 * s * d
                } else {
                    1.0 - 2.0 * (1.0 - s) * (1.0 - d)
                }
            }
            BlendMode::SoftLight => {
                if s <= 0.5 {
                    d - (1.0 - 2.0 * s) * d * (1.0 - d)
                } else {
                    d + (2.0 * s - 1.0)
                        * ((if d <= 0.25 {
                            ((16.0 * d - 12.0) * d + 4.0) * d
                        } else {
                            d.sqrt()
                        }) - d)
                }
            }
            BlendMode::Difference => {
                if s < d {
                    d - s
                } else {
                    s - d
                }
            }
            BlendMode::Exclusion => s + d - 2.0 * s * d,
            BlendMode::LinearDodge => {
                if crop && s + d > 1.0 {
                    1.0
                } else {
                    s + d
                }
            }
        };
    }

    pub fn blend_vec(self: &Self, s: &Vec3, d: &Vec3, crop: Option<bool>) -> Vec3 {
        // Assuming separable blend functions
        return Vec3::new(
            self.blend(s.x, d.x, crop),
            self.blend(s.y, d.y, crop),
            self.blend(s.z, d.z, crop),
        );
    }
}
