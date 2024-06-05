#[allow(non_snake_case)]
/// [Wrapper for R `SEXP`](Sexp)
#[path="base.s.rs"]
pub mod s;
use crate::prelude::*;
use core::ffi::c_char;
/// a trait for both std unwind safe and no_std unwind safe. It just impl UnwindSafe for all types in no_std mode, use it carefully!
#[cfg(feature = "std")]
pub trait UnwindSafe : std::panic::UnwindSafe {}
#[cfg(feature = "std")]
impl<T:std::panic::UnwindSafe> UnwindSafe for T {}

/// a trait for both std unwind safe and no_std unwind safe. It just impl UnwindSafe for all types in no_std mode, use it carefully!
#[cfg(not(feature = "std"))]
pub unsafe trait UnwindSafe {}
/// REALLY UNSAFE
#[cfg(not(feature = "std"))]
unsafe impl<T> UnwindSafe for T {}
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
// #[cfg(feature = "std")]
pub fn handle_panic<R, F: FnOnce() -> R + UnwindSafe>(f: F) -> R {
    let thing = match {
        #[cfg(feature = "std")] {
            std::panic::catch_unwind(f)
        }
        #[cfg(not(feature = "std"))] {
            unsafe {no_std::catch_unwind(f)}
        }
    } {
        Ok(ret) => return ret,
        Err(info) => {
            match info.downcast::<String>() {
                Ok(string)=>Owned::<character>::raw_from(format!("{:?}",string)),
                Err(info)=>Owned::<character>::raw_from(format!("payload type: {} (panic with no information)",core::any::type_name_of_val(&info)).as_str())
            }
        }
    };
    unsafe {thing.error()}
}
/// no_std aux functions, for internal procedure only.
#[cfg(not(feature = "std"))]
#[cfg_attr(doc, doc(cfg(not(feature = "std"))))]
pub mod no_std {
    use crate::{R::character, base::{macros::println, s::{Owned, SExt, r_type::lib_r::{R_chk_calloc, R_chk_free, R_chk_realloc}}}};
    use core::{intrinsics, any::Any, ffi::c_void, fmt::Write};
    extern crate alloc;
    use alloc::alloc::{GlobalAlloc, Layout};
    pub use alloc::{string::String, vec::Vec, boxed::Box};
    /// TODO: writting unwind code to enable unwind feature.
    #[lang = "eh_personality"]
    pub extern "C" fn rust_eh_personality() {}
    /// Global allocator using R's allocator.
    pub struct SimpleAllocator();
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
    use core::mem::{self, ManuallyDrop};
    #[allow(improper_ctypes_definitions)]
    extern "C" fn __rust_panic_cleanup(payload: *mut u8) -> *mut (dyn Any + Send + 'static) {
        unsafe {panic_unwind::__rust_panic_cleanup(payload)}
    }
    #[rustc_std_internal_symbol]
    extern "C" fn __rust_foreign_exception() -> ! {
        unsafe {crate::base::s::r_type::error("Rust cannot catch foreign exceptions\n\0".as_ptr())}
    }
    /// almost the same impl in std crate.
    pub unsafe fn catch_unwind<R, F: FnOnce() -> R>(f: F) -> Result<R, Box<dyn Any + Send>> {
        union Data<F, R> {
            f: ManuallyDrop<F>,
            r: ManuallyDrop<R>,
            p: ManuallyDrop<Box<dyn Any + Send>>,
        }

        // We do some sketchy operations with ownership here for the sake of
        // performance. We can only pass pointers down to `do_call` (can't pass
        // objects by value), so we do all the ownership tracking here manually
        // using a union.
        //
        // We go through a transition where:
        //
        // * First, we set the data field `f` to be the argumentless closure that we're going to call.
        // * When we make the function call, the `do_call` function below, we take
        //   ownership of the function pointer. At this point the `data` union is
        //   entirely uninitialized.
        // * If the closure successfully returns, we write the return value into the
        //   data's return slot (field `r`).
        // * If the closure panics (`do_catch` below), we write the panic payload into field `p`.
        // * Finally, when we come back out of the `try` intrinsic we're
        //   in one of two states:
        //
        //      1. The closure didn't panic, in which case the return value was
        //         filled in. We move it out of `data.r` and return it.
        //      2. The closure panicked, in which case the panic payload was
        //         filled in. We move it out of `data.p` and return it.
        //
        // Once we stack all that together we should have the "most efficient'
        // method of calling a catch panic whilst juggling ownership.
        let mut data = Data { f: ManuallyDrop::new(f) };

        let data_ptr = core::ptr::addr_of_mut!(data) as *mut u8;
        // SAFETY:
        //
        // Access to the union's fields: this is `std` and we know that the `r#try`
        // intrinsic fills in the `r` or `p` union field based on its return value.
        //
        // The call to `intrinsics::catch_unwind` is made safe by:
        // - `do_call`, the first argument, can be called with the initial `data_ptr`.
        // - `do_catch`, the second argument, can be called with the `data_ptr` as well.
        // See their safety preconditions for more information
        unsafe {
            return if intrinsics::catch_unwind(do_call::<F, R>, data_ptr, do_catch::<F, R>) == 0 {
                Ok(ManuallyDrop::into_inner(data.r))
            } else {
                Err(ManuallyDrop::into_inner(data.p))
            };
        }

        // We consider unwinding to be rare, so mark this function as cold. However,
        // do not mark it no-inline -- that decision is best to leave to the
        // optimizer (in most cases this function is not inlined even as a normal,
        // non-cold function, though, as of the writing of this comment).
        #[cold]
        unsafe fn cleanup(payload: *mut u8) -> Box<dyn Any + Send + 'static> {
            // SAFETY: The whole unsafe block hinges on a correct implementation of
            // the panic handler `__rust_panic_cleanup`. As such we can only
            // assume it returns the correct thing for `Box::from_raw` to work
            // without undefined behavior.
            let obj = unsafe { Box::from_raw(__rust_panic_cleanup(payload)) };
            // panic_count::decrease(); // there is no need to decrease the non-exist counter.
            obj
        }

