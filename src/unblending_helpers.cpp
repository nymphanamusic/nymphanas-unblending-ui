#include <unblending_helpers.hpp>
#include <Eigen/Core>

namespace unblending {
    Eigen::Vector3d make_vector3d(float x, float y, float z) {
        return { x, y, z };
    }
    Eigen::Vector4d make_vector4d(float x, float y, float z, float w) {
        return { x, y, z, w };
    }
}