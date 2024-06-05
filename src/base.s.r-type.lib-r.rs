#![allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SEXPREC([u8; 0]);
/// Raw SEXP expression, should not be used.
///
/// Since it is decleared under a hygiene macro, you cannot access it.
///
/// # Example
/// ```no_run
/// // using rmin::SEXP instead.
/// use rmin::{Owned, SEXP};
/// #[no_mangle] // do not forget this attribute
///              // otherwise you cannot call this function from R.
/// extern fn entry (
///     numeric:Sexp<numeric>, // for R numeric type
///     integer:Sexp<integer>, // for R integer type
///     logical:Sexp<logical>, // for R logical type
///     character:Sexp<character>, // for R character type
/// ) -> Owned<NULL> {todo!()}
/// ```
pub type SEXP = *mut SEXPREC;
#[doc = "NOT YET using enum:\n  1)\tThe internal SEXPREC struct has 'SEXPTYPE type : 5'\n\t(making FUNSXP and CLOSXP equivalent in there),\n\tgiving (-Wall only ?) warnings all over the place\n 2)\tMany switch(type) { case ... } statements need a final `default:'\n\tadded in order to avoid warnings like [e.g. l.170 of ../main/util.c]\n\t  \"enumeration value `FUNSXP' not handled in switch\""]
pub type SEXPTYPE = core::ffi::c_uint;
#[allow(non_camel_case_types)]
pub type cetype_t = u32;
#[allow(non_upper_case_globals)]
pub const cetype_t_CE_UTF8: u32 = 1;
// pub type Rboolean = u32;
#[link(name = "R", kind = "dylib")]
extern "C-unwind" {
    // fn Rf_errorcall(call:SEXP, error: *const core::ffi::c_char, ...) -> !; // avoid the possible copy.
    pub fn Rf_error(error: *const core::ffi::c_char) -> !;
    pub fn Rf_mkCharLenCE(
        data: *const core::ffi::c_char,
        len: core::ffi::c_int,
        enc: cetype_t,
    ) -> SEXP; // unwind while data contains '\0'
}
#[link(name = "R", kind = "dylib")]
extern "C" {
    // pub static mut R_CurrentExpression: SEXP;
    #[doc = "These are the public inlinable functions that are provided in\nRinlinedfuns.h It is *essential* that these do not appear in any\nother header file, with or without the Rf_ prefix."]
    pub fn Rf_allocVector(arg1: SEXPTYPE, arg2: R_xlen_t) -> SEXP;
    pub fn Rf_protect(arg1: SEXP) -> SEXP;
    pub fn Rf_unprotect_ptr(arg1: SEXP);
    #[cfg_attr(doc, doc(cfg(not(feature = "std"))))] #[cfg(not(feature = "std"))]
    pub fn R_chk_calloc(count: usize, size_and_align: usize) -> *mut core::ffi::c_void;
    #[cfg_attr(doc, doc(cfg(not(feature = "std"))))] #[cfg(not(feature = "std"))]
    pub fn R_chk_realloc(ptr: *mut core::ffi::c_void, new_size: usize) -> *mut core::ffi::c_void;
    #[cfg_attr(doc, doc(cfg(not(feature = "std"))))] #[cfg(not(feature = "std"))]
    pub fn R_chk_free(ptr: *mut core::ffi::c_void);
    #[cfg_attr(doc, doc(cfg(not(feature = "std"))))] #[cfg(not(feature = "std"))]
    pub fn Rprintf(arg1: *const core::ffi::c_char, ...);
    pub fn Rf_xlength(arg1: SEXP) -> R_xlen_t;
    pub fn DATAPTR_RO(x: SEXP) -> *const core::ffi::c_void;
    pub fn DATAPTR(x: SEXP) -> *mut core::ffi::c_void;
    pub fn TYPEOF(x: SEXP) -> SEXPTYPE;
    pub static mut R_NilValue:SEXP;
    // pub fn Rf_isReal(x: SEXP) -> Rboolean;
    // pub fn Rf_isLogical(x: SEXP) -> Rboolean;
    // pub fn Rf_isInteger(x: SEXP) -> Rboolean;
}
// pub fn Rf_error(error: *const core::ffi::c_char)->!{
//     unsafe { Rf_errorcall(R_CurrentExpression,error) }
// }

#[allow(non_camel_case_types)]
#[doc = "R_xlen_t is defined as int on 32-bit platforms, and\n that confuses Rust. Keeping it always as ptrdiff_t works\n fine even on 32-bit.\n <div rustbindgen replaces=\"R_xlen_t\"></div>"]
pub type R_xlen_t = isize;
