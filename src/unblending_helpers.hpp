#ifndef UNBLENDING_HELPERS_HPP
#define UNBLENDING_HELPERS_HPP

#include <Eigen/Core>
#include <unblending/blend_mode.hpp>
#include <unblending/comp_op.hpp>
#include <unblending/layer_info.hpp>
#include <unblending/unblending.hpp>

namespace unblending_helpers {
    Eigen::Vector3d make_vector3d(double x, double y, double z);

    Eigen::Vector4d make_vector4d(double x, double y, double z, double w);

    Eigen::Matrix3d make_matrix3d_s(double value);

    Eigen::Matrix3d make_matrix3d(
        double m00, double m01, double m02,
        double m10, double m11, double m12,
        double m20, double m21, double m22
    );

    unblending::LayerInfo make_layer_info(
        unblending::CompOp comp_op,
        unblending::BlendMode blend_mode,
        Eigen::Vector3d primary_color,
        Eigen::Matrix3d variance
    );

    void push_layer_info(std::vector<unblending::LayerInfo> &vec, unblending::LayerInfo &element);
}

#endif // UNBLENDING_HELPERS_HPP