#ifndef UNBLENDING_HELPERS_HPP
#define UNBLENDING_HELPERS_HPP

#include <Eigen/Core>
#include <unblending/blend_mode.hpp>
#include <unblending/comp_op.hpp>
#include <unblending/layer_info.hpp>
#include <unblending/unblending.hpp>

namespace unblending {
    Eigen::Vector3d make_vector3d(float x, float y, float z);

    Eigen::Vector4d make_vector4d(float x, float y, float z, float w);

    Eigen::Matrix3d make_matrix3d(float value);

    Eigen::Matrix3d make_matrix3d(
        float m00, float m01, float m02,
        float m10, float m11, float m12,
        float m20, float m21, float m22
    );

    LayerInfo make_layer_info(
        CompOp comp_op,
        BlendMode blend_mode,
        Eigen::Vector3d primary_color,
        Eigen::Matrix3d variance
    );
}

#endif // UNBLENDING_HELPERS_HPP