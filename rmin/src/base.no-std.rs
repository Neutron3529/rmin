use crate::{R::Rchar, base::s::{Sexp, SExt, r_type::lib_r::{calloc, free, realloc, malloc}}};
use crate::{format, println};
use core::{any::Any, ffi::c_void, fmt::{Debug, Display}, panic::PanicPayload};
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

/// Global allocator using R's allocator.
pub struct SimpleAllocator();
#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator();
unsafe impl Sync for SimpleAllocator {}
unsafe impl GlobalAlloc for SimpleAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            malloc(layout.size()) as *mut u8
            // R_chk_calloc(layout.size().div_euclid(layout.align()), layout.align()) as *mut u8
        }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe {
            calloc(layout.size().div_euclid(layout.align()), layout.align()) as *mut u8
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe { free(ptr as *mut c_void) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { realloc(ptr as *mut c_void, new_size) as *mut u8 }
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
        #[cfg(feature="panic-info-message")]
        write!(&mut message,"Panic occured: {}", info.message()).expect(REASON);
        #[cfg(not(feature="panic-info-message"))]
        write!(&mut message,"Panic occured: (message ignored)").expect(REASON);
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
#[derive(Debug)]
pub struct RewrapBox(Box<dyn Any + Send>);
impl Display for RewrapBox {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}
unsafe impl PanicPayload for RewrapBox {
    fn take_box(&mut self) -> *mut (dyn Any + Send) {
        Box::into_raw(core::mem::replace(&mut self.0, Box::new(())))
    }

    fn get(&mut self) -> &(dyn Any + Send) {
        &*self.0
    }
}
