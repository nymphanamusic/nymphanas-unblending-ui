use crate::unblending::color_image::ColorImage;
use crate::unblending::common::{Mat2, Mat3, Scalar, Vec2, Vec3};
use nalgebra::{SimdPartialOrd, SymmetricEigen};
use std::cell::RefCell;
use std::f64::consts::{FRAC_PI_2, PI};
use std::fmt::Debug;
use std::ops::MulAssign;

pub trait ColorModel: Sync + Debug {
    fn calculate_distance(self: &Self, color: &Vec3) -> Scalar;

    fn calculate_distance_gradient(self: &Self, color: &Vec3) -> Vec3;

    fn get_representative_color(self: &Self) -> Vec3;

    fn generate_visualization(self: &Self) -> Option<ColorImage>;
}

#[derive(Clone, Debug)]
pub struct GaussianColorModel {
    pub mu: Vec3,
    pub sigma_inv: Mat3,
}

impl GaussianColorModel {
    fn get_sigma(self: &Self) -> Mat3 {
        self.sigma_inv.try_inverse().unwrap()
    }
    fn set_sigma(self: &mut Self, sigma: &Mat3) {
        self.sigma_inv = sigma.try_inverse().unwrap();
    }
}

impl ColorModel for GaussianColorModel {
    fn calculate_distance(self: &Self, color: &Vec3) -> Scalar {
        ((*color - self.mu).transpose() * self.sigma_inv * (*color - self.mu)).to_scalar()
    }

    fn calculate_distance_gradient(self: &Self, color: &Vec3) -> Vec3 {
        2.0 * self.sigma_inv * (*color - self.mu)
    }

    fn get_representative_color(self: &Self) -> Vec3 {
        self.mu
    }

    fn generate_visualization(self: &Self) -> Option<ColorImage> {
        const size: usize = 480;

        let sigma = self.sigma_inv.try_inverse().unwrap();
        let eigen_solver = SymmetricEigen::new(sigma);
        let v = RefCell::new(eigen_solver.eigenvectors);
        let a = RefCell::new(eigen_solver.eigenvalues);

        let swap_cols = |i, j| {
            let binding = v.borrow_mut();
            let v_temp = binding.column(i);
            v.borrow_mut().set_column(i, &(v.borrow_mut().column(j)));
            v.borrow_mut().set_column(j, &v_temp);
            let a_temp = a.borrow()[i];
            a.borrow_mut()[i] = a.borrow_mut()[j];
            a.borrow_mut()[j] = a_temp;
        };

        // Generate many candidates
        let mut candidates: Vec<(Scalar, Mat3, Vec3)> = Vec::new();
        let mut evaluate_and_push = || {
            candidates.push((
                evaluate_visual_clutter(&v.borrow(), &a.borrow(), &self.mu),
                v.borrow().clone(),
                a.borrow().clone(),
            ))
        };
        let mut make_candidates = || {
            evaluate_and_push();
            (0..=2).for_each(|col| {
                v.borrow_mut().column_mut(col).mul_assign(-1.0);
                evaluate_and_push();
                v.borrow_mut().column_mut(col).mul_assign(-1.0);
            });
        };

        make_candidates();
        swap_cols(0, 1);
        make_candidates();

        // Pick up the least visually clutter one
        let (_, cleanest_v, cleanest_a) = candidates
            .iter()
            .min_by(|(error_a, _, _), (error_b, _, _)| error_a.total_cmp(error_b))?;

        Some(generate_unsorted_visualization(
            &cleanest_v,
            &cleanest_a,
            &self.mu,
            size,
        ))
    }
}

fn evaluate_visual_clutter(V: &Mat3, A: &Vec3, mu: &Vec3) -> Scalar {
    let make = |col: usize, coeff: Scalar| {
        (coeff * V.column(col) * A[col] + mu)
            .simd_clamp(Vec3::from_element(0.0), Vec3::from_element(1.0))
    };

    let v0 = make(0, 1.0);
    [
        make(1, 1.0),
        make(2, 1.0),
        make(0, -1.0),
        make(1, -1.0),
        make(2, -1.0),
        v0,
    ]
    .iter()
    // (v0 - v1).norm() + ... + (v5 - v0).norm
    .fold((0.0, v0), |(acc, prev), this| {
        (acc + (prev - this).norm(), *this)
    })
    .0
}

fn generate_unsorted_visualization(v: &Mat3, a: &Vec3, mu: &Vec3, size: usize) -> ColorImage {
    let mut image = ColorImage::from_scalar(size, size, 0.0);

    (0..size).for_each(|x_screen| {
        (0..size).for_each(|y_screen| {
            let x = (2.0 * x_screen as Scalar) / (size as Scalar - 1.0) - 1.0;
            let y = 1.0 - (2.0 * y_screen as Scalar) / (size as Scalar - 1.0);
            let t = if x.abs() > 1e-05 {
                (y / x).atan()
            } else {
                (y / y.abs()) * FRAC_PI_2
            };

            let b_1: Vec3;
            let b_2: Vec3;
            let s_1: Scalar;
            let s_2: Scalar;
            let r: Mat2;

            if t > PI / 6.0 {
                b_1 = v.column(0).into();
                b_2 = v.column(1).into();
                s_1 = a.x.max(0.0).sqrt();
                s_2 = a.y.max(0.0).sqrt();
                r = Mat2::from_columns(&[
                    Vec2::new((PI / 2.0).cos(), (PI / 2.0).sin()),
                    Vec2::new((PI / 6.0).cos(), (PI / 6.0).sin()),
                ])
            } else if t > -PI / 6.0 {
                b_1 = v.column(1).into();
                b_2 = v.column(2).into();
                s_1 = a.y.max(0.0).sqrt();
                s_2 = a.z.max(0.0).sqrt();
                r = Mat2::from_columns(&[
                    Vec2::new((PI / 6.0).cos(), (PI / 6.0).sin()),
                    Vec2::new((-PI / 6.0).cos(), (-PI / 6.0).sin()),
                ]);
            } else {
                b_1 = v.column(2).into();
                b_2 = v.column(0).into();
                s_1 = a.z.max(0.0).sqrt();
                s_2 = a.x.max(0.0).sqrt();
                r = Mat2::from_columns(&[
                    Vec2::new((-PI / 6.0).cos(), (-PI / 6.0).sin()),
                    Vec2::new((PI / 2.0).cos(), (PI / 2.0).sin()),
                ]);
            }

            let w = r.try_inverse().unwrap() * Vec2::new(x, y);

            // If the pixel is outside the hexagon, leave it transparent
            if w.x.abs() + w.y.abs() > 1.0 {
                return;
            }

            let color = (mu + w.x * s_1 * b_1 + w.y * s_2 * b_2)
                .simd_clamp(Vec3::from_element(0.0), Vec3::from_element(1.0));
            image.set_rgb_a(x_screen, y_screen, &color, 1.0);
        })
    });

    image
}
