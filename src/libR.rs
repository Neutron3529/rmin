#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SEXPREC([u8; 0]);
pub type SEXP = *mut SEXPREC;
#[doc = "NOT YET using enum:\n  1)\tThe internal SEXPREC struct has 'SEXPTYPE type : 5'\n\t(making FUNSXP and CLOSXP equivalent in there),\n\tgiving (-Wall only ?) warnings all over the place\n 2)\tMany switch(type) { case ... } statements need a final `default:'\n\tadded in order to avoid warnings like [e.g. l.170 of ../main/util.c]\n\t  \"enumeration value `FUNSXP' not handled in switch\""]
pub type SEXPTYPE = core::ffi::c_uint;
pub type Rboolean = u32;
extern "C" {
    #[doc = "These are the public inlinable functions that are provided in\nRinlinedfuns.h It is *essential* that these do not appear in any\nother header file, with or without the Rf_ prefix."]
    pub fn Rf_allocVector(arg1: SEXPTYPE, arg2: R_xlen_t) -> SEXP;
    pub fn Rf_protect(arg1: SEXP) -> SEXP;
    pub fn Rf_unprotect_ptr(arg1: SEXP);
    pub fn R_chk_calloc(count: usize, size_and_align: usize) -> *mut core::ffi::c_void;
    pub fn R_chk_realloc(ptr: *mut core::ffi::c_void, new_size: usize) -> *mut core::ffi::c_void;
    pub fn R_chk_free(ptr: *mut core::ffi::c_void);
    pub fn Rprintf(arg1: *const core::ffi::c_char, ...);
    pub fn Rf_error(arg1: *const core::ffi::c_char, ...) -> !;
    pub fn Rf_xlength(arg1: SEXP) -> R_xlen_t;
    pub fn DATAPTR_RO(x: SEXP) -> *const core::ffi::c_void;
    pub fn DATAPTR(x: SEXP) -> *mut core::ffi::c_void;
    pub fn Rf_isReal(x: SEXP) -> Rboolean;
    pub fn Rf_isLogical(x: SEXP) -> Rboolean;
    pub fn Rf_isInteger(x: SEXP) -> Rboolean;
}

pub const LGLSXP: SEXPTYPE = 10;
pub const INTSXP: SEXPTYPE = 13;
pub const REALSXP: SEXPTYPE = 14;
#[allow(non_camel_case_types)]
#[doc = "R_xlen_t is defined as int on 32-bit platforms, and\n that confuses Rust. Keeping it always as ptrdiff_t works\n fine even on 32-bit.\n <div rustbindgen replaces=\"R_xlen_t\"></div>"]
pub type R_xlen_t = isize;
