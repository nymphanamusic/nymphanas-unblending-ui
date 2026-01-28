use nalgebra::{
    DMatrix, Matrix1xX, Matrix2, Matrix3, Matrix3xX, Matrix4, Matrix4xX, Vector2, Vector3, Vector4,
};
use std::ops::Range;

pub type Scalar = f64;
pub type Vec2 = Vector2<Scalar>;
pub type Vec3 = Vector3<Scalar>;
pub type Vec4 = Vector4<Scalar>;
pub type Mat2 = Matrix2<Scalar>;
pub type Mat3 = Matrix3<Scalar>;
pub type Mat4 = Matrix4<Scalar>;
pub type Mat1X = Matrix1xX<Scalar>;
pub type Mat3X = Matrix3xX<Scalar>;
pub type Mat4X = Matrix4xX<Scalar>;
pub type MatX = DMatrix<Scalar>;

pub fn iter_xy(x_range: Range<usize>, y_range: Range<usize>, mut f: impl FnMut(usize, usize)) {
    x_range.for_each(|x| y_range.clone().for_each(|y| f(x, y)));
}
