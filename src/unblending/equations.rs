use crate::unblending::blend_mode::BlendMode;
use crate::unblending::color_model::ColorModel;
use crate::unblending::common::{Mat1X, Mat3, Mat3X, Mat4, Mat4X, MatX, Scalar, Vec3, Vec4};
use crate::unblending::comp_op::CompOp;
use nalgebra::{ComplexField, SimdPartialOrd};

pub fn composite_two_layers(
    c_s: &Vec3,
    c_d: &Vec3,
    a_s: Scalar,
    a_d: Scalar,
    comp_op: &CompOp,
    mode: BlendMode,
    crop: bool,
) -> Vec4 {
    let X = comp_op.x as Scalar;
    let Y = comp_op.y as Scalar;
    let Z = comp_op.z as Scalar;

    let epsilon = 1e-12;

    let a = X * a_s * a_d + Y * a_s * (1.0 - a_d) + Z * a_d * (1.0 - a_s);
    let f = mode.blend_vec(&c_s, &c_d, Some(crop));
    let c_pre = f * a_s * a_d + Y * a_s * (1.0 - a_d) * c_s + Z * a_d * (1.0 - a_s) * c_d;
    let c = if a > epsilon { c_pre / a } else { c_pre };

    assert!(!c.sum().is_nan());

    return if crop {
        c.push(a).simd_clamp(Vec4::zeros(), Vec4::from_element(1.0))
    } else {
        c.push(a)
    };
}

pub fn composite_layers(
    alphas: &Mat1X,
    colors: &Mat3X,
    comp_ops: &Vec<CompOp>,
    modes: &Vec<BlendMode>,
    crop: bool,
) -> Vec4 {
    let num_layers = alphas.ncols();

    assert_eq!(num_layers, comp_ops.len());
    assert_eq!(num_layers, modes.len());

    let mut color: Vec3 = colors.fixed_columns::<1>(0).into();
    let mut alpha = alphas[0];

    (1..num_layers).for_each(|index| {
        let x = composite_two_layers(
            &colors.fixed_columns::<1>(index).into(),
            &color.into(),
            alphas[index],
            alpha,
            &comp_ops[index],
            modes[index],
            crop,
        );
        color = x.xyz();
        alpha = x[3];
    });

    return color.push(alpha);
}

pub fn calculate_lagrange_term(constraint_vector: &Mat1X, lambda: &Mat1X) -> Scalar {
    (-lambda).dot(constraint_vector)
}

pub fn calculate_penalty_term(constraint_vector: &Mat1X, rho: Scalar) -> Scalar {
    0.5 * rho * constraint_vector.norm_squared()
}

pub fn calculate_unmixing_energy_term<T: ColorModel>(
    alphas: &Mat1X,
    colors: &Mat3X,
    models: &Vec<&T>,
    sigma: Scalar,
    use_sparcity: bool,
    use_minimum_alpha: bool,
) -> Scalar {
    let number_of_layers = alphas.len();

    // Main term
    let mut energy = 0.0;
    (0..number_of_layers).for_each(|i| {
        energy += alphas[i] * models[i].calculate_distance(&colors.fixed_columns::<1>(i).into());
    });

    // Sparcity term
    if use_sparcity {
        energy += sigma * ((alphas.sum() / alphas.norm_squared()) - 1.0);
    }

    // Minimum alpha term
    if use_minimum_alpha {
        let epsilon = 0.01;
        energy += epsilon * alphas.sum();
    }

    energy
}

pub fn calculate_constraint_vector(
    alphas: &Mat1X,
    colors: &Mat3X,
    target_color: &Vec3,
    comp_ops: &Vec<CompOp>,
    modes: &Vec<BlendMode>,
    use_target_alphas: bool,
    target_alphas: &Mat1X,
    gray_layers: &Vec<usize>,
) -> Mat1X {
    let composited_color = composite_layers(alphas, colors, comp_ops, &modes, false);
    let g_color: Vec3 = composited_color.xyz() - target_color;

    let num_layers = alphas.len();
    let num_gray_layers = gray_layers.len();
    let num_alpha_constraints = if use_target_alphas { num_layers } else { 1 };

    let mut constraints = Mat1X::from_element(3 + num_alpha_constraints + 3 * num_gray_layers, 0.0);
    constraints
        .fixed_columns_mut::<3>(0)
        .copy_from(&g_color.transpose());

    // Alpha constraints
    if use_target_alphas {
        let g_alpha = alphas - target_alphas;
        constraints
            .columns_mut(3, num_alpha_constraints)
            .copy_from(&g_alpha);
    } else {
        let g_alpha = composited_color[3] - 1.0;
        constraints[3] = g_alpha;
    }

    // Gray-scale constraints
    (0..num_gray_layers).for_each(|i| {
        let gray_layer = gray_layers[i];
        let color: Vec3 = colors.fixed_columns::<1>((gray_layer) as usize).into();
        let gray_constraint = 3.0f64.sqrt() * color - color.norm() * Vec3::from_element(1.0);

        constraints
            .fixed_columns_mut::<3>(3 + num_alpha_constraints + (3 * i))
            .copy_from(&gray_constraint.transpose());
    });

    constraints
}

