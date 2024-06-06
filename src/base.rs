#[allow(non_snake_case)]
/// [Wrapper for R `SEXP`](crate::Sexp)
#[path="base.s.rs"]
pub mod s;

#[cfg_attr(doc, doc(inline, cfg(all())))]
#[cfg(not(feature = "std"))]
pub use no_std::handle_panic;


#[cfg(feature = "std")]
use crate::{Sexp, SExt, R::character};

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
#[cfg_attr(doc, doc(cfg(all())))]
#[cfg(feature = "std")]
pub fn handle_panic<R, F: FnOnce() -> R + std::panic::UnwindSafe>(f: F) -> R {
    let thing:Sexp<character> = match std::panic::catch_unwind(f) {
        Ok(ret) => return ret,
        Err(info) => match info.downcast::<String>() {
            Ok(string)=>Sexp::raw_from(format!("{:?}",string)),
            Err(info)=>Sexp::raw_from(format!("payload type: {} (panic with no information)",core::any::type_name_of_val(&info)).as_str())
        }.into()
    };
    unsafe {thing.error()}
}
/// no_std aux functions, for internal procedure only.
#[cfg(not(feature = "std"))]
#[cfg_attr(doc, doc(cfg(not(feature = "std"))))]
#[path = "base.no-std.rs"]
pub mod no_std;
