use crate::unblending::blend_mode::BlendMode;
use crate::unblending::color_image::{ColorChannel, ColorImage};
use crate::unblending::color_model::ColorModel;
use crate::unblending::common::{iter_xy, Mat1X, Mat3X, Mat4X, Scalar, Vec3};
use crate::unblending::comp_op::CompOp;
use crate::unblending::equations::{
    calculate_constraint_vector, calculate_constraint_vector_wrapped,
    calculate_derivative_of_constraint_vector, calculate_derivative_of_unmixing_energy,
    calculate_lagrange_term, calculate_penalty_term, calculate_unmixing_energy_term,
};
use crate::unblending::image_processing::apply_guided_filter;
use crate::unblending::layer_info::{
    extract_blend_modes, extract_color_models, extract_comp_ops, LayerInfo,
};
use crate::unblending::optimization_parameter_set::OptimizationParameterSet;
use crate::unblending::parallel::parallel_for_2d;
use crate::unblending::{equations, nlopt_util};
use itertools::Itertools;
use nalgebra::SimdPartialOrd;
use std::iter::repeat_with;
use std::num::NonZero;
use std::sync::{Arc, Mutex};
use std::thread;
use tracing::{debug, instrument};

//(&[f64], Option<&mut [f64]>, T) -> f64
fn objective_function<T: ColorModel>(
    x: &[Scalar],
    grad: Option<&mut [Scalar]>,
    set: &mut OptimizationParameterSet<T>,
) -> Scalar {
    let x = Mat4X::from_iterator(x.len() / 4, x.iter().cloned());
    let alphas = x.fixed_rows::<1>(3);
    let colors = x.fixed_rows::<3>(0);

    let constraint_vector = calculate_constraint_vector(
        &alphas.into(),
        &colors.into(),
        &set.target_color,
        &set.comp_ops,
        &set.modes,
        set.use_target_alphas,
        &set.target_alphas,
        &set.gray_layers,
    );

    let derivative_of_unmixing_energy = calculate_derivative_of_unmixing_energy(
        &alphas.into(),
        &colors.into(),
        &set.models,
        set.sigma,
        set.use_sparcity,
        set.use_minimum_alpha,
    );
    let derivative_of_constraint_vector = calculate_derivative_of_constraint_vector(
        &alphas.into(),
        &colors.into(),
        &set.target_color,
        &set.comp_ops,
        &set.modes,
        set.use_target_alphas,
        &set.gray_layers,
    );

    let gradient = derivative_of_unmixing_energy
        + derivative_of_constraint_vector * (set.rho * &constraint_vector - &set.lambda);

    // Eigen::Map<VecX>(&grad[0], num_layers * 4) = gradient;
    grad.unwrap().copy_from_slice(&gradient.as_slice());

    let unmixing_energy = calculate_unmixing_energy_term(
        &alphas.into(),
        &colors.into(),
        &set.models,
        set.sigma,
        set.use_sparcity,
        set.use_minimum_alpha,
    );
    let lagrange = calculate_lagrange_term(&constraint_vector, &set.lambda);
    let penalty = calculate_penalty_term(&constraint_vector, set.rho);

    return unmixing_energy + lagrange + penalty;
}

fn find_initial_solution<T: ColorModel>(models: &Vec<&T>) -> Mat4X {
    let num_layers = models.len();

    let x_initial = Mat4X::from_columns(
        &(0..num_layers)
            .map(|index| models[index].get_representative_color().push(0.5))
            .collect_vec(),
    );

    x_initial
}

