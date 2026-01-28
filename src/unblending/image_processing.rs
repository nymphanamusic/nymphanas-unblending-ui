use crate::unblending::color_image::{ColorChannel, ColorImage};
use crate::unblending::common::{iter_xy, Mat3, MatX, Scalar, Vec3};
use crate::unblending::parallel::parallel_for_2d;
use std::cell::RefCell;
use std::num::NonZero;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn apply_box_filter(image: &MatX, radius: NonZero<u32>) -> MatX {
    let size = (2 * radius.get() + 1) as usize;
    let kernel = MatX::from_element(size, size, 1.0 / (size * size) as Scalar);
    apply_convolution(image, &kernel, None)
}

pub fn apply_convolution<'a>(
    image: &MatX,
    kernel: &MatX,
    target_concurrency: Option<usize>,
) -> MatX {
    let w = image.ncols();
    let h = image.nrows();

    let kernel_size = kernel.nrows();

    assert_eq!(kernel_size % 2, 1);
    assert_eq!(kernel_size, kernel.ncols());

    let new_image = Arc::new(Mutex::new(MatX::zeros(image.nrows(), image.ncols())));
    let new_image_clone = new_image.clone();

    let process = move |x, y| {
        let mut value = 0.0;
        iter_xy(0..kernel_size, 0..kernel_size, |kernel_x, kernel_y| {
            let original_image_x =
                ((x + kernel_x - ((kernel_size - 1) / 2)) as usize).clamp(0, w - 1);
            let original_image_y =
                ((y + kernel_y - ((kernel_size - 1) / 2)) as usize).clamp(0, h - 1);

            value += kernel[(kernel_x, kernel_y)] * image[(original_image_x, original_image_y)];
        });
        new_image_clone.lock().unwrap()[(x, y)] = value;
    };

    thread::scope(|scope| {
        parallel_for_2d(w, h, target_concurrency, scope, process);
    });

    // TODO I think this is a full clone which isn't great
    new_image.lock().unwrap().clone()
}

pub fn apply_guided_filter(
    input_image: MatX,
    guidance_image: &ColorImage,
    radius: NonZero<u32>,
    epsilon: Scalar,
) -> MatX {
    let width = guidance_image.width;
    let height = guidance_image.height;

    let I_r = guidance_image.get_channel(ColorChannel::Red);
    let I_g = guidance_image.get_channel(ColorChannel::Green);
    let I_b = guidance_image.get_channel(ColorChannel::Blue);

    let mean_I_r = apply_box_filter(&I_r, radius);
    let mean_I_g = apply_box_filter(&I_g, radius);
    let mean_I_b = apply_box_filter(&I_b, radius);

    let mean_p = apply_box_filter(&input_image, radius);

    let mean_Ip_r = apply_box_filter(&(&I_r * &input_image), radius);
    let mean_Ip_g = apply_box_filter(&(&I_g * &input_image), radius);
    let mean_Ip_b = apply_box_filter(&(&I_b * &input_image), radius);

    let cov_Ip_r = mean_Ip_r - (&mean_I_r * &mean_p);
    let cov_Ip_g = mean_Ip_g - (&mean_I_g * &mean_p);
    let cov_Ip_b = mean_Ip_b - (&mean_I_b * &mean_p);

    let var_I_rr = apply_box_filter(&(&I_r * &I_r), radius) - (&mean_I_r * &mean_I_r);
    let var_I_rg = apply_box_filter(&(&I_r * &I_g), radius) - (&mean_I_r * &mean_I_g);
    let var_I_rb = apply_box_filter(&(&I_r * &I_b), radius) - (&mean_I_r * &mean_I_b);
    let var_I_gg = apply_box_filter(&(&I_g * &I_g), radius) - (&mean_I_g * &mean_I_g);
    let var_I_gb = apply_box_filter(&(&I_g * &I_b), radius) - (&mean_I_g * &mean_I_b);
    let var_I_bb = apply_box_filter(&(&I_b * &I_b), radius) - (&mean_I_b * &mean_I_b);

    let a_r = RefCell::new(MatX::zeros(width, height));
    let a_g = RefCell::new(MatX::zeros(width, height));
    let a_b = RefCell::new(MatX::zeros(width, height));

    iter_xy(0..width, 0..height, |x, y| {
        let sigma = Mat3::new(
            var_I_rr[(x, y)],
            var_I_rg[(x, y)],
            var_I_rb[(x, y)],
            var_I_rg[(x, y)],
            var_I_gg[(x, y)],
            var_I_gb[(x, y)],
            var_I_rb[(x, y)],
            var_I_gb[(x, y)],
            var_I_bb[(x, y)],
        );

        let cov_Ip = Vec3::new(cov_Ip_r[(x, y)], cov_Ip_g[(x, y)], cov_Ip_b[(x, y)]);
        let a_xy = (sigma + epsilon * Mat3::identity()).try_inverse().unwrap() * cov_Ip;

        (&a_r).borrow_mut()[(x, y)] = a_xy[0];
        (&a_g).borrow_mut()[(x, y)] = a_xy[1];
        (&a_b).borrow_mut()[(x, y)] = a_xy[2];
    });

    let b = ((mean_p - (&*a_r.borrow() * mean_I_r)) - (&*a_g.borrow() * mean_I_g))
        - (&*a_b.borrow() * mean_I_b);

    let mut q = apply_box_filter(&b, radius);
    q = q + (apply_box_filter(&a_r.borrow(), radius) * I_r);
    q = q + (apply_box_filter(&a_g.borrow(), radius) * I_g);
    q = q + (apply_box_filter(&a_b.borrow(), radius) * I_b);

    q
}