        // SAFETY:
        // data must be non-NUL, correctly aligned, and a pointer to a `Data<F, R>`
        // Its must contains a valid `f` (type: F) value that can be use to fill
        // `data.r`.
        //
        // This function cannot be marked as `unsafe` because `intrinsics::catch_unwind`
        // expects normal function pointers.
        #[inline]
        fn do_call<F: FnOnce() -> R, R>(data: *mut u8) {
            // SAFETY: this is the responsibility of the caller, see above.
            unsafe {
                let data = data as *mut Data<F, R>;
                let data = &mut (*data);
                let f = ManuallyDrop::take(&mut data.f);
                data.r = ManuallyDrop::new(f());
            }
        }

        // We *do* want this part of the catch to be inlined: this allows the
        // compiler to properly track accesses to the Data union and optimize it
        // away most of the time.
        //
        // SAFETY:
        // data must be non-NUL, correctly aligned, and a pointer to a `Data<F, R>`
        // Since this uses `cleanup` it also hinges on a correct implementation of
        // `__rustc_panic_cleanup`.
        //
        // This function cannot be marked as `unsafe` because `intrinsics::catch_unwind`
        // expects normal function pointers.
        #[inline]
        #[rustc_nounwind] // `intrinsic::r#try` requires catch fn to be nounwind
        fn do_catch<F: FnOnce() -> R, R>(data: *mut u8, payload: *mut u8) {
            // SAFETY: this is the responsibility of the caller, see above.
            //
            // When `__rustc_panic_cleaner` is correctly implemented we can rely
            // on `obj` being the correct thing to pass to `data.p` (after wrapping
            // in `ManuallyDrop`).
            unsafe {
                let data = data as *mut Data<F, R>;
                let data = &mut (*data);
                let obj = cleanup(payload);
                data.p = ManuallyDrop::new(obj);
            }
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
            Owned::<character>::raw_from(message.as_str())
        };
        println!("Unhandled panic occurred, may cause memory leak, be caution!");
        unsafe {
            strsxp.error();
        }
    }
}
/// Basic println for no_std mode.
#[cfg(not(feature = "std"))]
#[cfg_attr(doc, doc(cfg(not(feature = "std"))))]
pub mod macros {
    /// Print things into R
    pub macro println ($($tt:tt)*) {{
        let mut x=$crate::String::new();
        core::fmt::write(&mut x, format_args!($($tt)*)).and_then(|_|::core::fmt::write(&mut x, format_args!("\n\0"))).expect("failed to write string");
        #[allow(unused_unsafe)]
        unsafe{ crate::base::s::r_type::print(x.as_ptr()) }
    }}

    /// Print things into R
    pub macro format ($($tt:tt)*) {{
        let mut x=$crate::String::new();
        core::fmt::write(&mut x, format_args!($($tt)*)).expect("failed to write string");
        x
    }}
}
