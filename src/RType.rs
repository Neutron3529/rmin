use crate::libR::*;
use core::ffi::c_char;
/// get length of a SEXP type.
#[inline(always)]
pub fn len(s:SEXP) -> R_xlen_t {
    unsafe { Rf_xlength(s) }
}
pub trait RType:Copy {
    const SEXPTYPE:SEXPTYPE;
    type Data:Copy=Self; // For more custom type in case collision happened.
    type New;
    /// allocate a owned new vector with given length.
    /// for charater, yield a new R character object from a give &str.
    fn new(len: Self::New) -> SEXP;
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
/// SAFETY: need to unsure the underlying type is modifyable.
pub unsafe trait RTypeMut:RType {}

#[allow(non_camel_case_types)]
pub type logical=u32;
#[allow(non_camel_case_types)]
pub type integer=i32;
#[allow(non_camel_case_types)]
pub type numeric=f64;
#[allow(non_camel_case_types)]
pub type character=u8;

pub const CHARSXP: SEXPTYPE = 9;
pub const LGLSXP: SEXPTYPE = 10;
pub const INTSXP: SEXPTYPE = 13;
pub const REALSXP: SEXPTYPE = 14;

impl RType for character {
    const SEXPTYPE:SEXPTYPE=CHARSXP;
    type New=String;
    /// allocate a owned R string object from rust String.
    #[inline(always)]
    fn new(s: Self::New) -> SEXP {
        let s:&str=s.as_ref();
        // SAFETY: ffi call.
        unsafe { Rf_mkCharLenCE(
            s.as_ptr() as *const c_char,
            s.len() as i32,
            cetype_t_CE_UTF8,)
        }
    }
}
impl RType for logical {
    const SEXPTYPE:SEXPTYPE=LGLSXP;
    type New=R_xlen_t;
    fn new(len: Self::New) -> SEXP {
        // SAFETY: ffi call.
        unsafe { Rf_allocVector(Self::SEXPTYPE, len) }
    }
}
impl RType for integer {
    const SEXPTYPE:SEXPTYPE=INTSXP;
    type New=R_xlen_t;
    fn new(len: Self::New) -> SEXP {
        // SAFETY: ffi call.
        unsafe { Rf_allocVector(Self::SEXPTYPE, len) }
    }
}
impl RType for numeric {
    const SEXPTYPE:SEXPTYPE=REALSXP;
    type New=R_xlen_t;
    fn new(len: Self::New) -> SEXP {
        // SAFETY: ffi call.
        unsafe { Rf_allocVector(Self::SEXPTYPE, len) }
    }
}
unsafe impl RTypeMut for logical{}
unsafe impl RTypeMut for integer{}
unsafe impl RTypeMut for numeric{}