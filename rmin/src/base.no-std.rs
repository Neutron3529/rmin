use crate::{R::Rchar, base::s::{Sexp, SExt, r_type::lib_r::{R_chk_calloc, R_chk_free, R_chk_realloc}}};
use macros::*;
use core::{ffi::c_void, any::Any, panic::PanicPayload};
#[cfg(not(test))]
use core::fmt::Write; // for panic panic_handler

extern crate alloc;
use alloc::alloc::{GlobalAlloc, Layout};
pub use alloc::{string::{String, ToString}, vec::Vec, boxed::Box};

/// load unwind related functions, should not use in production.
#[path = "base.no-std.unwind.rs"]
pub mod unwind;

/// handle_panic for no_std
pub fn handle_panic<R, F: FnOnce() -> R>(f: F) -> R {
    let thing : Sexp<_> = match unsafe {unwind::catch_unwind(f)} {
        Ok(ret) => return ret,
        Err(info) => // match {info.downcast::<String>()}
        {
            match info.downcast::<Sexp<Rchar>>(){
                Ok(charexp)=>*charexp,
                Err(info)=>Sexp::raw_from(format!("{:?}", info)).into()
            }
            // Ok(string)=>Owned::<Rchar>::raw_from(format!("{:?}",string)),
            // Err(info)=>Owned::<Rchar>::raw_from(format!("payload type: {} (panic with no information)",core::any::type_name_of_val(&info)).as_str())
        }
    };
    unsafe {thing.error()}
}



/// Basic println for no_std mode.
pub mod macros {
    /// Print things into R
    pub macro println ($($tt:tt)*) {{
        let mut x=$crate::String::new();
        core::fmt::write(&mut x, format_args!($($tt)*)).and_then(|_|::core::fmt::write(&mut x, format_args!("\n\0"))).expect("failed to write string");
        #[allow(unused_unsafe)]
        unsafe{ $crate::base::s::r_type::print(x.as_ptr()) }
    }}

    /// Print things into R
    pub macro format ($($tt:tt)*) {{
        let mut x=$crate::String::new();
        core::fmt::write(&mut x, format_args!($($tt)*)).expect("failed to write string");
        x
    }}
}


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

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    // print errors.
    // adding extra scope to ensure all the dynamic allocated resources are dropped.
    let charsxp : Sexp<_> = {
        const REASON:&str="Fatal: cannot write to string during panic handling.";
        let mut message = String::new();
        if let Some(s) = info.payload().downcast_ref::<&str>() {
            write!(&mut message, "panic occurred: {s:?}").expect(REASON)
        } else {
            write!(&mut message, "panic occurred:").expect(REASON)
        }
        #[cfg(feature="panic-info-message")]
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
                "\n  at {}, line {}, column {}.",
                i.file(),
                    i.line(),
                    i.column(),
            ).expect(REASON)
        }
        // println!("finalized {message} to ");
        Sexp::raw_from(message.as_str()).into()
    };

    let reason = unsafe {panic_unwind::__rust_start_panic(&mut RewrapBox(Box::new(charsxp)))};
    println!("Unhandled panic occurred, may cause memory leak, be caution!\nNote: This panic omits some deconstructor and may cause memory leak, please restart R as soon as possible to avoid further issues.\n  panic_unwind::reason = {}",reason);
    unsafe {
        charsxp.error();
    }
}
/// std struct for unwinding.
pub struct RewrapBox(Box<dyn Any + Send>);
unsafe impl PanicPayload for RewrapBox {
    fn take_box(&mut self) -> *mut (dyn Any + Send) {
        Box::into_raw(core::mem::replace(&mut self.0, Box::new(())))
    }

    fn get(&mut self) -> &(dyn Any + Send) {
        &*self.0
    }
}
