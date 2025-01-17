use crate::encode::{EncodeArgument, EncodeArguments, EncodeReturn};
use crate::rc::Retained;
use crate::runtime::Bool;
use crate::Message;

mod argument_private {
    pub trait Sealed {}
}

/// Represents types that can be converted to/from an [`EncodeArgument`] type.
///
/// This is implemented specially for [`bool`] to allow using that as
/// Objective-C `BOOL`, where it would otherwise not be allowed (since they
/// are not ABI compatible).
///
/// This is also done specially for `&mut Retained<_>`-like arguments, to
/// allow using those as "out" / pass-by-writeback parameters.
pub trait ConvertArgument: argument_private::Sealed {
    /// The inner type that this can be converted to and from.
    #[doc(hidden)]
    type __Inner: EncodeArgument;

    /// A helper type for out parameters.
    ///
    /// When dropped, this will process any necessary change to the
    /// parameters.
    #[doc(hidden)]
    type __WritebackOnDrop: Sized;

    #[doc(hidden)]
    fn __from_defined_param(inner: Self::__Inner) -> Self;

    /// # Safety
    ///
    /// The `__WritebackOnDrop` return type must not be leaked, and the
    /// `__Inner` pointer must not be used after the `__WritebackOnDrop` has
    /// dropped.
    ///
    /// NOTE: The standard way to ensure such a thing is with closures, but
    /// using those would interact poorly with backtraces of the message send,
    /// so we're forced to ensure this out of band.
    #[doc(hidden)]
    unsafe fn __into_argument(self) -> (Self::__Inner, Self::__WritebackOnDrop);
}

// Implemented in writeback.rs
impl<T: Message> argument_private::Sealed for &mut Retained<T> {}
impl<T: Message> argument_private::Sealed for Option<&mut Retained<T>> {}
impl<T: Message> argument_private::Sealed for &mut Option<Retained<T>> {}
impl<T: Message> argument_private::Sealed for Option<&mut Option<Retained<T>>> {}

impl<T: EncodeArgument> argument_private::Sealed for T {}
impl<T: EncodeArgument> ConvertArgument for T {
    type __Inner = Self;

    type __WritebackOnDrop = ();

    #[inline]
    fn __from_defined_param(inner: Self::__Inner) -> Self {
        inner
    }

    #[inline]
    unsafe fn __into_argument(self) -> (Self::__Inner, Self::__WritebackOnDrop) {
        (self, ())
    }
}

impl argument_private::Sealed for bool {}
impl ConvertArgument for bool {
    type __Inner = Bool;

    type __WritebackOnDrop = ();

    #[inline]
    fn __from_defined_param(inner: Self::__Inner) -> Self {
        inner.as_bool()
    }

    #[inline]
    unsafe fn __into_argument(self) -> (Self::__Inner, Self::__WritebackOnDrop) {
        (Bool::new(self), ())
    }
}

mod return_private {
    pub trait Sealed {}
}

/// Same as [`ConvertArgument`], but for return types.
pub trait ConvertReturn: return_private::Sealed {
    /// The inner type that this can be converted to and from.
    #[doc(hidden)]
    type __Inner: EncodeReturn;

    #[doc(hidden)]
    fn __into_defined_return(self) -> Self::__Inner;

    #[doc(hidden)]
    fn __from_return(inner: Self::__Inner) -> Self;
}

impl<T: EncodeReturn> return_private::Sealed for T {}
impl<T: EncodeReturn> ConvertReturn for T {
    type __Inner = Self;

    #[inline]
    fn __into_defined_return(self) -> Self::__Inner {
        self
    }

    #[inline]
    fn __from_return(inner: Self::__Inner) -> Self {
        inner
    }
}

impl return_private::Sealed for bool {}
impl ConvertReturn for bool {
    type __Inner = Bool;

    #[inline]
    fn __into_defined_return(self) -> Self::__Inner {
        Bool::new(self)
    }

    #[inline]
    fn __from_return(inner: Self::__Inner) -> Self {
        inner.as_bool()
    }
}

pub trait ConvertArguments {
    #[doc(hidden)]
    type __Inner: EncodeArguments;

    #[doc(hidden)]
    type __WritebackOnDrop: Sized;

    #[doc(hidden)]
    unsafe fn __into_arguments(self) -> (Self::__Inner, Self::__WritebackOnDrop);
}

