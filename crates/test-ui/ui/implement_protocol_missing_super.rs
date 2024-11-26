//! Test that implementing certain traits like `NSURLSessionDelegate` requires
//! super protocols like `NSObjectProtocol` to also be implemented.
use objc2::{declare_class, MainThreadOnly};
use objc2_foundation::{NSObject, NSURLSessionDelegate};

declare_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "CustomObject"]
    struct CustomObject;

    unsafe impl NSURLSessionDelegate for CustomObject {}
);

fn main() {}
