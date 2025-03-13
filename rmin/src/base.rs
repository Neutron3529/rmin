#[allow(non_snake_case)]
/// [Wrapper for R `SEXP`](crate::Sexp)
#[path="base.s.rs"]
pub mod s;

#[cfg_attr(doc, doc(inline, cfg(all())))]
#[cfg(not(have_std))]
pub use no_std::handle_panic;


#[cfg(have_std)]
use crate::{Sexp, SExt, R::Rchar};
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
#[cfg(have_std)]
pub fn handle_panic<R, F: FnOnce() -> R + std::panic::UnwindSafe>(f: F) -> R {
    let thing:Sexp<Rchar> = match std::panic::catch_unwind(f) {
        Ok(ret) => return ret,
        Err(info) => match info.downcast::<String>() {
            Ok(string)=>Sexp::raw_from(format!("{:?}",string)),
            Err(info)=>Sexp::raw_from(format!("payload type: {} (panic with no information)",core::any::type_name_of_val(&info)).as_str())
        }.into()
    };
    unsafe {thing.error()}
}

/// no_std aux functions, for internal procedure only.
#[cfg(not(have_std))]
#[cfg_attr(doc, doc(cfg(not(have_std))))]
#[path = "base.no-std.rs"]
pub mod no_std;



/// Basic println for both std and no_std mode.
/// this println is more preferred than the default one, since it could output capturable outputs in windows
pub mod macros {
    #[cfg(not(have_std))]
    pub(crate) use crate::String;
    #[cfg(not(have_std))]
    macro fmt ($fn:tt, $($tt:tt)*) {{
        let mut x=String::new();
        core::fmt::write(&mut x, $fn!($($tt)*)).expect("failed to write string");
        x
    }}
    #[cfg(not(have_std))]
    /// make format string
    pub macro format ($($tt:tt)*) {{
        fmt!(format_args, $($tt)*)
    }}
    #[cfg(not(have_std))]
    /// make format string ended with "\n"
    pub macro format_nl ($($tt:tt)*) {{
        fmt!(format_args_nl, $($tt)*)
    }}
    #[allow(unused_macros)]
    macro rprint($arg:literal, $fn:ident, $($tt:tt)*) {{
        let mut x=String::new();
        core::fmt::write(&mut x,format_args!($arg,format_args!($($tt)*))).expect("failed to write string");
        #[allow(unused_unsafe)]
        unsafe{ $crate::base::s::r_type::$fn(x.as_ptr()) }
    }}
    /// Print things into R
    pub macro println ($($tt:tt)*) {
        rprint!("{}\n\0", print, $($tt)*)
    }
    /// Print things into R, without "\n"
    pub macro print ($($tt:tt)*) {
        rprint!("{}\0", print, $($tt)*)
    }
    /// Print errors into R
    pub macro eprintln ($($tt:tt)*) {
        rprint!("{}\n\0", eprint, $($tt)*)
    }
    /// Print errors into R, without "\n"
    pub macro eprint ($($tt:tt)*) {
        rprint!("{}\0", eprint, $($tt)*)
    }
}
