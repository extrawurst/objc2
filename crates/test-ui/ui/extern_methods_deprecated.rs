//! Test using deprecated `#[unsafe(method_id(...))]`.
#![deny(warnings)]
use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{extern_class, extern_methods};

extern_class!(
    #[unsafe(super(NSObject))]
    pub struct MyObject;
);

impl MyObject {
    extern_methods!(
        #[unsafe(method_id(myMethod:))]
        fn my_method(param: i32) -> Retained<Self>;
    );
}

fn main() {}