pub fn calculate_derivative_of_unmixing_energy<T: ColorModel>(
    alphas: &Mat1X,
    colors: &Mat3X,
    models: &Vec<&T>,
    sigma: Scalar,
    use_sparcity: bool,
    use_minimum_alpha: bool,
) -> Mat1X {
    let num_layers = alphas.len();
    let mut grad = Mat1X::from_element(num_layers * 4, 0.0);

    // Main term
    (0..num_layers).for_each(|idx| {
        let m = &models[idx];
        let u: Vec3 = colors.fixed_columns::<1>(idx).into();

        grad[idx] = m.calculate_distance(&u);
        grad.fixed_columns_mut::<3>(num_layers + idx * 3)
            .copy_from(&(alphas[idx] * m.calculate_distance_gradient(&u)).transpose());
    });

    // Sparcity term
    if use_sparcity {
        let alpha_sum = alphas.sum();
        let alpha_squared_sum = alphas.norm_squared();
        (0..num_layers).for_each(|index| {
            grad[index] += sigma * (alpha_squared_sum - 2.0 * alphas[index] * alpha_sum)
                / (alpha_squared_sum * alpha_squared_sum);
        });
    }

    // Minimum alpha term
    if use_minimum_alpha {
        let epsilon = 0.01;
        grad.columns_mut(0, num_layers)
            .copy_from(&(epsilon * Mat1X::from_element(num_layers, 1.0)));
    }

    grad
}

pub fn calculate_derivative_of_composite_alpha_by_source_alpha(
    alpha_d: Scalar,
    comp_op: &CompOp,
) -> Scalar {
    comp_op.x as Scalar * alpha_d + comp_op.y as Scalar * (1.0 - alpha_d)
        - comp_op.z as Scalar * alpha_d
}

pub fn calculate_derivative_of_composite_alpha_by_destination_alpha(
    alpha_s: Scalar,
    comp_op: &CompOp,
) -> Scalar {
    comp_op.x as Scalar * alpha_s - comp_op.y as Scalar * alpha_s
        + comp_op.z as Scalar * (1.0 - alpha_s)
}

// In general cases, the return value should be a dense 3-by-3 matrix; however, the use of separable blend functions allows it to be a diagonal matrix.
pub fn calculate_derivative_of_blend_function_by_source(
    c_s: &Vec3,
    c_d: &Vec3,
    mode: BlendMode,
) -> Vec3 {
    Vec3::new(
        mode.blend_grad_s(c_s[0], c_d[0], None),
        mode.blend_grad_s(c_s[1], c_d[1], None),
        mode.blend_grad_s(c_s[2], c_d[2], None),
    )
}

// In general cases, the return value should be a dense 3-by-3 matrix; however, the use of separable blend functions allows it to be a diagonal matrix.
pub fn calculate_derivative_of_blend_function_by_destination(
    c_s: &Vec3,
    c_d: &Vec3,
    mode: BlendMode,
) -> Vec3 {
    Vec3::new(
        mode.blend_grad_d(c_s[0], c_d[0], None),
        mode.blend_grad_d(c_s[1], c_d[1], None),
        mode.blend_grad_d(c_s[2], c_d[2], None),
    )
}

