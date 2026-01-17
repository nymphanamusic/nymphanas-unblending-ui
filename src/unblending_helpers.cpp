#include <unblending_helpers.hpp>
#include <Eigen/Core>
#include <Eigen/LU>
#include <unblending/blend_mode.hpp>
#include <unblending/comp_op.hpp>
#include <unblending/layer_info.hpp>
#include <unblending/unblending.hpp>

namespace unblending {
    Eigen::Vector3d make_vector3d(float x, float y, float z) {
        return { x, y, z };
    }

    Eigen::Vector4d make_vector4d(float x, float y, float z, float w) {
        return { x, y, z, w };
    }

    Eigen::Matrix3d make_matrix3d(float value) {
        Eigen::Matrix3d mat;
        for (int col = 0; col < 3; ++ col) {
            for (int row = 0; row < 3; ++ row) {
                mat.coeffRef(row, col) = value;
            }
        }
        return mat;
    }

    Eigen::Matrix3d make_matrix3d(
        float m00, float m01, float m02,
        float m10, float m11, float m12,
        float m20, float m21, float m22
    ) {
        Eigen::Matrix3d mat;
        mat.coeffRef(0, 0) = m00;
        mat.coeffRef(0, 1) = m01;
        mat.coeffRef(0, 2) = m02;
        mat.coeffRef(1, 0) = m10;
        mat.coeffRef(1, 1) = m11;
        mat.coeffRef(1, 2) = m12;
        mat.coeffRef(2, 0) = m20;
        mat.coeffRef(2, 1) = m21;
        mat.coeffRef(2, 2) = m22;
        return mat;
    }

    LayerInfo make_layer_info(
        CompOp comp_op,
        BlendMode blend_mode,
        Eigen::Vector3d primary_color,
        Eigen::Matrix3d variance
    ) {
        return LayerInfo{
            comp_op,
            blend_mode,
            std::make_shared<GaussianColorModel>(primary_color, variance.inverse())
        };
    }
}