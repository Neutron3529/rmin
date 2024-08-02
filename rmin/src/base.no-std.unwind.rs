use core::{intrinsics, any::Any, mem::ManuallyDrop};
use super::Box;
use crate::println;

// extern crate unwind;
// use unwind as uw;
extern crate panic_unwind;
use panic_unwind::__rust_panic_cleanup;

extern crate rmin_eh;
// pub mod personality; // switched to rmin_eh
// #[lang ="eh_personality"]
// pub extern "C" fn eh_personality(){super::println!("eh_personality called");}
/// a function just make linker happy.
#[cfg_attr(not(test), no_mangle)]
pub extern "C" fn __rust_foreign_exception()->!{println!("__rust_foreign_exception called, abort.");core::intrinsics::abort()}
/// a function just make linker happy.
#[cfg_attr(not(test), no_mangle)]
pub extern "C" fn __rust_drop_panic()->!{println!("__rust_drop_panic called, abort.");core::intrinsics::abort()}
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