pub fn calculate_derivative_of_composite_two_layers_by_source(
    x_s: &Vec4,
    x_d: &Vec4,
    comp_op: &CompOp,
    mode: BlendMode,
) -> Mat4 {
    let x_m = composite_two_layers_wrapped(&x_s, &x_d, &comp_op, mode, None);
    let A = x_m[3];
    let B = x_m.fixed_rows::<3>(0);
    let D = mode.blend_vec(
        &x_s.fixed_rows::<3>(0).into(),
        &x_d.fixed_rows::<3>(0).into(),
        None,
    );

    let partial_A_per_partial_a_s =
        calculate_derivative_of_composite_alpha_by_source_alpha(x_d[3], comp_op);

    // Diagonal matrices (In general cases, these should be dense 3-by-3 matrices; however, the use of separable blend functions allows them to be diagonal matrices.)
    let partial_D_per_partial_c_s = calculate_derivative_of_blend_function_by_source(
        &x_s.fixed_rows::<3>(0).into(),
        &x_d.fixed_rows::<3>(0).into(),
        mode,
    );
    let partial_C_per_partial_c_s = x_s[3] * x_d[3] * partial_D_per_partial_c_s
        + Vec3::from_element(comp_op.y as Scalar * (1.0 - x_d[3]) * x_s[3]);
    let partial_B_per_partial_c_s = partial_C_per_partial_c_s / A;
    let partial_C_per_partial_a_s = D * x_d[3]
        + comp_op.y as Scalar * (1.0 - x_d[3]) * x_s.fixed_rows::<3>(0)
        - comp_op.z as Scalar * x_d[3] * x_d.fixed_rows::<3>(0);

    // Row vector
    let partial_B_per_partial_a_s =
        ((partial_C_per_partial_a_s - B * partial_A_per_partial_a_s) / A).transpose();

    let mut derivative = Mat4::zeros();
    derivative[(0, 0)] = partial_B_per_partial_c_s[0];
    derivative[(1, 1)] = partial_B_per_partial_c_s[1];
    derivative[(2, 2)] = partial_B_per_partial_c_s[2];
    derivative[(3, 3)] = partial_A_per_partial_a_s;
    derivative
        .fixed_view_mut::<1, 3>(3, 0)
        .copy_from(&partial_B_per_partial_a_s);

    derivative
}

pub fn calculate_derivative_of_composite_two_layers_by_destination(
    x_s: &Vec4,
    x_d: &Vec4,
    comp_op: &CompOp,
    mode: BlendMode,
) -> Mat4 {
    let x_m = composite_two_layers_wrapped(x_s, x_d, comp_op, mode, None);
    let A = x_m[3];
    let B = x_m.fixed_rows::<3>(0);
    let D = mode.blend_vec(
        &x_s.fixed_rows::<3>(0).into(),
        &x_d.fixed_rows::<3>(0).into(),
        None,
    );

    let partial_A_per_partial_a_d =
        calculate_derivative_of_composite_alpha_by_destination_alpha(x_s[3], comp_op);

    // Diagonal matrices (In general cases, these should be dense 3-by-3 matrices; however, the use of separable blend functions allows them to be diagonal matrices.)
    let partial_D_per_partial_c_d = calculate_derivative_of_blend_function_by_destination(
        &x_s.fixed_rows::<3>(0).into(),
        &x_d.fixed_rows::<3>(0).into(),
        mode,
    );
    let partial_C_per_partial_c_d = x_s[3] * x_d[3] * partial_D_per_partial_c_d
        + Vec3::from_element(comp_op.z as Scalar * (1.0 - x_s[3]) * x_d[3]);
    let partial_B_per_partial_c_d = partial_C_per_partial_c_d / A;
    let partial_C_per_partial_a_d = D * x_s[3]
        - comp_op.y as Scalar * x_s[3] * x_s.fixed_rows::<3>(0)
        + comp_op.z as Scalar * (1.0 - x_s[3]) * x_d.fixed_rows::<3>(0);

    // Row vector
    let partial_B_per_partial_a_d =
        ((partial_C_per_partial_a_d - B * partial_A_per_partial_a_d) / A).transpose();

    let mut derivative = Mat4::zeros();
    derivative[(0, 0)] = partial_B_per_partial_c_d[0];
    derivative[(1, 1)] = partial_B_per_partial_c_d[1];
    derivative[(2, 2)] = partial_B_per_partial_c_d[2];
    derivative[(3, 3)] = partial_A_per_partial_a_d;
    derivative
        .fixed_view_mut::<1, 3>(3, 0)
        .copy_from(&partial_B_per_partial_a_d);

    derivative
}

/// \details Equation 16:
/// \f[
///     \frac{\partial}{\partial \mathbf{x}_i} \hat{\mathbf{x}}_k
/// \f]
pub fn calculate_derivative_of_k_th_composited_rgba_by_i_th_layer_rgba(
    alphas: &Mat1X,
    colors: &Mat3X,
    target_color: &Vec3,
    comp_ops: &Vec<CompOp>,
    modes: &Vec<BlendMode>,
    i: usize,
    k: usize,
) -> Mat4 {
    if i == 0 && k == 0 {
        return Mat4::identity();
    }

    let comp_ops_k_minus_1: Vec<CompOp> = comp_ops[..k].into();
    let modes_k_minus_1: Vec<BlendMode> = modes[..k].into();

    let x_k = Vec4::new(
        colors[k * 3 + 0],
        colors[k * 3 + 1],
        colors[k * 3 + 2],
        alphas[k],
    );
    let x_hat_k_minus_1 = composite_layers(
        &alphas.columns(0, k).into(),
        &colors.columns(0, k).into(),
        &comp_ops_k_minus_1,
        &modes_k_minus_1,
        false,
    );

    return if i == k {
        calculate_derivative_of_composite_two_layers_by_source(
            &x_k,
            &x_hat_k_minus_1,
            &comp_ops[k],
            modes[k],
        )
    } else {
        let derivative_1 = calculate_derivative_of_k_th_composited_rgba_by_i_th_layer_rgba(
            alphas,
            colors,
            target_color,
            comp_ops,
            modes,
            i,
            k - 1,
        );
        let derivative_2 = calculate_derivative_of_composite_two_layers_by_destination(
            &x_k,
            &x_hat_k_minus_1,
            &comp_ops[k],
            modes[k],
        );

        derivative_1 * derivative_2
    };
}

