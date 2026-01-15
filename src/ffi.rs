#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("nymphanas-unblending-ui/include/unblending/unblending/include/unblending.hpp");
        include!("nymphanas-unblending-ui/include/unblending/unblending/include/equations.hpp");

        type ColorImage;

        fn
    }
}
