#ifndef UNBLENDING_HELPERS_HPP
#define UNBLENDING_HELPERS_HPP

#include <Eigen/Core>

namespace unblending {
    Eigen::Vector3d make_vector3d(float x, float y, float z);
    Eigen::Vector4d make_vector4d(float x, float y, float z, float w);
}

#endif // UNBLENDING_HELPERS_HPP