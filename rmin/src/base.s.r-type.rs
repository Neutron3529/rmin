/// minimal low-level libR-sys.
#[path = "base.s.r-type.lib-r.rs"]
pub mod lib_r;
use super::Sexp;
#[cfg(doc)]
use super::{Owned, Protected, SExt};
use core::ffi::{c_char, c_int, c_void};
use lib_r::*;
pub use lib_r::{SEXP, SEXPTYPE}; // for doc generations
/// macros for define additional RType, for crate developer only.
pub mod macros {
    // pub macro RType(RType=$RType:tt SEXPTYPE=$SEXPTYPE:tt R_xlen_t=$R_xlen_t:tt Rf_allocVector=$Rf_allocVector:tt SEXP=$SEXP:tt, ty=$ty:tt Rty=$Rty:tt RCODE=$RCODE:tt=$RCODEVAL:tt){
    //     /// bind $ty with R $Rty type
    //     pub type $Rty=$ty;
    //     /// R $Rty type
    //     pub const $RCODE: $SEXPTYPE = $RCODEVAL;
    //     impl $RType for $Rty {
    //         const $SEXPTYPE:$SEXPTYPE=$RCODE;
    //     }
    // }
    /// simple wrapper for defining an arbitrary trait
    pub macro trait_def($(#[$m:meta])*; {$($vis:tt)*} $trait:tt $qual:tt $blk:tt for $macros:tt {$($extra:tt)*} $($t:tt)*){
        $(#[$m])*
        $($vis)* trait $trait : $qual
        $blk
        $macros!{$trait with {$($extra)*} for $($t)*}
        // $($($extra)* impl $trait for $t $tblk)*
    }
    /// wrapper for easily defining the RType implementations
    pub macro RTypeMatrix($trait:tt with {$SEXPTYPE:tt $alias:tt $define:tt $self:tt} for $($ty:ty $(: $tyalias:literal)?, $Rty:tt, $RCode:tt $(=$RCodeVal:tt)? {$($extraTrait:tt)*});*$(;)?) {
        $(
            // /// bind u8 with R Rchar type (read only!)
            // #[doc=core::concat!(" bind [`",core::stringify!($ty),"`] with R [`",core::stringify!($Rty),"`] type")]
            #[allow(non_camel_case_types)]
            // pub type Rchar = u8;
            #[doc=core::concat!("Data type that an R [`SEXP`] with [`stype`](SEXPext::stype)` == `[`",stringify! ($RCode),"`] "$(,"(",$RCodeVal,")")?," contains.")]
            #[doc=""]
            #[doc=core::concat!("Such [`SEXP`] has R type [`",core::stringify!($Rty), "`] and is defined into Rust [`",core::stringify!($ty) ,"`]",$($tyalias,)?" type")]
            pub type $Rty = $ty;

            crate::macros::cond_eval!{($($RCodeVal)?) use $self::$define::$RCode;}
            impl $trait for $Rty { #[doc=core::concat!("[`SEXP`] type of R [`",core::stringify!($Rty), "`] type is ",stringify!($RCode))] const $SEXPTYPE:$SEXPTYPE=$RCode; }
            $(impl $extraTrait for $Rty {})*
        )*
        /// R [`SEXPTYPE`] constants
        pub mod $define {
            #[cfg(doc)]
            use $self::$alias::*;
            $($(
                #[doc=core::concat!(" R [`",core::stringify!($Rty),"`] type")] pub const $RCode: $self::$SEXPTYPE = $RCodeVal;
            )?)*
        }
        /// R type aliases
        pub mod $alias {
            $(
                #[doc(inline)] pub use $self::$Rty;
            )*
        }
    }
}
use macros::{trait_def, RTypeMatrix};
trait_def!(
/// Basic R data storage type
///
/// Since it is defined behind a macro invocation, thus it cannot be touched.
/// Currently, RType is defined for 4 types:
///
/// [`f64`] (R [`numeric`]), [`i32`] (R [`integer`]), [`u32`] (R [`logical`]) and [`u8`] (R [`Rchar`])
///
/// [See the full supported list here](#implementors)
;
{ pub }
RType
Copy
{
    /// the real SEXPTYPE of the data,
    ///
    /// Only a limited part of SEXPTYPE is supported, see [Implementors](#implementors) part.
    ///
    const SEXPTYPE:SEXPTYPE;
    /// indicate the underlying data type.
    type Data:Copy=Self; // For more custom type in case collision happened.
    // /// parameter that should be sent to `new`. Currently [`R_xlen_t`] for things other than Rchar.
    // /// For Rchar, a [`String`] is required
    /// Allocate a new [`SEXP`] object with default value of length `len`
    ///
    /// SAFETY: You should adding additional marks such as
    ///
    /// [`Owned<T>`]`{`[`sexp`](Owned)`:`[`SEXP`]`, `[`_marker`](Owned)`:[]}`
    ///
    /// or
    ///
    /// [`Protected<T>`]`{`[`sexp`](Protected)`:`[`SEXP`]`.`[`protect`]()`(), `[`_marker`](Protected)`:[]}`
    ///
    /// it requires visiting the private constructor or a dangerous transmute (you should never transmute [`SEXP`] into [`Protected<T>`]).
    ///
    /// Actually, since the re-export is behind a macro, you might never access this crate.
    unsafe fn new(len: usize) -> SEXP where Self : RDefault {
        // SAFETY: marked the SEXP as either Owned or Protected, thus is safe.
        unsafe {<Self as RDefault>::new(len as R_xlen_t)}
    }
    /// Create a R copy from Rust object.
    ///
    /// SAFETY: You should adding additional marks such as
    ///
    /// [`Owned<T>`]`{`[`sexp`](Owned)`:`[`SEXP`]`, `[`_marker`](Owned)`:[]}`
    ///
    /// or
    ///
    /// [`Protected<T>`]`{`[`sexp`](Protected)`:`[`SEXP`]`.`[`protect`]()`(), `[`_marker`](Protected)`:[]}`
    ///
    /// it requires visiting the private constructor or a dangerous transmute (you should never transmute [`SEXP`] into [`Protected<T>`]).
    ///
    /// Actually, since the re-export is behind a macro, you might never access this crate.
    #[inline(always)]
    unsafe fn from(s:impl core::convert::AsRef<[<Self as RType>::Data]>) -> SEXP where Self:RTypeFrom {
        unsafe {<Self as RTypeFrom>::from(s)}
    }
    /// Get the length of a vector.
    #[inline(always)]
    fn is_type(s:SEXP) -> bool {
        // SAFETY: ffi.
        unsafe { TYPEOF(s) == Self::SEXPTYPE }
    }
    /// Get the data pointer of a vector.
    /// although get a pointer should be safe, I marked it as an unsafe function
    /// since you must take care about what you want to do.
    #[inline(always)]
    unsafe fn data(s:SEXP) -> *const Self::Data {
        unsafe { DATAPTR_RO(s) as *const Self::Data }
    }
    /// Get the mutable data pointer of a vector.
    /// although get a pointer should be safe, I marked it as an unsafe function
    /// since you must take care about what you want to do.
    #[inline(always)]
    unsafe fn data_mut(s:SEXP) -> *mut Self::Data where Self:RTypeMut {
        unsafe { DATAPTR(s) as *mut Self::Data }
    }
}
for RTypeMatrix { SEXPTYPE alias define self }
    ():"(unit)", NULL, NILSXP=0 { };
    u8, Rchar, CHARSXP=9 { };
    u32, logical, LGLSXP =10 {RDefault RTypeMut};
    i32, integer, INTSXP =13 {RDefault RTypeMut};
    f64, numeric, REALSXP=14 {RDefault RTypeMut};
    SEXP, list, VECSXP=19 {RDefault RTypeMut};
    *mut c_void, externalptr, EXTPTRSXP=22 { };
    Sexp<numeric>:"(Sexp)", numeric_list, VECSXP {RDefault RTypeMut};
    Sexp<integer>:"(Sexp)", integer_list, VECSXP {RDefault RTypeMut};
    Sexp<logical>:"(Sexp)", logical_list, VECSXP {RDefault RTypeMut};
    Sexp<Rchar>, character, STRSXP=16 {RDefault RTypeMut};
    // Sexp<list>:"(Sexp)", list_list, VECSXP {RDefault RTypeMut};
);

/// Indicate whether a [`RType`] could be allocate by default
pub trait RDefault: RType {
    /// create an unprotected allocated [`SEXP`] with length `len`
    ///
    /// should be unsafe since a mark is need to indicate whether it is protected.
    unsafe fn new(len: R_xlen_t) -> SEXP {
        // SAFETY: FFI call.
        unsafe { Rf_allocVector(<Self as RType>::SEXPTYPE, len) }
    }
}
impl RDefault for NULL {
    /// Note: the new method with [`NULL`] type only yields a vector with length 0, no memory allocation occurs.
    unsafe fn new(_len: R_xlen_t) -> SEXP {
        R_NilValue
    }
}
/// Indicate whether a [`RType`] could be mutablly indexed.
pub trait RTypeMut: RType {}
/// Indicate whether a [`RType`] could be converted from a [`&[Rust]`](core::slice) type.
pub trait RTypeFrom: RType {
    /// create a R copy from rust object.
    unsafe fn from(data: impl core::convert::AsRef<[<Self as RType>::Data]>) -> SEXP;
}
impl<T: RDefault + RTypeMut> RTypeFrom for T {
    #[inline(always)]
    unsafe fn from(data: impl core::convert::AsRef<[<Self as RType>::Data]>) -> SEXP {
        let data = data.as_ref();
        let (src, len) = (data.as_ptr(), data.len());
        unsafe {
            // SAFETY: Since wrapping it is still needed, thus it is safe.
            let sexp = <T as RType>::new(data.len());
            // SAFETY:
            // get mut pointer to perform
            let dst = <T as RType>::data_mut(sexp);
            // SAFETY:
            // src must be valid for reads of count * size_of::<T>() bytes.
            // dst must be valid for writes of count * size_of::<T>() bytes.
            // Both src and dst must be properly aligned.
            // The region of memory beginning at src with a size of count * size_of::<T>() bytes must not overlap with the region of memory beginning at dst with the same size.
            // T:Copy, thus it cannot violate memory safety
            core::intrinsics::copy_nonoverlapping(src, dst, len);
            sexp
        }
    }
}
/// allocate a owned R string object from rust String.
impl RTypeFrom for Rchar {
    #[inline(always)]
    unsafe fn from(data: impl core::convert::AsRef<[<Self as RType>::Data]>) -> SEXP {
        let data = data.as_ref();
        // SAFETY: ffi call.
        unsafe {
            Rf_mkCharLenCE(
                data.as_ptr() as *const c_char,
                data.len() as c_int,
                cetype_t_CE_UTF8,
            )
        }
    }
}
/// unsafe trait for [`SEXP`]
pub trait SEXPext: Copy {
    /// Protect a [`SEXP`] variable
    ///
    /// It is a really dangerous function that you should not use it at all.
    ///
    /// If you insist to call this function, file an issue and tell me why.
    /// In most cases, [`Owned<T>`] and [`Protected<T>`] is enough.
    ///
    /// SAFETY: in case the type is a field of [`Owned<T>`], the result will wrapping in a [`Protected<T>`], then it could be safe.
    ///
    /// Otherwise, use at your own risk.
    unsafe fn protect(self) -> Self;

    /// Unprotect a [`SEXP`] variable
    ///
    /// It is a really dangerous function that you should not use it at all.
    ///
    /// If you insist to call this function, file an issue and tell me why.
    /// In most cases, [`Owned<T>`] and [`Protected<T>`] is enough.
    ///
    /// SAFETY: in case the type is a field of [`Protected<T>`], the result will wrapping in a non-[`Protected<T>`] field, then it could be safe.
    ///
    /// Otherwise, use at your own risk.
    unsafe fn unprotect(self);

    /// get stype
    /// SAFETY: this function should be safe, but the previous function `*.as_sexp()` could be unsafe.
    /// Mark it as unsafe since using `[SExt::stype]` might be better.
    unsafe fn stype(self) -> SEXPTYPE;

    /// get length of a SEXP type.
    /// SAFETY: this function should be safe, but the previous function `*.as_sexp()` could be unsafe.
    /// Mark it as unsafe since using `[SExt::len]` might be better.
    unsafe fn len(self) -> R_xlen_t;

    /// get whether self is missing.
    /// it seems that, return 0 means the value is not missing, but I'm not sure.
    /// It is really unsafe, the best practice is that, never rely on this feature.
    unsafe fn missing(self) -> c_int;

    /// get attr from SEXP
    unsafe fn get_attr(self, attr: SEXP) -> SEXP;

    /// get attr from SEXP
    unsafe fn set_attr(self, attr: SEXP, val: SEXP) -> SEXP;
}
impl SEXPext for SEXP {
    #[inline(always)]
    unsafe fn protect(self) -> Self {
        // SAFETY: FFi calls.
        unsafe { Rf_protect(self) }
    }
    #[inline(always)]
    unsafe fn unprotect(self) {
        // SAFETY: FFi calls.
        unsafe { Rf_unprotect_ptr(self) }
    }
    #[inline(always)]
    unsafe fn stype(self) -> SEXPTYPE {
        // SAFETY: FFi calls.
        unsafe { TYPEOF(self) }
    }
    #[inline(always)]
    unsafe fn len(self) -> R_xlen_t {
        unsafe { Rf_xlength(self) }
    }
    /// REALLY UNSAFE
    #[inline(always)]
    unsafe fn missing(self) -> c_int {
        unsafe { MISSING(self) }
    }
    #[inline(always)]
    unsafe fn get_attr(self, slot: SEXP) -> SEXP {
        unsafe { Rf_getAttrib(self, slot) }
    }
    #[inline(always)]
    unsafe fn set_attr(self, slot: SEXP, val: SEXP) -> SEXP {
        unsafe { Rf_setAttrib(self, slot, val) }
    }
}

/// REALLY UNSAFE FUNCTION
///
/// YOU SHOULD NOT USE IT AT ALL.
///
/// UNLESS YOU ARE AN EXPERT OF BOTH R AND RUST.
///
/// In case you want to issue an error, just use the [`panic`] macro is enough.
/// This really dangerous function is used for implementing a new panic handler.
/// When you really need this function, considering writting a new lib might be better.
///
/// SAFETY: This function contains a longjmp, thus calling it directly might involve a memory leak.
/// You could only use it after R executed all deconstructors.
/// To be specific, at the end of an FFI call, after call drop
/// to anything you could call.
/// Otherwise, memory leak may happens by the longjmp instruction in the R internal `Rf_error` function.
pub unsafe fn error(message: *const Rchar) -> ! {
    unsafe { Rf_errorcall(lib_r::R_CurrentExpression, FMT, message as *const c_char) }
}
/// print function.
pub unsafe fn print(message: *const Rchar) {
    // SAFETY: FMT is add to ensure the code is well-encoded
    unsafe { Rprintf(FMT, message as *const c_char) }
}
/// eprint function.
pub unsafe fn eprint(message: *const Rchar) {
    // SAFETY: FMT is add to ensure the code is well-encoded
    unsafe { REprintf(FMT, message as *const c_char) }
}

/// formatter avoid %s being translated.
pub const FMT: *const c_char = c"%s".as_ptr();
