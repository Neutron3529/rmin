#![cfg_attr(not(feature = "std"), no_std)]
#![allow(internal_features)]
#![feature(lang_items)]
#![feature(panic_info_message)]

macro_rules! syntax_group {
    ($($tt:tt)*) => { $($tt)* };
}
use core::{ffi::c_void, slice};
#[allow(non_snake_case)]
mod libR;
use libR::{
    R_xlen_t as int, Rf_allocVector, Rf_isInteger, Rf_isLogical, Rf_isReal, Rf_protect,
    Rf_unprotect_ptr, Rf_xlength, Rprintf, DATAPTR, DATAPTR_RO, INTSXP, LGLSXP, REALSXP, SEXP, SEXPTYPE
};
#[cfg(not(feature = "std"))]
syntax_group!{
    #[lang = "eh_personality"]
    pub extern "C" fn rust_eh_personality() {}
    macro_rules! println{
            ($($tt:tt)*) => {
            #[allow(unused_unsafe)]
            unsafe{
                let mut x=alloc::string::String::new();
                core::fmt::write(&mut x, format_args!($($tt)*)).and_then(|_|core::fmt::write(&mut x, format_args!("\n\0"))).expect("failed to write string");
                Rprintf(x.as_ptr() as *const core::ffi::c_char);
            }}
        }
    extern crate alloc;
    use libR::{R_chk_calloc, R_chk_free, R_chk_realloc, Rf_error};
    struct SimpleAllocator();
    #[global_allocator]
    static ALLOCATOR: SimpleAllocator = SimpleAllocator();
    unsafe impl Sync for SimpleAllocator {}
    unsafe impl alloc::alloc::GlobalAlloc for SimpleAllocator {
        unsafe fn alloc(&self, layout: alloc::alloc::Layout) -> *mut u8 {
            unsafe { R_chk_calloc(layout.size().div_euclid(layout.align()), layout.align()) as *mut u8 }
        }
        unsafe fn alloc_zeroed(&self, layout: alloc::alloc::Layout) -> *mut u8 {
            unsafe { R_chk_calloc(layout.size().div_euclid(layout.align()), layout.align()) as *mut u8 }
        }
        unsafe fn dealloc(&self, ptr: *mut u8, _layout: alloc::alloc::Layout) {
            unsafe { R_chk_free(ptr as *mut core::ffi::c_void) }
        }
        unsafe fn realloc(
            &self,
            ptr: *mut u8,
            _layout: alloc::alloc::Layout,
            new_size: usize,
        ) -> *mut u8 {
            unsafe { R_chk_realloc(ptr as *mut core::ffi::c_void, new_size) as *mut u8 }
        }
    }
    use alloc::string::String;
    #[panic_handler]
    fn panic_handler(info: &core::panic::PanicInfo) -> ! {
        let mut x = String::new();

        if let Some(s) = info.payload().downcast_ref::<&str>() {
            println!("panic occurred: {s:?}");
        } else {
            println!("panic occurred");
        }

        info.message().map(|i| {
            core::fmt::write(&mut x, *i)
                .and_then(|_| core::fmt::write(&mut x, format_args!("\n")))
                .expect("failed to write string")
        });
        info.location().map(|i| {
            core::fmt::write(&mut x, format_args!("{:?}\n\0", i)).expect("failed to write string")
        });
        unsafe {
            Rprintf(x.as_ptr() as *const core::ffi::c_char);
            Rf_error("the program is paniced\0".as_ptr() as *const core::ffi::c_char)
        }
    }
}
// May trigger double panic.
// #[cfg(feature="std")]
// fn panic(info: &core::panic::PanicInfo){
//     panic_handler(info);
// }
// #[cfg(feature="std")]
// #[no_mangle]
// pub extern fn init()->Owned {
//     std::panic::set_hook(Box::new(panic));
//     unsafe {Owned::new_bool(0)}
// }