fn solve_per_pixel_optimization<T: ColorModel + Clone>(
    set: &mut OptimizationParameterSet<T>,
    has_opaque_background: Option<bool>,
    initial_colors: Option<&Mat1X>,
    force_smooth_background: Option<bool>,
    target_background_color: Option<&Vec3>,
) -> Mat4X {
    let has_opaque_background = has_opaque_background.unwrap_or(true);
    let empty_mat = Mat1X::zeros(0);
    let initial_colors = initial_colors.unwrap_or(&empty_mat);
    let force_smooth_background = force_smooth_background.unwrap_or(false);
    let empty_vec = Vec3::zeros();
    let target_background_color = target_background_color.unwrap_or(&empty_vec);

    let is_for_refinement = set.use_target_alphas;

    let num_layers = set.models.len();

    let mut upper = Mat1X::from_element(num_layers * 4, 1.0);
    let mut lower = Mat1X::from_element(num_layers * 4, 0.0);

    // Find an initial solution
    let mut x = find_initial_solution(&set.models);
    if is_for_refinement {
        x.rows_mut(0, num_layers).copy_from(&set.target_alphas);
        x.rows_mut(num_layers, num_layers * 3)
            .copy_from(initial_colors);
    }

    // Enforce background opacity
    if has_opaque_background {
        lower[0] = 1.0;
        x[0] = 1.0;
    }

    // Enforce background smoothness
    if force_smooth_background {
        assert!(has_opaque_background);

        upper
            .rows_mut(num_layers, 3)
            .copy_from(target_background_color);
        lower
            .rows_mut(num_layers, 3)
            .copy_from(target_background_color);
        x.rows_mut(num_layers, 3).copy_from(target_background_color);
    }

    let gamma = 0.25;
    let epsilon = 5e-03;
    let local_epsilon = 5e-03;
    let beta = 10.0;

    let mut count = 0;
    let max_count = 20;
    loop {
        let x_new = Mat4X::from_iterator(
            num_layers,
            nlopt_util::solve::<OptimizationParameterSet<T>>(
                x.data.as_slice(),
                upper.as_slice(),
                lower.as_slice(),
                &objective_function,
                None,
                None,
                Some(nlopt::Algorithm::Lbfgs),
                set,
                Some(false),
                Some(1000),
                Some(local_epsilon),
                Some(local_epsilon),
                None,
                None,
            )
            .iter()
            .cloned(),
        );
        let g = calculate_constraint_vector_wrapped(
            &x,
            &set.target_color,
            &set.comp_ops,
            &set.modes,
            set.use_target_alphas,
            &set.target_alphas,
            &set.gray_layers,
        );
        let g_new = calculate_constraint_vector_wrapped(
            &x_new,
            &set.target_color,
            &set.comp_ops,
            &set.modes,
            set.use_target_alphas,
            &set.target_alphas,
            &set.gray_layers,
        );

        set.lambda -= set.rho * &g_new;
        if g_new.norm() > gamma * g.norm() {
            set.rho *= beta;
        }

        let is_unchanged = (&x_new - &x).norm() < epsilon;
        let is_satisfied = g_new.norm() < epsilon;

        x = x_new;

        if (is_unchanged && is_satisfied) || count > max_count {
            break;
        };

        count += 1;
    }

    x
}

fn normalize_alphas(alphas: &Mat1X, comp_ops: &Vec<CompOp>) -> Mat1X {
    let mut is_all_plus = true;
    let mut is_all_source_over = true;
    comp_ops.iter().for_each(|comp_op| {
        is_all_plus = is_all_plus && comp_op.is_plus();
        is_all_source_over = is_all_source_over && comp_op.is_source_over();
    });

    assert!(is_all_plus || is_all_source_over);

    let epsilon = 1e-05;

    let has_opaque_background = (alphas[0] - 1.0).abs() < epsilon;

    if is_all_source_over && has_opaque_background {
        return alphas.clone();
    }
    if is_all_plus {
        return alphas / alphas.sum();
    }

    // This line is never performed
    return Mat1X::zeros(alphas.nrows());
}

pub fn perform_matte_refinement<T: ColorModel + Clone>(
    image: &ColorImage,
    layers: &Vec<&ColorImage>,
    layer_infos: &Vec<&LayerInfo<T>>,
    has_opaque_background: bool,
    force_smooth_background: bool,
    target_concurrency: Option<usize>,
) -> Vec<ColorImage> {
    // let timer = ("perform_matte_refinement");

    let models = extract_color_models(layer_infos);
    let comp_ops = extract_comp_ops(layer_infos);
    let modes = extract_blend_modes(layer_infos);

    assert_eq!(layers.len(), models.len());

    let number = layers.len();
    let width = image.width;
    let height = image.height;
    let radius = NonZero::new((60 * width.min(height) / 1000) as u32).unwrap();

    let epsilon = 1e-04;

    // Apply guided filter
    let mut refined_alphas = vec![];
    layers.iter().for_each(|layer| {
        let alpha = layer.get_channel(ColorChannel::Alpha);
        let refined_alpha = apply_guided_filter(alpha, image, radius, epsilon);

        refined_alphas.push(refined_alpha);
    });

    // Clamp alphas into [0, 1]
    iter_xy(0..width, 0..height, |x, y| {
        (0..number).for_each(|i| {
            refined_alphas[i][(x, y)] = (refined_alphas[i][(x, y)]).clamp(0.0, 1.0);
        });
    });

    iter_xy(0..width, 0..height, |x, y| {
        let mut alphas = Mat1X::zeros(number);
        (0..number).for_each(|i| {
            alphas[i] = refined_alphas[i][(x, y)];
        });
        alphas = normalize_alphas(&alphas, &comp_ops);

        (0..number).for_each(|i| {
            refined_alphas[i][(x, y)] = alphas[i];
        });
    });
    // Normalize alphas such that the composited alpha becomes one for each pixel
    // Smooth background
    let mut smoothed_background = ColorImage::from_scalar(width, height, 0.0);
    if force_smooth_background {
        assert!(has_opaque_background);
        smoothed_background.set_channel(
            ColorChannel::Red,
            apply_guided_filter(
                layers[0].get_channel(ColorChannel::Red),
                image,
                radius,
                epsilon,
            ),
        );
        smoothed_background.set_channel(
            ColorChannel::Green,
            apply_guided_filter(
                layers[0].get_channel(ColorChannel::Green),
                image,
                radius,
                epsilon,
            ),
        );
        smoothed_background.set_channel(
            ColorChannel::Blue,
            apply_guided_filter(
                layers[0].get_channel(ColorChannel::Blue),
                image,
                radius,
                epsilon,
            ),
        );
    }

    let refined_layers = Arc::new(Mutex::<Vec<ColorImage>>::new(vec![]));
    let refined_layers_clone = Arc::clone(&refined_layers);
    thread::scope(|scope| {
        // Perform optimization
        let per_pixel_process = move |x, y| {
            let mut initial_colors = Mat1X::zeros(number * 3);
            let mut target_alphas = Mat1X::zeros(number);
            (0..number).for_each(|i| {
                initial_colors
                    .fixed_rows_mut::<3>(i * 3)
                    .copy_from(&layers[i].get_rgba(x, y).xyz());
                target_alphas[i] = refined_alphas[i][(x, y)];
            });

            if force_smooth_background {
                initial_colors.rows_mut(0, 3).copy_from(
                    &smoothed_background
                        .get_rgba(x, y)
                        .xyz()
                        .simd_clamp(Vec3::zeros(), Vec3::from_element(1.0)),
                );
            }

            let pixel_color = (&image).get_rgba(x, y).xyz();
            let mut set = OptimizationParameterSet::new(
                pixel_color,
                models.clone(),
                comp_ops.clone(),
                modes.clone(),
                Some(true),
                Some(target_alphas),
            );
            let solution = solve_per_pixel_optimization(
                &mut set,
                Some(has_opaque_background),
                Some(&initial_colors),
                Some(force_smooth_background),
                Some(
                    &smoothed_background
                        .get_rgba(x, y)
                        .xyz()
                        .simd_clamp(Vec3::zeros(), Vec3::from_element(1.0)),
                ),
            );

            (0..number).for_each(|index| {
                (&refined_layers_clone).lock().unwrap()[index].set_rgba(
                    x,
                    y,
                    &solution.fixed_columns::<1>(index).into(),
                );
            });
        };
        parallel_for_2d(width, height, target_concurrency, scope, per_pixel_process);
    });
    Arc::into_inner(refined_layers)
        .unwrap()
        .into_inner()
        .unwrap()
}

