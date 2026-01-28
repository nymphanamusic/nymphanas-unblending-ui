use crate::unblending::blend_mode::BlendMode;
use crate::unblending::color_model::ColorModel;
use crate::unblending::comp_op::CompOp;

pub struct LayerInfo {
    comp_op: CompOp,
    blend_mode: BlendMode,
    color_model: dyn ColorModel,
}

pub fn extract_comp_ops<'a>(layer_infos: &Vec<&'a LayerInfo>) -> Vec<CompOp> {
    layer_infos
        .iter()
        .map(|layer_info| layer_info.comp_op)
        .collect()
}

pub fn extract_blend_modes(layer_infos: &Vec<&LayerInfo>) -> Vec<BlendMode> {
    layer_infos
        .iter()
        .map(|layer_info| layer_info.blend_mode)
        .collect()
}

pub fn extract_color_models<'a>(layer_infos: &Vec<&'a LayerInfo>) -> Vec<Box<&'a dyn ColorModel>> {
    layer_infos
        .iter()
        .map(|layer_info| Box::new(&layer_info.color_model))
        .collect()
}
