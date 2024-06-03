/// minimal low-level libR-sys.
pub mod libR {
    pub use crate::libR::*;
}
use libR::*;
use core::ffi::c_char;
/// get length of a SEXP type.
#[inline(always)]
pub fn len(s: SEXP) -> R_xlen_t {
    unsafe { Rf_xlength(s) }
}

mod macros {
    // pub macro RType(RType=$RType:tt SEXPTYPE=$SEXPTYPE:tt R_xlen_t=$R_xlen_t:tt Rf_allocVector=$Rf_allocVector:tt SEXP=$SEXP:tt, ty=$ty:tt Rty=$Rty:tt RCODE=$RCODE:tt=$RCODEVAL:tt){
    //     /// bind $ty with R $Rty type
    //     #[allow(non_camel_case_types)]
    //     pub type $Rty=$ty;
    //     /// R $Rty type
    //     pub const $RCODE: $SEXPTYPE = $RCODEVAL;
    //     impl $RType for $Rty {
    //         const $SEXPTYPE:$SEXPTYPE=$RCODE;
    //     }
    // }
    pub macro trait_def($(#[$m:meta])*; {$($vis:tt)*} $trait:tt $qual:tt $blk:tt for $macros:tt {$($extra:tt)*} $($t:tt)*){
        $(#[$m])*
        $($vis)* trait $trait : $qual
        $blk
        $macros!{$trait with {$($extra)*} for $($t)*}
        // $($($extra)* impl $trait for $t $tblk)*
    }
    pub macro RTypeMatrix($trait:tt with {$SEXPTYPE:tt} for $($ty:ty, $Rty:tt, $RCode:tt $RCodeVal:tt {$($extraTrait:tt)*});*$(;)?) {$(
        // /// bind u8 with R character type (read only!)
        #[doc=core::concat!(" bind [`",core::stringify!($ty),"`] with R [`",core::stringify!($Rty),"`] type")]
        // #[allow(non_camel_case_types)]
        #[allow(non_camel_case_types)]
        // pub type character = u8;
        #[doc=core::concat!(" R [`SEXP`](crate::libR::SEXP) with type ",$RCodeVal," has R type [`",core::stringify!($Rty), "`] and is defined into [`",core::stringify!($ty) ,"`]")]
        pub type $Rty = $ty;
        // character { const SEXPTYPE:SEXPTYPE=CHARSXP; } { #[doc = " R character type"] pub const CHARSXP: SEXPTYPE = 9; }
        #[doc=core::concat!(" R [`",core::stringify!($Rty),"`] type")] pub const $RCode: $SEXPTYPE = $RCodeVal;
        impl $trait for $Rty { #[doc=core::concat!("[`SEXP`](crate::libR::SEXP) type of R [`",core::stringify!($Rty), "`] type is ",$RCodeVal)] const $SEXPTYPE:$SEXPTYPE=$RCode; }
        $(impl $extraTrait for $Rty {})*
    )*}
}
use macros::{RTypeMatrix, trait_def};
trait_def!(
/// Basic R data type
///
/// Since it is defined behind a macro invocation, thus it cannot be touched.
/// Currently, RType is defined for 4 types, `f64` (R double), `i32` (R integer), `u32` (R logical) and `u8` (R character)
/// May add further implementations.
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
    // /// parameter that should be sent to `new`. Currently [`R_xlen_t`](crate::doc::R_xlen_t) for things other than character.
    // /// For character, a [`String`] is required
    /// Allocate a new [`SEXP`] object with default value of length `len`
    ///
    /// SAFETY: You should adding additional marker such as `[Owned<T>](crate::Owned){sexp:[SEXP], _marker:[]}` or `[Protected<T>](crate::Protected){sexp:[SEXP], _marker:[]}`
    /// Both method require visit the private constructor or a dangerous transmute (you should never transmute [`SEXP`] into [`Protected<T>`](crate::S::Protected)).
    /// Actually, since the re-export is behind a macro, you might never access this crate.
    /// # Example
    /// ```compile_fail
    /// use crate::RType::RType;
    /// ```
    unsafe fn new(len: usize) -> SEXP where Self : RDefault {
        // SAFETY: marked the SEXP as either Owned or Protected, thus is safe.
        unsafe {<Self as RDefault>::new(len as R_xlen_t)}
    }
    /// Create a R copy from rust object.
    ///
    /// Although get a pointer should be safe, I marked it as an unsafe function
    /// Since you must mark the correct type (and protect it later).
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
for RTypeMatrix { SEXPTYPE }
    u8, character, CHARSXP 9 { };
    u32, logical, LGLSXP  10 {RDefault RTypeMut};
    i32, integer, INTSXP  13 {RDefault RTypeMut};
    f64, numeric, REALSXP 14 {RDefault RTypeMut};
    SEXP, list, VECSXP 19 {RDefault RTypeMut};
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
/// Indicate whether a [`RType`] could be mutablly indexed.
pub trait RTypeMut: RType {}
/// Indicate whether a [`RType`] could be converted from a [`&[Rust]`](core::slice) type.
pub trait RTypeFrom:RType {
    /// create a R copy from rust object.
    unsafe fn from(data: impl core::convert::AsRef<[<Self as RType>::Data]>)->SEXP;
}
impl<T:RDefault + RTypeMut> RTypeFrom for T {
    #[inline(always)]
    unsafe fn from(data: impl core::convert::AsRef<[<Self as RType>::Data]>)->SEXP {
        let data = data.as_ref();
        let (src, len) = (data.as_ptr(), data.len());
        unsafe {
            // SAFETY: Since wrapping it is still needed, thus it is safe.
            let sexp=<T as RType>::new(data.len());
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
impl RTypeFrom for character {
    #[inline(always)]
    unsafe fn from(data: impl core::convert::AsRef<[<Self as RType>::Data]>)->SEXP {
        let data = data.as_ref();
        // SAFETY: ffi call.
        unsafe { Rf_mkCharLenCE(
            data.as_ptr() as *const c_char,
            data.len() as core::ffi::c_int,
            cetype_t_CE_UTF8
        )}
    }
}
