use crate::unblending::blend_mode::BlendMode;
use crate::unblending::color_model::ColorModel;
use crate::unblending::common::{Mat1X, Vec3};
use crate::unblending::comp_op::CompOp;

#[derive(Clone)]
pub struct OptimizationParameterSet<'a> {
    pub models: Vec<Box<&'a dyn ColorModel>>,
    pub comp_ops: Vec<CompOp>,
    pub modes: Vec<BlendMode>,

    pub target_color: Vec3,
    pub lambda: Mat1X,
    pub rho: f64,
    pub sigma: f64, // Weight for the sparcity term
    pub use_sparcity: bool,
    pub use_minimum_alpha: bool,
    pub use_target_alphas: bool, // If true, the alternative constraint (Eq. 6) will be used instead of the unity constraint (Eq. 2).
    pub target_alphas: Mat1X,    // This will be used when "use_target_alphas" is true.

    pub gray_layers: Vec<usize>, // A list of gray layer indices. For example, if the second and fourth layers are to be gray, it looks like { 1, 3 }.
}

impl<'a> OptimizationParameterSet<'a> {
    pub fn new(
        target_color: Vec3,
        models: Vec<Box<&'a dyn ColorModel>>,
        comp_ops: Vec<CompOp>,
        modes: Vec<BlendMode>,
        is_for_refinement: Option<bool>,
        target_alphas: Option<Mat1X>,
    ) -> Self {
        let is_for_refinement = is_for_refinement.unwrap_or(false);
        let target_alphas = target_alphas.unwrap_or(Mat1X::zeros(0));

        let num_layers = models.len();
        let gray_layers: Vec<usize> = vec![];
        let num_alpha_constraints = if is_for_refinement { num_layers } else { 1 };
        let num_constraints = 3 + num_alpha_constraints + 3 * gray_layers.len();
        let initial_rho = 100.0;

        OptimizationParameterSet::<'a> {
            models,
            comp_ops,
            modes,
            lambda: Mat1X::from_element(num_constraints, 0.0),
            rho: initial_rho,
            target_color,
            sigma: 10.0,
            target_alphas,
            use_sparcity: false,
            use_minimum_alpha: !is_for_refinement,
            use_target_alphas: is_for_refinement,
            gray_layers,
        }
    }
}
