use alloc::ffi::CString;
use core::ffi::c_char;
use core::ops::Deref;
use std::sync::Once;

use crate::encode::{Encode, Encoding, RefEncode};
use crate::rc::Retained;
use crate::runtime::{AnyClass, AnyObject, AnyProtocol, ClassBuilder, ProtocolBuilder, Sel};
use crate::{ffi, msg_send, sel, Message};

#[derive(Debug)]
#[repr(C)]
pub(crate) struct CustomObject(AnyObject);

unsafe impl RefEncode for CustomObject {
    const ENCODING_REF: Encoding = Encoding::Object;
}

unsafe impl Message for CustomObject {}

impl Deref for CustomObject {
    type Target = AnyObject;

    fn deref(&self) -> &AnyObject {
        &self.0
    }
}

#[derive(Debug, Eq, PartialEq)]
#[repr(C)]
pub(crate) struct CustomStruct {
    pub(crate) a: u64,
    pub(crate) b: u64,
    pub(crate) c: u64,
    pub(crate) d: u64,
}

unsafe impl Encode for CustomStruct {
    const ENCODING: Encoding = Encoding::Struct(
        "CustomStruct",
        &[u64::ENCODING, u64::ENCODING, u64::ENCODING, u64::ENCODING],
    );
}

// TODO: Remove once c"" strings are in MSRV
fn c(s: &str) -> CString {
    CString::new(s).unwrap()
}

pub(crate) fn custom_class() -> &'static AnyClass {
    static REGISTER_CUSTOM_CLASS: Once = Once::new();

    REGISTER_CUSTOM_CLASS.call_once(|| {
        // The runtime will call this method, so it has to be implemented
        extern "C-unwind" fn custom_obj_class_initialize(_this: &AnyClass, _cmd: Sel) {}

        let mut builder = ClassBuilder::root(
            &c("CustomObject"),
            custom_obj_class_initialize as extern "C-unwind" fn(_, _),
        )
        .unwrap();
        let proto = custom_protocol();

        builder.add_protocol(proto);
        builder.add_ivar::<u32>(&c("_foo"));

        unsafe extern "C-unwind" fn custom_obj_release(this: *mut AnyObject, _cmd: Sel) {
            unsafe {
                #[allow(deprecated)]
                ffi::object_dispose(this);
            }
        }

        extern "C-unwind" fn custom_obj_set_foo(this: &AnyObject, _cmd: Sel, foo: u32) {
            let ivar = this.class().instance_variable(&c("_foo")).unwrap();
            unsafe { *ivar.load_ptr::<u32>(this) = foo }
        }

        extern "C-unwind" fn custom_obj_get_foo(this: &AnyObject, _cmd: Sel) -> u32 {
            let ivar = this.class().instance_variable(&c("_foo")).unwrap();
            unsafe { *ivar.load::<u32>(this) }
        }

        extern "C-unwind" fn custom_obj_get_foo_reference(this: &AnyObject, _cmd: Sel) -> &u32 {
            let ivar = this.class().instance_variable(&c("_foo")).unwrap();
            unsafe { ivar.load::<u32>(this) }
        }

        extern "C-unwind" fn custom_obj_get_struct(_this: &AnyObject, _cmd: Sel) -> CustomStruct {
            CustomStruct {
                a: 1,
                b: 2,
                c: 3,
                d: 4,
            }
        }

        extern "C-unwind" fn custom_obj_class_method(_this: &AnyClass, _cmd: Sel) -> u32 {
            7
        }

        extern "C-unwind" fn get_nsinteger(_this: &AnyObject, _cmd: Sel) -> ffi::NSInteger {
            5
        }

        extern "C-unwind" fn custom_obj_set_bar(this: &AnyObject, _cmd: Sel, bar: u32) {
            let ivar = this.class().instance_variable(&c("_bar")).unwrap();
            unsafe { *ivar.load_ptr::<u32>(this) = bar }
        }

        extern "C-unwind" fn custom_obj_add_number_to_number(
            _this: &AnyClass,
            _cmd: Sel,
            fst: i32,
            snd: i32,
        ) -> i32 {
            fst + snd
        }

        extern "C-unwind" fn custom_obj_multiple_colon(
            _obj: &AnyObject,
            _cmd: Sel,
            arg1: i32,
            arg2: i32,
            arg3: i32,
            arg4: i32,
        ) -> i32 {
            arg1 * arg2 * arg3 * arg4
        }

        extern "C-unwind" fn custom_obj_multiple_colon_class(
            _cls: &AnyClass,
            _cmd: Sel,
            arg1: i32,
            arg2: i32,
            arg3: i32,
            arg4: i32,
        ) -> i32 {
            arg1 + arg2 + arg3 + arg4
        }

        unsafe {
            // On GNUStep 2.0, it is required to have `dealloc` methods for some reason
            if cfg!(all(feature = "gnustep-2-0", not(feature = "gnustep-2-1"))) {
                unsafe extern "C-unwind" fn forward_to_dealloc(this: *mut AnyObject, _cmd: Sel) {
                    unsafe { msg_send![this, dealloc] }
                }

                let release: unsafe extern "C-unwind" fn(_, _) = forward_to_dealloc;
                builder.add_method(sel!(release), release);

                let release: unsafe extern "C-unwind" fn(_, _) = custom_obj_release;
                builder.add_method(sel!(dealloc), release);
            } else {
                let release: unsafe extern "C-unwind" fn(_, _) = custom_obj_release;
                builder.add_method(sel!(release), release);
            }

            let set_foo: extern "C-unwind" fn(_, _, _) = custom_obj_set_foo;
            builder.add_method(sel!(setFoo:), set_foo);
            let get_foo: extern "C-unwind" fn(_, _) -> _ = custom_obj_get_foo;
            builder.add_method(sel!(foo), get_foo);
            let get_foo_reference: extern "C-unwind" fn(_, _) -> _ = custom_obj_get_foo_reference;
            builder.add_method(sel!(fooReference), get_foo_reference);
            let get_struct: extern "C-unwind" fn(_, _) -> CustomStruct = custom_obj_get_struct;
            builder.add_method(sel!(customStruct), get_struct);
            let class_method: extern "C-unwind" fn(_, _) -> _ = custom_obj_class_method;
            builder.add_class_method(sel!(classFoo), class_method);

            let get_nsinteger: extern "C-unwind" fn(_, _) -> _ = get_nsinteger;
            builder.add_method(sel!(getNSInteger), get_nsinteger);

            let protocol_instance_method: extern "C-unwind" fn(_, _, _) = custom_obj_set_bar;
            builder.add_method(sel!(setBar:), protocol_instance_method);
            let protocol_class_method: extern "C-unwind" fn(_, _, _, _) -> _ =
                custom_obj_add_number_to_number;
            builder.add_class_method(sel!(addNumber:toNumber:), protocol_class_method);

            let f: extern "C-unwind" fn(_, _, _, _, _, _) -> _ = custom_obj_multiple_colon;
            builder.add_method(sel!(test::test::), f);
            let f: extern "C-unwind" fn(_, _, _, _, _, _) -> _ = custom_obj_multiple_colon_class;
            builder.add_class_method(sel!(test::test::), f);
        }

        builder.register();
    });

    // Can't use `class!` here since `CustomObject` is dynamically created.
    AnyClass::get(&c("CustomObject")).unwrap()
}