pub fn calculate_derivative_of_constraint_vector(
    alphas: &Mat1X,
    colors: &Mat3X,
    target_color: &Vec3,
    comp_ops: &Vec<CompOp>,
    modes: &Vec<BlendMode>,
    use_target_alphas: bool,
    // target_alphas: &VecX,
    gray_layers: &Vec<usize>,
) -> MatX {
    let num_gray_layers = gray_layers.len();
    let num_layers = alphas.len();
    let num_alpha_constraints = if use_target_alphas { num_layers } else { 1 };
    let num_constraints = 3 + num_alpha_constraints + 3 * num_gray_layers;

    let mut derivative = MatX::zeros(4 * num_layers, num_constraints);

    (0..num_layers).for_each(|i| {
        let i_th_derivative = calculate_derivative_of_k_th_composited_rgba_by_i_th_layer_rgba(
            alphas,
            &colors,
            target_color,
            comp_ops,
            modes,
            i,
            num_layers - 1,
        );

        if use_target_alphas {
            derivative
                .fixed_view_mut::<1, 3>(i, 0)
                .copy_from(&i_th_derivative.fixed_view::<1, 3>(3, 0));
            derivative
                .fixed_view_mut::<3, 3>(num_layers + i * 3, 0)
                .copy_from(&i_th_derivative.fixed_view::<3, 3>(0, 0));
            derivative[(i, 3 + i)] = 1.0;
        } else {
            derivative
                .fixed_view_mut::<1, 4>(i, 0)
                .copy_from(&i_th_derivative.fixed_view::<1, 4>(3, 0));
            derivative
                .fixed_view_mut::<3, 4>(num_layers + i * 3, 0)
                .copy_from(&i_th_derivative.fixed_view::<3, 4>(0, 0));
        }
    });

    // Constraints for gray-scale layers
    (0..num_gray_layers).for_each(|i| {
        let gray_layer = gray_layers[i];
        let color: Vec3 = colors.fixed_columns::<1>(gray_layer).into();
        let ccc = Mat3::from_columns(&[color, color, color]);

        // Note: When color.norm() is sufficiently small, the derivative becomes nearly zeros
        let epsilon = 1e-03;
        if color.norm() > epsilon {
            derivative
                .fixed_view_mut::<3, 3>(
                    num_layers + gray_layer * 3,
                    3 + num_alpha_constraints + i * 3,
                )
                .copy_from(&(3.0.sqrt() * Mat3::identity() - (1.0 / color.norm()) * ccc));
        }
    });

    derivative
}

// Wrappers

pub fn composite_two_layers_wrapped(
    x_s: &Vec4,
    x_d: &Vec4,
    comp_op: &CompOp,
    mode: BlendMode,
    crop: Option<bool>,
) -> Vec4 {
    let crop = crop.unwrap_or(false);
    return composite_two_layers(
        &x_s.fixed_rows::<3>(0).into(),
        &x_d.fixed_rows::<3>(0).into(),
        x_s[3],
        x_d[3],
        comp_op,
        mode,
        crop,
    );
}

pub fn calculate_unmixing_energy_term_wrapped<T: ColorModel>(
    x: Mat4X,
    models: &Vec<&T>,
    sigma: Scalar,
    use_sparcity: bool,
    use_minimum_alpha: bool,
) -> Scalar {
    let alphas = x.fixed_rows::<1>(3);
    let colors = x.fixed_rows::<3>(0);
    calculate_unmixing_energy_term(
        &alphas.into(),
        &colors.into(),
        models,
        sigma,
        use_sparcity,
        use_minimum_alpha,
    )
}

pub fn calculate_constraint_vector_wrapped(
    x: &Mat4X,
    target_color: &Vec3,
    comp_ops: &Vec<CompOp>,
    modes: &Vec<BlendMode>,
    use_target_alphas: bool,
    target_alphas: &Mat1X,
    gray_layers: &Vec<usize>,
) -> Mat1X {
    let alphas = x.fixed_rows::<1>(3);
    let colors = x.fixed_rows::<3>(0);
    calculate_constraint_vector(
        &alphas.into(),
        &colors.into(),
        target_color,
        comp_ops,
        modes,
        use_target_alphas,
        target_alphas,
        gray_layers,
    )
}
