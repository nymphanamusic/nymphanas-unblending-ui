use crate::unblending::common::{Mat3X, MatX, Scalar, Vec3, Vec4};
use itertools::Itertools;
use std::fmt::{Debug, Formatter};
use std::iter::{repeat, Map};
use std::slice::Iter;

pub enum ColorChannel {
    Red = 0,
    Green = 1,
    Blue = 2,
    Alpha = 3,
}

pub struct ColorImage {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Scalar>,
}

impl Debug for ColorImage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ColorImage")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl ColorImage {
    pub const depth: usize = 4;

    pub fn from_scalar(width: usize, height: usize, scalar: Scalar) -> Self {
        let pixels = Vec::from_iter(repeat(scalar).take(width * height * Self::depth));
        ColorImage {
            width,
            height,
            pixels,
        }
    }

    // pub fn composite_layers(
    //     images: &Vec<&ColorImage>,
    //     comp_ops: &Vec<&CompOp>,
    //     modes: &Vec<BlendMode>,
    //     clamp: bool,
    // ) -> ColorImage {
    //     let num_layers = images.len();
    //     assert_eq!(num_layers, comp_ops.len());
    //     assert_eq!(num_layers, modes.len());
    //
    //     (0..num_layers-1).for_each(|i| {
    //         let composited = composite_two_layers(
    //             &colors.rows(i * 3, 3).into(),
    //             &color,
    //             alphas[i],
    //             alpha,
    //             &comp_ops[i],
    //             modes[i],
    //             clamp,
    //         );
    //         color = composited.rows(0, 3).into();
    //         alpha = composited[3];
    //     });
    //
    //     color.push(alpha)
    // }
    //
    // pub fn composite_two_layers(
    //     self: &mut Self,
    //     other: &Self,
    //     comp_op: &CompOp,
    //     mode: BlendMode,
    //     clamp: bool,
    // ) -> ColorImage {
    //     let x = comp_op.x as Scalar;
    //     let y = comp_op.y as Scalar;
    //     let z = comp_op.z as Scalar;
    //     const epsilon: Scalar = 1e-12;
    //
    //     let mut new_image = Self::from_scalar(self.width, self.height, 0.0);
    //     iter_xy(0..self.width, 0..self.height, |x, y| {
    //         let s = self.get_rgba(x, y);
    //         let d = other.get_rgba(x, y);
    //         let c_s: Vec3 = s.xyz();
    //         let c_d: Vec3 = d.xyz();
    //         let a_s = s.w;
    //         let a_d = d.w;
    //
    //         // let a = x * &a_s * &a_d + y * &a_s * (1.0 - &a_d) + z * &a_d * (1.0 - &a_s);
    //         let mut a = (x as f64 * a_s * a_d) + (y as f64 * a_s * (1.0 - a_d)) + (z * a_d * (1.0 - a_s));
    //         let f = mode.blend_vec(&c_s, &c_d, None);
    //         let c_pre = (f * a_s * a_d) + (y as f64 * a_s * (1.0 - a_d) * c_s) + (z * a_d * (1.0 - a_s) * c_d);
    //         let mut c = if a > epsilon { c_pre / a } else { c_pre };
    //
    //         assert!(!c.sum().is_nan());
    //
    //
    //         if clamp {
    //             c = c.simd_clamp(zero(), Vec3::from_element(1.0));
    //             a = a.clamp(0.0, 1.0);
    //         }
    //         new_image.set_rgba(x, y, &c, a);
    //     });
    //
    //     new_image
    // }

    pub fn get_colors(self: &Self) -> Mat3X {
        let pixels = &self.pixels;
        let vec = pixels
            .iter()
            .chunks(4)
            .into_iter()
            .map(|i| i.cloned().take(3).collect::<Vec<Scalar>>())
            .flatten()
            .collect::<Vec<Scalar>>();
        Mat3X::from_row_iterator(self.width * self.height, vec)
    }

    pub fn get_rgba(self: &Self, x: usize, y: usize) -> Vec4 {
        let pos = (y * self.width + x) * Self::depth;
        // Vec4::from_vec(self.pixels.iter().skip(pos).take(4).cloned().collect::<Vec<Scalar>>())
        Vec4::new(
            self.pixels[pos],
            self.pixels[pos + 1],
            self.pixels[pos + 2],
            self.pixels[pos + 3],
        )
    }

    pub fn set_rgba(self: &mut Self, x: usize, y: usize, color: &Vec4) {
        let pos = (y * self.width + x) * Self::depth;
        self.pixels[pos] = color.x;
        self.pixels[pos + 1] = color.y;
        self.pixels[pos + 2] = color.z;
        self.pixels[pos + 3] = color.w;
    }

    pub fn set_rgb_a(self: &mut Self, x: usize, y: usize, color: &Vec3, alpha: Scalar) {
        self.set_rgba(x, y, &color.push(alpha))
    }

    pub fn get_channel(self: &Self, channel: ColorChannel) -> MatX {
        MatX::from_row_iterator(
            self.height,
            self.width,
            self.pixels
                .iter()
                .skip(channel as usize)
                .step_by(4)
                .map(|x| x.clone()),
        )
    }

    pub fn set_channel(self: &mut Self, channel: ColorChannel, pixels: MatX) {
        self.pixels
            .iter_mut()
            .skip(channel as usize)
            .step_by(4)
            .zip(pixels.iter())
            .for_each(|(this_pixel, other_pixel)| {
                *this_pixel = other_pixel.clone();
            });
    }

    pub fn iter_rgba(self: &'_ Self) -> Map<Iter<'_, [Scalar; 4]>, fn(&[Scalar; 4]) -> Vec4> {
        self.pixels
            .as_chunks::<4>()
            .0
            .iter()
            .map(|[x, y, z, w]| Vec4::new(*x, *y, *z, *w))
    }
}
