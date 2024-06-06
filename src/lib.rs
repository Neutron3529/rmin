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
//! The currently 0.3.0 version is slightly different from 0.2.0, which rename `SEXP<T>` to [`Sexp<T>`], and (will) support things like [`Sexp`]<[`numeric_list`]>(R::numeric_list) or even an arbitrary list `Sexp<(T1,T2)>`.
//! Since renaming breaks the current symver, I'll delay the publishing of 0.3.0
//!
//! I cannot ensure whether the api will change again in the future, but the api seems to be stable.
//!
//! ## 0.3.0, bring `#[no_std]` back!
//!
//! In 0.3.0, feature `std` is optional again, which will give us a faster code generating speed.
//!
//! ### Changes:
//!
//! 1. \[ x \] currently, new method and from (rust type) method goes to SExt, you could still write [`Owned<T>`]`::`[`new`](crate::prelude::Owned::new)`()`, but a [`Protected<T>`](crate::base::s::Protected) yields.
//! 2. \[ x \] Add a [`catch_unwind`](crate::base::no_std::catch_unwind) for `no_std`.
//! 3. \[ x \] Move `SEXP<T>` to [`Sexp<T>`] thus SEXP and Sexp could be occur in the same situation
//! 4. \[ x \] Using macro 2.0 to hide most of the struct and method from user interface, but remains the doc for debug purpose.
//! 5. \[   \] Adding support for lists (partially done.)
//!
//! ### grammar
//! ```no_run
//! #![no_std]
//! use rmin::*;
//! /// Return a+b to R.
//! #[no_mangle]
//! pub extern "C" fn add_protect(a:Sexp<f64>,b:Sexp<f64>) -> Owned<f64> {
//!     handle_panic(||{
//!         let mut c=Owned::new(1);
//!         c[0]=a[0]+b[0];
//!         c.into()
//!     })
//! }
//! #[no_mangle]
//! pub extern "C" fn add_noprotect(a:Sexp<f64>,b:Sexp<f64>) -> Owned<f64> {
//!     handle_panic(||{
//!         let mut c=Owned::new(1);
//!         c[0]=a[0]+b[0];
//!         c.into()
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
//! fn main() {} // just makes compiler happy
//! ```
//!
//! The program above could be tested with test command
//! ```bash
//! export LOAD="dyn.load('target/release/examples/libcompare_rmin.so');addnp=getNativeSymbolInfo('add_noprotect');addp=getNativeSymbolInfo('add_protect');panic=getNativeSymbolInfo('panic')" ; LC_ALL=C r -e "$LOAD;system.time(sapply(1:100000,function(x)tryCatch(.Call(wrap__panic),error=I)))" 2>/dev/null ; LC_ALL=C r -e "$LOAD;system.time(sapply(1:1000000,function(x).Call(addp,1.,2.)));system.time(sapply(1:1000000,function(x).Call(addp,1.,2.)))"
//! ```
#![cfg_attr(
    not(feature = "std"),
    feature(needs_panic_runtime, rustc_attrs, core_intrinsics, panic_unwind, std_internals, strict_provenance, exposed_provenance),
    no_std
)]
#![feature(rustdoc_missing_doc_code_examples, decl_macro)]
#![cfg_attr(doc, feature(doc_cfg))]
#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    // rustdoc::missing_doc_code_examples
)]
#![allow(rustdoc::private_intra_doc_links)] // Protected<T>::sexp is a private field.
#![allow(unused_imports)]
#![allow(internal_features)]
#![feature(
    lang_items,
    panic_info_message,
    c_unwind,
    associated_type_defaults,
    impl_trait_in_assoc_type
)]
#[cfg(not(feature = "std"))]
extern crate panic_unwind;
macro pm {
    ()=>{},
    ($(#[$meta:meta])* $pub:ident $mod:tt $item:tt ; $($tt:tt)*) => {
        $(#[$meta])*
        #[cfg_attr(doc, doc(cfg(feature="public-all")))]
        #[cfg(any(doc, feature="public-all"))]
        $pub $mod $item;
        $(#[$meta])*
        #[cfg(not(any(doc, feature="public-all")))]
        $mod $item;
        pm!{$($tt)*}
    },
    ($(#[$meta:meta])* $pub:ident $mod:tt $item:tt {$($blk:tt)*} $($tt:tt)*) => {
        $(#[$meta])*
        #[cfg_attr(doc, doc(cfg(feature="public-all")))]
        #[cfg(any(doc, feature="public-all"))]
        $pub $mod $item {$($blk)*}
        $(#[$meta])*
        #[cfg(not(any(doc, feature="public-all")))]
        $mod $item {$($blk)*}
        pm!{$($tt)*}
    }
}
pm! {
    /// macros for `no_std` mode.
    pub mod macros {
        /// eval things conditionally, used in macros.
        pub macro cond_eval {
            (($cond:tt) $($tt:tt)*)=>{$($tt)*},
            (() $($tt:tt)*)=>{}
        }
    }
    /// Basic module for panic handling.
    ///
    /// Note: this module is invisible unless you enable the `public-all` feature gate.
    ///
    /// # Example
    /// ```compile_fail
    /// use base;
    /// ```
    pub mod base;
}

/// Prelude, the only thing you could (and should) use
///
/// Originally, to prevent the misuse of things like putting [`Protected<T>`](crate::base::s::Protected) into a FFI interface, most of the structs are private and hided. And the crate is designed to function just with the visible prelude module.
///
/// Currently, a feature gate `public-all` is added, and thus users could see all of the docs, which helps debugging with this crate.
/// But, since the `public-all` feature gate is added to the base module, all the type are marked as **Available on crate feature `public-all` only**
///
/// To really checks what you could use directly, all the doc that prelude imports are inlined, and the prelude crate are marked as **Available on crate feature `always-avaliable-with-prelude-accesses` only.**
///
/// Here, `always-avaliable-with-prelude-accesses` is a dummy feature, it is enabled by default and does nothing (thus disable it makes nothing happens). The usage of that feature is that, when you see that feature,
/// you could use it without enable `public-all`.
///
/// Sometimes, you may obtain *private* things such as [`Protected<T>`](crate::base::s::Protected) (from a [`SExt`]`::<_>::`[`new`](SExt::new)`(_)`), that's OK, since the R ffi does not accept a [`Protected<T>`](crate::base::s::Protected),
/// and you cannot visit [`Protected<T>`](crate::base::s::Protected) directly without `public-all`, you could only keep that type until rust drop it automatically.
///
/// As for normal usage, it make no different to write `use rmin::*` or `use rmin::prelude::*`.
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

#[cfg_attr(
    doc,
    doc(cfg(all(
        feature = "public-by-default-even-public-all-is-not-set",
        feature = "public-all"
    )))
)]
pub mod prelude {
    #[doc(inline)]
    pub use crate::base::{
        handle_panic,
        s::{Owned, SExt, Sexp},
    };

    #[doc(inline)]
    pub use crate::base::s::r_type::alias as R;

    #[doc(inline)]
    pub use crate::base::s::r_type::define as Rdef;

    #[doc(inline)]
    #[cfg(not(feature = "min-import"))]
    #[cfg_attr(doc, doc(cfg(not(any(doc, feature = "min-import")))))]
    pub use crate::base::s::r_type::alias::*;

    #[doc(inline)]
    #[cfg(not(feature = "std"))]
    #[cfg_attr(doc, doc(cfg(not(any(doc, feature = "std")))))]
    pub use crate::base::no_std::{String, ToString, Vec, Box, macros::{format, println}};
}
pub use prelude::*;
