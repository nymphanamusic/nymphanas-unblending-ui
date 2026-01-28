use crate::unblending::blend_mode::BlendMode;
use crate::unblending::color_model::ColorModel;
use crate::unblending::comp_op::CompOp;

pub struct LayerInfo<T: ColorModel> {
    pub comp_op: CompOp,
    pub blend_mode: BlendMode,
    pub color_model: T,
}

pub fn extract_comp_ops<T: ColorModel>(layer_infos: &Vec<&LayerInfo<T>>) -> Vec<CompOp> {
    layer_infos
        .iter()
        .map(|layer_info| layer_info.comp_op)
        .collect()
}

pub fn extract_blend_modes<T: ColorModel>(layer_infos: &Vec<&LayerInfo<T>>) -> Vec<BlendMode> {
    layer_infos
        .iter()
        .map(|layer_info| layer_info.blend_mode)
        .collect()
}

pub fn extract_color_models<'a, T: ColorModel>(layer_infos: &Vec<&'a LayerInfo<T>>) -> Vec<&'a T> {
    layer_infos
        .iter()
        .map(|layer_info| &layer_info.color_model)
        .collect()
}
