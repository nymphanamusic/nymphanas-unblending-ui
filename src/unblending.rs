use autocxx::prelude::*;
include_cpp! {
    #include "unblending/unblending.hpp"
    #include "unblending/image_processing.hpp"
    safety!(unsafe_ffi)
    generate!("unblending::ColorImage")
}

#[allow(unused_imports)]
pub use ffi::*;
