#include <unblending_helpers.hpp>
#include <Eigen/Core>
#include <Eigen/LU>
#include <unblending/blend_mode.hpp>
#include <unblending/comp_op.hpp>
#include <unblending/layer_info.hpp>
#include <unblending/unblending.hpp>

namespace unblending_helpers {
//    struct Vector3dWrapper {
//        Eigen::Vector3d vec {};
//
//        Eigen::Vector3d get() {
//            return vec;
//        }
//
//        static inline load(Eigen::Vector3d vec) { vec };
//
//        double x() {
//            return vec.x();
//        }
//    }

    Eigen::Vector3d make_vector3d(double x, double y, double z) {
        return { x, y, z };
    }

    Eigen::Vector4d make_vector4d(double x, double y, double z, double w) {
        return { x, y, z, w };
    }

    Eigen::Matrix3d make_matrix3d_s(double value) {
        Eigen::Matrix3d mat;
        for (int col = 0; col < 3; ++ col) {
            for (int row = 0; row < 3; ++ row) {
                mat.coeffRef(row, col) = value;
            }
        }
        return mat;
    }

    Eigen::Matrix3d make_matrix3d(
        double m00, double m01, double m02,
        double m10, double m11, double m12,
        double m20, double m21, double m22
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

    unblending::LayerInfo make_layer_info(
        unblending::CompOp comp_op,
        unblending::BlendMode blend_mode,
        Eigen::Vector3d primary_color,
        Eigen::Matrix3d variance
    ) {
        return unblending::LayerInfo{
            comp_op,
            blend_mode,
            std::make_shared<unblending::GaussianColorModel>(primary_color, variance.inverse())
        };
    }

    void push_layer_info(std::vector<unblending::LayerInfo> &vec, unblending::LayerInfo &element) {
      vec.emplace_back(std::move(element));
    }
}