pub(crate) fn custom_protocol() -> &'static AnyProtocol {
    static REGISTER_CUSTOM_PROTOCOL: Once = Once::new();

    REGISTER_CUSTOM_PROTOCOL.call_once(|| {
        let mut builder = ProtocolBuilder::new(&c("CustomProtocol")).unwrap();

        builder.add_method_description::<(i32,), ()>(sel!(setBar:), true);
        builder.add_method_description::<(), *const c_char>(sel!(getName), false);
        builder.add_class_method_description::<(i32, i32), i32>(sel!(addNumber:toNumber:), true);

        builder.register();
    });

    AnyProtocol::get(&c("CustomProtocol")).unwrap()
}

pub(crate) fn custom_subprotocol() -> &'static AnyProtocol {
    static REGISTER_CUSTOM_SUBPROTOCOL: Once = Once::new();

    REGISTER_CUSTOM_SUBPROTOCOL.call_once(|| {
        let super_proto = custom_protocol();
        let mut builder = ProtocolBuilder::new(&c("CustomSubProtocol")).unwrap();

        builder.add_protocol(super_proto);
        builder.add_method_description::<(u32,), u32>(sel!(calculateFoo:), true);

        builder.register();
    });

    AnyProtocol::get(&c("CustomSubProtocol")).unwrap()
}

pub(crate) fn custom_object() -> Retained<CustomObject> {
    let ptr: *const AnyClass = custom_class();
    unsafe { Retained::from_raw(ffi::class_createInstance(ptr, 0).cast()) }.unwrap()
}

pub(crate) fn custom_subclass() -> &'static AnyClass {
    static REGISTER_CUSTOM_SUBCLASS: Once = Once::new();

    REGISTER_CUSTOM_SUBCLASS.call_once(|| {
        let superclass = custom_class();
        let mut builder = ClassBuilder::new(&c("CustomSubclassObject"), superclass).unwrap();

        extern "C-unwind" fn custom_subclass_get_foo(this: &AnyObject, _cmd: Sel) -> u32 {
            let foo: u32 = unsafe { msg_send![super(this, custom_class()), foo] };
            foo + 2
        }

        extern "C-unwind" fn custom_subclass_class_method(_cls: &AnyClass, _cmd: Sel) -> u32 {
            9
        }

        unsafe {
            let get_foo: extern "C-unwind" fn(_, _) -> _ = custom_subclass_get_foo;
            builder.add_method(sel!(foo), get_foo);
            let class_method: extern "C-unwind" fn(_, _) -> _ = custom_subclass_class_method;
            builder.add_class_method(sel!(classFoo), class_method);
        }

        builder.register();
    });

    AnyClass::get(&c("CustomSubclassObject")).unwrap()
}

pub(crate) fn custom_subclass_object() -> Retained<CustomObject> {
    let ptr: *const AnyClass = custom_subclass();
    unsafe { Retained::from_raw(ffi::class_createInstance(ptr, 0).cast()) }.unwrap()
}
