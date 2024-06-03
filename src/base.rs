use crate::{libR::Rf_error, Owned, SExt, SEXP};
use core::ffi::c_char;
/// The most common function that may use close to the FFI boundary.
/// This function could catch all possible panic and convert them into normal R error message.
/// # Usage:
/// ```no_run
/// use rmin::*;
/// handle_panic(||{
///     let a:Vec<_>=(0..16).collect();
///     panic!("handle_panic will drop `a` even a panic is triggered.")
/// });
/// ```
#[cfg(feature = "std")]
pub fn handle_panic<R, F: FnOnce() -> R + std::panic::UnwindSafe>(f: F) -> R {
    let thing = match std::panic::catch_unwind(f) {
        Ok(ret) => return ret,
        Err(info) => {
            match info.downcast::<String>() {
                Ok(string)=>Owned::<crate::RType::character>::raw_from(format!("{:?}",string)),
                Err(info)=>Owned::<crate::RType::character>::raw_from(format!("payload type: {} (panic with no information)",core::any::type_name_of_val(&info)).as_str())
            }
        }
    };
    unsafe {Rf_error(thing.data().as_ptr() as *const c_char)}
}
#[cfg(not(feature = "std"))]
syntax_group! {
    extern crate alloc;
    use alloc::alloc::{GlobalAlloc, Layout};
    pub use alloc::{string::String, vec::Vec};
    use libR::{R_chk_calloc, R_chk_free, R_chk_realloc};
    /// TODO: writting unwind code to enable unwind feature.
    #[lang = "eh_personality"]
    pub extern "C" fn rust_eh_personality() {}

    struct SimpleAllocator();
    #[global_allocator]
    static ALLOCATOR: SimpleAllocator = SimpleAllocator();
    unsafe impl Sync for SimpleAllocator {}
    unsafe impl GlobalAlloc for SimpleAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            unsafe {
                R_chk_calloc(layout.size().div_euclid(layout.align()), layout.align()) as *mut u8
            }
        }
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            unsafe {
                R_chk_calloc(layout.size().div_euclid(layout.align()), layout.align()) as *mut u8
            }
        }
        unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
            unsafe { R_chk_free(ptr as *mut c_void) }
        }
        unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
            unsafe { R_chk_realloc(ptr as *mut c_void, new_size) as *mut u8 }
        }
    }
    #[panic_handler]
    fn panic_handler(info: &core::panic::PanicInfo) -> ! {
        // print errors.
        // adding extra scope to ensure all the dynamic allocated resources are dropped.
        let strsxp={
            const REASON:&str="Fatal: cannot write to string during panic handling.";
            let mut message = String::new();
            if let Some(s) = info.payload().downcast_ref::<&str>() {
                write!(&mut message, "panic occurred: {s:?}").expect(REASON)
            } else {
                write!(&mut message, "panic occurred:").expect(REASON)
            }
            if let Some(i) = info.message() {
                if let Some(s) = info.payload().downcast_ref::<&str>() {
                    write!(&mut message," {i} ({s})").expect(REASON)
                } else {
                    write!(&mut message," {i}").expect(REASON)
                }
            }
            if let Some(i) = info.location() {
                write!(
                    &mut message,
                    "\n  at {}, line {}, column {}.\nNote: This panic omits some deconstructor and may cause memory leak, please restart R as soon as possible to avoid further issues.",
                    i.file(),
                       i.line(),
                       i.column(),
                ).expect(REASON)
            }
            // println!("finalized {message} to ");
            RStr::from_str(message.as_str())
        };
        // println!("strsxp with len {} and type {}",strsxp.0.len(),unsafe{TYPEOF(strsxp.0)});
        unsafe {
            Rf_error(strsxp.0.data() as *const c_char)
        }
    }
}
