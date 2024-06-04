//! # rmin - A minimal Rust lib for writting R extensions
//!
//! This is a very early version, only support vector type, and thus its overhead is minimized.
//!
//! Compare to the well-knowned `rextendr`, This crate although with some limitations but could provide a faster implementation, a smaller code size, and a faster compile time
//! ( could generate a release build in 0.45s with `#![no_std], but may cause memory leak).
//!
//! Since it is small enough, you could vendor this crate easily into your CRAN package.
//!
//! # Note
//!
//! Please switch to [`prelude`] module page for a first glance, here contains all of the doc, which will only be enabled with feature=
//!
//! # Usage
//!
//! Version 0.1.0 provides a fastest (but ugly) way to achieve about 2x speedup on with functions. They are discarded in 0.2.* since they are really unsafe and may cause memory leak.
//!
//! The currently 0.3.0 version is slightly different from 0.2.0, which rename `SEXP<T>` to [`Sexp<T>`], and (will) support things like `Sexp<[f64]>` or even an arbitrary list `Sexp<(T1,T2)>`.
//! Since renaming breaks the current symver, I'll delay the publishing of 0.3.0
//!
//! I cannot ensure whether the api will change again in the future, but the api seems to be stable.
//!
//! ## 0.3.0, (plans:) bring `#[no_std]` back!
//!
//! In 0.3.0, feature `std` is optional again, which will give us a faster code generating speed.
//!
//! ### Changes:
//!
//! 1. \[ x \] currently, new method and from (rust type) method goes to SExt, you could still write [`Owned<T>`]`::`[`new`](crate::prelude::Owned::new)`()`, but a [`Protected<T>`](crate::base::s::Protected) yields.
//! 2. \[   \] Add a [`panic_handler`] for `no_std`.
//! 3. \[   \] Move `SEXP<T>` to [`Sexp<T>`] thus SEXP and Sexp could be occur in the same situation
//! 4. \[ x \] Using macro 2.0 to hide most of the struct and method from user interface, but remains the doc for debug purpose.
//! 5. \[   \] Adding support for lists
//!
//! ### grammar
//! ```rust
//! #![no_std]
//! use rmin::*;
//! /// Return a+b to R.
//! #[no_mangle]
//! pub extern "C" fn add_protect(a:Sexp<f64>,b:Sexp<f64>) -> Owned<f64> {
//!     handle_panic(||{
//!         let mut c=Owned::new(1).protect();
//!         c[0]=a[0]+b[0];
//!         c.into()
//!     })
//! }
//! #[no_mangle]
//! pub extern "C" fn add_noprotect(a:Sexp<f64>,b:Sexp<f64>) -> Owned<f64> {
//!     handle_panic(||{
//!         let mut c=Owned::new(1);
//!         c[0]=a[0]+b[0];
//!         c
//!     })
//! }
//!
//! /// raise panic.
//! #[no_mangle]
//! pub extern "C" fn panic() -> Owned<f64> {
//!     handle_panic(||{
//!         panic!("error occurs")
//!     })
//! }
//! ```
//!
//! The program above could be tested with test command
//! ```bash
//! export LOAD="dyn.load('target/release/examples/libcompare_rmin.so');addnp=getNativeSymbolInfo('add_noprotect');addp=getNativeSymbolInfo('add_protect');panic=getNativeSymbolInfo('panic')" ; LC_ALL=C r -e "$LOAD;system.time(sapply(1:100000,function(x)tryCatch(.Call(wrap__panic),error=I)))" 2>/dev/null ; LC_ALL=C r -e "$LOAD;system.time(sapply(1:1000000,function(x).Call(addp,1.,2.)));system.time(sapply(1:1000000,function(x).Call(addp,1.,2.)))"
//! ```
#![feature(rustdoc_missing_doc_code_examples, decl_macro, doc_cfg)]
#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    // rustdoc::missing_doc_code_examples
)]
#![allow(rustdoc::private_intra_doc_links)] // Protected<T>::sexp is a private field.
#![allow(unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(internal_features)]
#![feature(
    lang_items,
    panic_info_message,
    c_unwind,
    associated_type_defaults,
    impl_trait_in_assoc_type
)]
/// private macro, adding feature attributes to items gracefully
macro syntax_group($($tt:tt)*) { $($tt)* }

#[doc(cfg(feature="public-all"))]
/// macros for `no_std` mode.
pub mod macros;
#[doc(cfg(feature="public-all"))]
/// Basic module for panic handling.
pub mod base;
#[cfg(not(any(doc, feature="public-all")))]
mod macros;
#[cfg(not(any(doc, feature="public-all")))]
mod base;
/// Prelude, the only thing you could (and should) use
///
/// Currently, `rmin` generate a full doc for everything.
///
/// Choose your favorite one:)
///
/// # Example
/// ## 1. use prelude
/// ```
/// use rmin::prelude::*;
/// ```
/// ## 2. use a shorter version
/// ```
/// use rmin::*;
/// ```
pub mod prelude {
    pub use crate::base::{s::{Sexp, Owned, SExt, r_type::{NULL, logical, integer, character, numeric, lib_r::SEXP}}, handle_panic};
    #[cfg(not(feature = "std"))]
    pub use crate::base::{String,Vec};
}
pub use prelude::*;
