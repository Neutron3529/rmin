#![allow(missing_docs, non_camel_case_types, dead_code)]
use core::ffi::{c_char, c_int, c_uint, c_void};
type size_t = usize;

#[doc = "R_xlen_t is defined as int on 32-bit platforms, and\n that confuses Rust. Keeping it always as ptrdiff_t works\n fine even on 32-bit.\n <div rustbindgen replaces=\"R_xlen_t\"></div>"]
pub type R_xlen_t = isize;

/// simplified [`DL_FUNC`] definations, not compatitable with libR-sys.
///
/// Usage:
/// ```
/// // using &[R_CallMethodDef] thus no need to count how much method is added.
/// const R_CALL_METHOD:&[rmin::reg::R_CallMethodDef]=&[
///     rmin::reg::R_CallMethodDef {name:"wrapped_foo".as_ptr() as *const c_char, fun:Some(wrapped_foo as DL_FUNC_INNER), numArgs:0}
///     rmin::reg::R_CallMethodDef {name:core::ptr::null(), fun:None, numArgs:0}
/// ];
/// ```
#[cfg_attr(doc, doc(cfg(feature = "register-routines")))]
#[cfg(any(doc, feature = "register-routines"))]
pub type DL_FUNC = *const c_void;

#[repr(C)]
#[cfg_attr(doc, doc(cfg(feature = "register-routines")))]
#[cfg(any(doc, feature = "register-routines"))]
pub struct R_CallMethodDef {
    pub name: *const c_char,
    pub fun: DL_FUNC,
    pub numArgs: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SEXPREC([u8; 0]);

#[repr(C)]
pub struct DllInfo([u8; 0]);
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
///     Rchar:Sexp<Rchar>, // for R char type (not character type)
/// ) -> Owned<NULL> {todo!()}
/// ```
pub type SEXP = core::ptr::NonNull<*mut SEXPREC>;
#[doc = "NOT YET using enum:\n  1)\tThe internal SEXPREC struct has 'SEXPTYPE type : 5'\n\t(making FUNSXP and CLOSXP equivalent in there),\n\tgiving (-Wall only ?) warnings all over the place\n 2)\tMany switch(type) { case ... } statements need a final `default:'\n\tadded in order to avoid warnings like [e.g. l.170 of ../main/util.c]\n\t  \"enumeration value `FUNSXP' not handled in switch\""]
pub type SEXPTYPE = c_uint;

pub type cetype_t = u32;
#[allow(non_upper_case_globals)]
pub const cetype_t_CE_UTF8: u32 = 1;
// pub type Rboolean = u32;
#[link(name = "R", kind = "dylib")]
extern "C-unwind" {
    pub fn Rf_errorcall(call: SEXP, error: *const c_char, ...) -> !; // avoid the possible copy.
                                                                     // pub fn Rf_error(error: *const c_char, ...) -> !;
    pub fn Rf_mkCharLenCE(data: *const c_char, len: c_int, enc: cetype_t) -> SEXP; // unwind while data contains '\0'
}
#[link(name = "R", kind = "dylib")]
extern "C" {
    /// for register-routines feature.
    #[cfg_attr(doc, doc(cfg(feature = "register-routines")))]
    #[cfg(any(doc, feature = "register-routines"))]
    pub fn R_registerRoutines(
        info: *mut DllInfo,
        croutines: *const c_void,
        callRoutines: *const R_CallMethodDef,
        fortranRoutines: *const c_void,
        externalRoutines: *const c_void,
    ) -> c_int;
    #[cfg_attr(doc, doc(cfg(feature = "register-routines")))]
    #[cfg(any(doc, feature = "register-routines"))]
    pub fn R_useDynamicSymbols(dll: *mut DllInfo, flag: c_uint) -> c_uint;
    #[cfg_attr(doc, doc(cfg(feature = "register-routines")))]
    #[cfg(any(doc, feature = "register-routines"))]
    pub fn R_forceSymbols(dll: *mut DllInfo, flag: c_uint) -> c_uint;
    pub static mut R_CurrentExpression: SEXP;
    #[doc = "These are the public inlinable functions that are provided in\nRinlinedfuns.h It is *essential* that these do not appear in any\nother header file, with or without the Rf_ prefix."]
    pub fn Rf_allocVector(arg1: SEXPTYPE, arg2: R_xlen_t) -> SEXP;
    pub fn Rf_protect(arg1: SEXP) -> SEXP;
    /// unprotect, only for new protection.
    /// head <-> *here* <-> tail
    /// head -> tag1 -> tag2 <-> tail
    ///          |       |
    ///          V       V
    ///    protected    pseudo
    pub fn Rf_unprotect(arg1: c_int);
    pub fn Rf_unprotect_ptr(arg1: SEXP);
    // #[cfg_attr(doc, doc(cfg(not(have_std))))] #[cfg(not(have_std))]
    // pub fn R_chk_calloc(count: usize, size_and_align: usize) -> *mut c_void;
    // #[cfg_attr(doc, doc(cfg(not(have_std))))] #[cfg(not(have_std))]
    // pub fn R_chk_realloc(ptr: *mut c_void, new_size: usize) -> *mut c_void;
    // #[cfg_attr(doc, doc(cfg(not(have_std))))] #[cfg(not(have_std))]
    // pub fn R_chk_free(ptr: *mut c_void);
    pub fn Rf_xlength(arg1: SEXP) -> R_xlen_t;
    pub fn MISSING(x: SEXP) -> c_int;
    pub fn DATAPTR_RO(x: SEXP) -> *const c_void;
    pub fn DATAPTR(x: SEXP) -> *mut c_void;
    pub fn TYPEOF(x: SEXP) -> SEXPTYPE;
    pub static mut R_NilValue: SEXP;
    pub static mut R_ClassSymbol: SEXP;
    pub fn R_MakeExternalPtr(p: *mut c_void, tag: SEXP, prot: SEXP) -> SEXP;
    pub fn R_ExternalPtrTag(s: SEXP) -> SEXP;
    pub fn R_ExternalPtrAddr(s: SEXP) -> *mut c_void;
    pub fn Rf_getAttrib(item: SEXP, slot: SEXP) -> SEXP;
    pub fn Rf_setAttrib(item: SEXP, slot: SEXP, val: SEXP) -> SEXP;
    pub fn R_RegisterCFinalizer(s: SEXP, fun: extern "C" fn(SEXP) -> ());
    pub fn Rprintf(arg1: *const c_char, ...);
    pub fn REprintf(arg1: *const c_char, ...);
}

#[cfg_attr(doc, doc(cfg(not(have_std))))]
#[cfg(not(have_std))]
extern "C" {
    pub fn malloc(size: size_t) -> *mut c_void;
    pub fn calloc(nobj: size_t, size: size_t) -> *mut c_void;
    pub fn realloc(p: *mut c_void, size: size_t) -> *mut c_void;
    pub fn free(p: *mut c_void);
    pub fn posix_memalign(memptr: *mut *mut c_void, align: size_t, size: size_t) -> c_int;
}