#[instrument]
pub fn compute_color_unmixing<T: ColorModel + Clone>(
    image: &ColorImage,
    layer_infos: &Vec<&LayerInfo<T>>,
    has_opaque_background: bool,
    target_concurrency: Option<usize>,
) -> Vec<ColorImage> {
    debug!(?layer_infos, "Beginning color unmixing");
    // timer::Timer timer("compute_color_unmixing");

    let models = extract_color_models(layer_infos);
    let comp_ops = extract_comp_ops(layer_infos);
    let modes = extract_blend_modes(layer_infos);

    let width = image.width;
    let height = image.height;
    let num_layers = models.len();

    let mut layers = Vec::from_iter(
        repeat_with(|| ColorImage::from_scalar(width, height, 0.0)).take(num_layers),
    );

    // let (start_process_tx, start_process_rx) = mpmc::channel();
    // let (finish_process_tx, finish_process_rx) = mpsc::channel();

    let per_pixel_process = |x, y| {
        let pixel_color = image.get_rgba(x, y).xyz();
        let mut set = OptimizationParameterSet::new(
            pixel_color,
            models.clone(),
            comp_ops.clone(),
            modes.clone(),
            Some(false),
            None,
        );
        let solution =
            solve_per_pixel_optimization(&mut set, Some(has_opaque_background), None, None, None);

        (0..num_layers).for_each(|index| {
            layers[index].set_rgba(x, y, &solution.fixed_columns::<1>(index).into());
        });
    };

    debug!("Beginning per-pixel process");
    thread::scope(|scope| {
        parallel_for_2d(width, height, target_concurrency, scope, per_pixel_process);
    });

    debug!("Per-pixel process complete");
    return layers;
}

fn composite_layers(
    layers: &Vec<&ColorImage>,
    comp_ops: &Vec<CompOp>,
    modes: &Vec<BlendMode>,
) -> ColorImage {
    let number = layers.len();
    let width = layers.first().unwrap().width;
    let height = layers.first().unwrap().height;

    let mut composited_image = ColorImage::from_scalar(width, height, 0.0);

    (0..width).for_each(|x| {
        (0..height).for_each(|y| {
            let mut alphas = Mat1X::zeros(number);
            let mut colors = Mat3X::zeros(number);
            (0..number).for_each(|index| {
                alphas[index] = layers[index].get_rgba(x, y)[3];
                colors
                    .fixed_rows_mut::<3>(index * 3)
                    .copy_from(&layers[index].get_rgba(x, y).xyz());
            });
            let composited_color =
                equations::composite_layers(&alphas, &colors, comp_ops, modes, false);
            composited_image.set_rgba(x, y, &composited_color);
        })
    });

    composited_image
}