pub trait TupleExtender<T> {
    #[doc(hidden)]
    type PlusOneArgument;
    #[doc(hidden)]
    fn add_argument(self, arg: T) -> Self::PlusOneArgument;
}

macro_rules! args_impl {
    ($($a:ident: $t:ident),*) => (
        impl<$($t: ConvertArgument),*> ConvertArguments for ($($t,)*) {
            type __Inner = ($($t::__Inner,)*);

            type __WritebackOnDrop = ($($t::__WritebackOnDrop,)*);

            #[inline]
            unsafe fn __into_arguments(self) -> (Self::__Inner, Self::__WritebackOnDrop) {
                let ($($a,)*) = self;
                // SAFETY: Upheld by caller
                $(let $a = unsafe { ConvertArgument::__into_argument($a) };)*

                (($($a.0,)*), ($($a.1,)*))
            }
        }

        impl<$($t,)* T> TupleExtender<T> for ($($t,)*) {
            type PlusOneArgument = ($($t,)* T,);

            #[inline]
            fn add_argument(self, arg: T) -> Self::PlusOneArgument {
                let ($($a,)*) = self;
                ($($a,)* arg,)
            }
        }
    );
}

args_impl!();
args_impl!(a: A);
args_impl!(a: A, b: B);
args_impl!(a: A, b: B, c: C);
args_impl!(a: A, b: B, c: C, d: D);
args_impl!(a: A, b: B, c: C, d: D, e: E);
args_impl!(a: A, b: B, c: C, d: D, e: E, f: F);
args_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G);
args_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H);
args_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I);
args_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J);
args_impl!(
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
    g: G,
    h: H,
    i: I,
    j: J,
    k: K
);
args_impl!(
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
    g: G,
    h: H,
    i: I,
    j: J,
    k: K,
    l: L
);
args_impl!(
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
    g: G,
    h: H,
    i: I,
    j: J,
    k: K,
    l: L,
    m: M
);
args_impl!(
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
    g: G,
    h: H,
    i: I,
    j: J,
    k: K,
    l: L,
    m: M,
    n: N
);
args_impl!(
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
    g: G,
    h: H,
    i: I,
    j: J,
    k: K,
    l: L,
    m: M,
    n: N,
    o: O
);
args_impl!(
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
    g: G,
    h: H,
    i: I,
    j: J,
    k: K,
    l: L,
    m: M,
    n: N,
    o: O,
    p: P
);

#[cfg(test)]
mod tests {
    use super::*;

    use core::any::TypeId;

    #[test]
    fn convert_normally_noop() {
        assert_eq!(
            TypeId::of::<<i32 as ConvertArgument>::__Inner>(),
            TypeId::of::<i32>()
        );
        assert_eq!(<i32 as ConvertArgument>::__from_defined_param(42), 42);
        assert_eq!(unsafe { ConvertArgument::__into_argument(42i32).0 }, 42);
    }

    #[test]
    fn convert_i8() {
        assert_eq!(
            TypeId::of::<<i8 as ConvertArgument>::__Inner>(),
            TypeId::of::<i8>()
        );
        assert_eq!(<i8 as ConvertArgument>::__from_defined_param(-3), -3);
        assert_eq!(unsafe { ConvertArgument::__into_argument(-3i32).0 }, -3);
    }

    #[test]
    fn convert_bool() {
        assert!(!<bool as ConvertArgument>::__from_defined_param(Bool::NO));
        assert!(<bool as ConvertArgument>::__from_defined_param(Bool::YES));
        assert!(!<bool as ConvertReturn>::__from_return(Bool::NO));
        assert!(<bool as ConvertReturn>::__from_return(Bool::YES));

        assert!(!unsafe { ConvertArgument::__into_argument(false).0 }.as_bool());
        assert!(unsafe { ConvertArgument::__into_argument(true).0 }.as_bool());
        assert!(!ConvertReturn::__into_defined_return(false).as_bool());
        assert!(ConvertReturn::__into_defined_return(true).as_bool());

        #[cfg(all(target_vendor = "apple", target_os = "macos", target_arch = "x86_64"))]
        assert_eq!(
            <bool as ConvertArgument>::__Inner::ENCODING_ARGUMENT,
            crate::encode::Encoding::Char,
        );
    }
}
