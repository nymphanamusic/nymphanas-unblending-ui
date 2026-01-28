use crate::unblending::common::Scalar;
use itertools::Itertools;
use nlopt::ObjFn;

const constraint_tol: f64 = 1e-10;

pub fn solve<T>(
    x_initial: &[Scalar],
    upper: &[Scalar],
    lower: &[Scalar],
    objective_function: &dyn ObjFn<T>,
    equality_constraints: Option<Vec<&dyn ObjFn<T>>>,
    inequality_constraints: Option<Vec<&dyn ObjFn<T>>>,
    algorithm: Option<nlopt::Algorithm>,
    data: &T,
    is_maximization: Option<bool>,
    max_evaluations: Option<u32>,
    relative_func_tol: Option<Scalar>,
    relative_param_tol: Option<Scalar>,
    verbose: Option<bool>,
    initial_step_scale: Option<Scalar>,
) -> Vec<Scalar>
where
    T: Clone,
{
    let algorithm = algorithm.unwrap_or(nlopt::Algorithm::TNewton);
    let is_maximization = is_maximization.unwrap_or(false);
    let max_evaluations = max_evaluations.unwrap_or(1000);
    let relative_func_tol = relative_func_tol.unwrap_or(1e-06);
    let relative_param_tol = relative_param_tol.unwrap_or(1e-06);
    let verbose = verbose.unwrap_or(false);
    let initial_step_scale = initial_step_scale.unwrap_or(1.0);

    let M = x_initial.len();

    let mut solver = nlopt::Nlopt::new(
        algorithm,
        M,
        objective_function,
        if is_maximization {
            nlopt::Target::Maximize
        } else {
            nlopt::Target::Minimize
        },
        data.clone(),
    );

    if upper.len() != 0 {
        solver.set_upper_bounds(upper);
    }

    if lower.len() != 0 {
        solver.set_lower_bounds(lower);
    }

    solver.set_maxeval(max_evaluations);
    solver.set_ftol_rel(relative_func_tol);
    solver.set_xtol_rel(relative_param_tol);

    if let Some(equality_constraints) = equality_constraints {
        equality_constraints.iter().for_each(|func| {
            solver.add_equality_constraint(func, data.clone(), constraint_tol);
        })
    };

    if let Some(inequality_constraints) = inequality_constraints {
        inequality_constraints.iter().for_each(|func| {
            solver.add_inequality_constraint(func, data.clone(), constraint_tol);
        })
    };

    let mut x_star = x_initial.to_owned();

    // // Record the cost value for the initial solution
    // let initial_cost_value: Scalar;
    // if verbose {
    //     let dummy = Vec::from_iter(repeat(0.0).take(M));
    //     initial_cost_value = objective_function(x_star, dummy, data);
    // }

    // Scale the initial step size (only for derivative-free algorithms such as nlopt::LN_COBYLA)
    let step = solver
        .get_initial_step(&x_star)
        .unwrap()
        .iter()
        .map(|x| x * initial_step_scale)
        .collect_vec();
    solver.set_initial_step(step.as_slice());

    // // Start timing measurement
    // const auto t_start = std::chrono::system_clock::now();

    // Run the optimization
    let final_cost_value = solver.optimize(&mut x_star).ok();

    // catch (nlopt::roundoff_limited)
    // // Ignore roundoff_limited exceptions
    // catch (std::invalid_argument e)
    // {
    //     if (verbose) { std::cerr << e.what() << std::endl; }
    //     assert(false);
    // }
    // catch (std::runtime_error e)
    // {
    //     if (verbose) { std::cerr << e.what() << std::endl; }
    //     return x_initial;
    // }

    // // Stop timing measurement
    // const auto t_end = std::chrono::system_clock::now();

    // // Show statistics if "verbose" is set as true
    // if (verbose)
    // {
    //     const double t_elapsed_in_sec = std::chrono::duration_cast<std::chrono::milliseconds>(t_end - t_start).count() / 1000.0;
    //
    //     std::cout << "---- nlopt-util ----" << std::endl;
    //     std::cout << "Dimensions     : " << M << std::endl;
    //     std::cout << "Function value : " << initial_cost_value << " => " << final_cost_value << std::endl;
    //     std::cout << "Elapsed time   : " << t_elapsed_in_sec << " [s]" << std::endl;
    //     std::cout << "--------------------" << std::endl;
    // }

    // return Eigen::Map < Eigen::VectorXd > (&x_star[0], M);
    x_star
}
