use objc2::define_class;
use objc2::rc::{Allocated, Retained};
use objc2::runtime::NSObject;

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "CustomObject"]
    struct CustomObject;

    impl CustomObject {
        #[unsafe(method_id(initNotSameGenerics))]
        fn test_init_not_same_generics(this: Allocated<Self>) -> Retained<NSObject> {
            unimplemented!()
        }

        #[unsafe(method_id(notRetained))]
        fn test_not_retained(&self) -> i32 {
            unimplemented!()
        }
    }
);

fn main() {}