#[repr(transparent)]
pub struct Owned(SEXP);
#[repr(transparent)]
pub struct Protected(Owned);
pub trait SExt where Self: Sized {
    fn as_sexp(&self) -> SEXP;
    /// Get the length of a vector.
    #[inline(always)]
    fn len(&self) -> usize {
        unsafe { Rf_xlength(self.as_sexp()) as usize }
    }
    /// Get the data pointer of a vector.
    /// although get a pointer should be safe, I marked it as an unsafe function
    /// since you must take care about what you want to do.
    #[inline(always)]
    unsafe fn data(self) -> *const c_void {
        unsafe { DATAPTR_RO(self.as_sexp()) }
    }
    /// Get the mutable data pointer of a vector.
    /// although get a pointer should be safe, I marked it as an unsafe function
    /// since you must take care about what you want to do.
    #[inline(always)]
    unsafe fn data_mut(self) -> *mut c_void {
        unsafe { DATAPTR(self.as_sexp()) }
    }
    /// check whether the data is a f64 vector
    #[inline(always)]
    fn is_real(self) -> bool {
        unsafe { Rf_isReal(self.as_sexp()) == 1 }
    }
    /// check whether the data is a i32 vector
    /// Since the current implementation only support i32, bool(stored as i32) and f64
    /// this function just check whether the vector is not real (thus is i32)
    #[inline(always)]
    unsafe fn unsafe_fast_compare_is_int(self) -> bool {
        unsafe { Rf_isReal(self.as_sexp()) != 1 }
    }
    /// check whether the data is a i32 vector
    /// the SEXP could be stored as integer if it is integer or logical.
    #[inline(always)]
    fn could_be_integer(self) -> bool {
        let robj = self.as_sexp();
        unsafe { Rf_isInteger(robj) == 1 || Rf_isLogical(robj) == 1 }
    }
    /// convert SEXP vector to read-only &[f64] slice.
    #[inline(always)]
    unsafe fn as_real_slice_unchecked(&self) -> &[f64] {
        let sexp: SEXP = self.as_sexp();
        unsafe { slice::from_raw_parts(sexp.data() as *const f64, sexp.len()) }
    }
    /// convert SEXP vector to read-only &[i32] slice.
    #[inline(always)]
    unsafe fn as_int_slice_unchecked(&self) -> &[i32] {
        let sexp: SEXP = self.as_sexp();
        unsafe { slice::from_raw_parts(sexp.data() as *const i32, sexp.len()) }
    }
}
pub trait MutableSEXP : SExt where Self: Sized {
    /// SAFETY: len should >0, ty should be `REALSXP`, `INTSXP` or `LGLSXP`
    unsafe fn new_type(len: int, ty:SEXPTYPE)->Self;
    /// SAFETY:
    /// FFI calls. length should >0 else I have no idea what will happen
    #[inline(always)]
    fn new_real(len: int) -> Self {
        unsafe { MutableSEXP::new_type(len, REALSXP) }
    }
    /// SAFETY:
    /// FFI calls. length should >0 else I have no idea what will happen
    #[inline(always)]
    fn new_int(len: int) -> Self {
        unsafe { MutableSEXP::new_type(len, INTSXP) }
    }
    /// SAFETY:
    /// FFI calls. length should >0 else I have no idea what will happen
    #[inline(always)]
    fn new_bool(len: int) -> Self {
        unsafe { MutableSEXP::new_type(len, LGLSXP) }
    }
    /// convert SEXP vector to read-only &mut [f64] slice.
    /// Do not use this trait in SEXP, since they might not owned by rust program
    /// and should be recognized as read-only.
    #[inline(always)]
    unsafe fn as_mut_real_slice_unchecked(&mut self) -> &mut [f64] {
        let sexp: SEXP = self.as_sexp();
        unsafe { slice::from_raw_parts_mut(sexp.data_mut() as *mut f64, sexp.len()) }
    }
    /// convert SEXP vector to read-only &mut [f64] slice.
    /// Do not use this trait in SEXP, since they might not owned by rust program
    /// and should be recognized as read-only.
    #[inline(always)]
    unsafe fn as_mut_int_slice_unchecked(&mut self) -> &mut [i32] {
        let sexp: SEXP = self.as_sexp();
        unsafe { slice::from_raw_parts_mut(sexp.data_mut() as *mut i32, sexp.len()) }
    }
}
impl SExt for SEXP {
    fn as_sexp(&self) -> SEXP {
        *self
    }
}
impl SExt for Owned {
    fn as_sexp(&self) -> SEXP {
        self.0
    }
}
impl SExt for Protected {
    fn as_sexp(&self) -> SEXP {
        self.0 .0
    }
}
impl MutableSEXP for Owned {
    #[inline(always)]
    unsafe fn new_type(len: int, ty:SEXPTYPE) -> Self {
        unsafe { Self(Rf_allocVector(ty, len)) }
    }
}
impl MutableSEXP for Protected {
    #[inline(always)]
    unsafe fn new_type(len: int, ty:SEXPTYPE) -> Self {
        unsafe { Self(Owned(Rf_protect(Rf_allocVector(ty, len)))) }
    }
}
impl From<Owned> for Protected {
    #[inline(always)]
    fn from(s: Owned) -> Self {
        // SAFETY: FFI calls.
        Self(unsafe { Owned(Rf_protect(s.0)) })
    }
}
impl From<Protected> for Owned {
    #[inline(always)]
    fn from(s: Protected) -> Self {
        unsafe {
            let s = core::mem::transmute::<Protected, Owned>(s);
            Rf_unprotect_ptr(s.0);
            s
        }
    }
}
impl From<Protected> for SEXP {
    #[inline(always)]
    fn from(s: Protected) -> Self {
        s.0 .0
    }
}
impl From<Owned> for SEXP {
    #[inline(always)]
    fn from(s: Owned) -> Self {
        s.0
    }
}
impl Drop for Protected {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { Rf_unprotect_ptr(self.0 .0) }
    }
}
pub mod prelude {
    pub use crate::{SExt, MutableSEXP};
    pub type SEXP=crate::SEXP;
    pub type Owned=crate::Owned;
    pub type Protected=crate::Protected;
}
