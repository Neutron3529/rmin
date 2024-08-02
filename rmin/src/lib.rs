/*!
# rmin - A minimal Rust lib for writting R extensions

This is a very early version, only support vector type, and thus its overhead is minimized.

Compare to the well-knowned `rextendr`, This crate although with some limitations but could provide a faster implementation, a smaller code size, and a faster compile time.

Since it is small enough, you could vendor this crate easily into your CRAN package easily.

# Status

The recent usable version is v0.4.3, DO NOT USE THE GIT VERSION

Please notice that, I am not familar with switching branches, the commit directly into the main branch is highly untrustable. Github is a repo only

# Breaking changes in v0.4.3:

- [x] Create an (unsafe) type [`OptionSexp`], since there is no guarateen makes `MISSING(a missing value)` in R returns non-zero (nor even returns, since the pointer may be invalid). This is not a choice, since with macro support, [`Sexp`] cannot missing,
- [x] unify the two entry with lib*.so and *.so in the macro: create both.
- [x] You should import with `use rmin::{*, println};` in std mode, otherwise an warning is generated. Since `std::println!` cannot output to Rgui.exe, override std::println with an explicit import is needed.

# Upcoming breaking changes in v0.4.0:

[`character`] re-bind to [`Sexp<char>`], which has [`SEXPTYPE`] binds to [`STRSXP`](Rdef::STRSXP)

The v0.3.0 [`character`] binding moves to [`Rchar`], since R tells me the returned [`CHARSXP`](Rdef::CHARSXP) type has the name

adding an extra '%s\0' to printf and rf_errorcall, which prevent formating errors

# Features

Need at least one of these feature: `cfg-if`(for no_std environment) or `std`(for normal usage).

<details>
<summary>Details:</summary>

# `panic-info-message`

Enable rust feature `panic_info_message`, will bring Rust panic messages back to R, might be useful for debugging. Enabled by default.

# `std`

Most of the rust crates are rely on `std::*`, if you want to use other crate, you should enable this feature. It takes ~1s compile the whole crate without `lto`, but if you enable `lto` for a faster executing speed, it might takes ~5s to finish compiling it.

# `core`

A counter part for `std`, currently `std` is an indicator that just yields a warning while not correctly being specific correctly. This feature controls the linking of exception handling language items, and thus cannot be ignored when enable it.

# `rmin-macros`

Import proc-macros `#[export] fn func_name(...)...{...}` and `done!(crate_name)` into [`crate::prelude`] and thus avaliable in [`crate`]::* root directly.

Notice that, macros require `rmin::reg` path to work (it is enabled automatically when choosing macros in `rmin` crate, if you enable rmin-macros as an independent dependency, you should enable `rmin::reg` manually.)

## `rmin-macros-camel-ass-wrapper`

Internal use only, define the internal name with camel-ass naming method (aka iOS naming method) to avoid name collision.

## `rmin-macros-warning`

Raise a warning with function with error (or empty) signature, for example. `fn()->Owned<character>` will yield a `warning, [[]] is omitted` since the signature is empty.

`fn(a:Sexp<f64>,)->Owned<f64>` also yields a same warning (due to the last comma)

They might harm the macro, thus raise an warning (although the 2 examples above are harmless, writting things like `(a:Sexp<f64>,,b:Sexp<f64>)` will interrupt the compile procedure.)

## `rmin-macros-verbose`

Disable by default, contains some simple information such as the exported function name, and what the finalizer generates.

# `public-all`

The most evil and dangerous feature. Better not to enable it. Most of the useful functions have a marker feature named `public-by-default-even-public-all-is-not-set`, that feature is a marker feature, do nothing but only tells you what function you could obtain from [`prelude`] module.

# `min-import`

For [`prelude`] module. Since all the [`RType`](base::s::r_type::RType) aliases could be access from [`crate::prelude::R`], this feature disable import the aliases into [`prelude`] module.

# `register-routines`

Register R routines, mainly for macros since hand writting such thing is painful.

# `cfg-if`

Enable by default since compile the exception handling functtion for `no_std` environment need `cfg-if`. If you are using `std` feature, this could be disabled.

# `public-by-default-even-public-all-is-not-set`

Dummy feature. Nothing happens if you disable it with `--no-default-feature`.

</details>

# Note

Please switch to [`prelude`] module page for a first glance, since I want to show all docs, most of the private things are documented with a `public-all` feature flag.
Please do not use them directly since most of them have a safe wrapper, and it is dangerous to use them directly.

# Usage

Version 0.1.0 provides a fastest (but ugly) way to achieve about 2x speedup on with functions. They are discarded in 0.2.* since they are really unsafe and may cause memory leak.

The currently 0.3.0 version is slightly different from 0.2.0, which rename `SEXP<T>` to [`Sexp<T>`], and (will) support things like [`Sexp`]<[`numeric_list`]>(R::numeric_list) or even an arbitrary list `Sexp<(T1,T2)>`.

Note: In the upcoming 0.4.0, all the decl_macro might be moved into a seperate crate which provide macros and proc_macros. This might only affect users with default no_std environment.

## 0.3.0, bring `#[no_std]` back!

In 0.3.0, feature `std` is optional again, which will give us a faster code generating speed.

### Changes:

1. - [x] currently, new method and from (rust type) method goes to SExt, you could still write [`Owned<T>`]`::`[`new`](crate::prelude::Owned::new)`()`, but a [`Protected<T>`](crate::base::s::Protected) yields.
2. - [x] Add a [`catch_unwind`](crate::base::no_std::unwind::catch_unwind) for `no_std`.
3. - [x] Move `SEXP<T>` to [`Sexp<T>`] thus SEXP and Sexp could be occur in the same situation
4. - [x] Using macro 2.0 to hide most of the struct and method from user interface, but remains the doc for debug purpose.
5. - [ ] Adding support for lists (partially done.)

### grammar
```no_run
#![no_std]
use rmin::{*, println};
/// Return a+b to R.
#[no_mangle]
pub extern "C" fn add_protect(a:Sexp<f64>,b:Sexp<f64>) -> Owned<f64> {
    handle_panic(||{
        let mut c=Owned::new(1);
        c[0]=a[0]+b[0];
        c.into()
    })
}
#[no_mangle]
pub extern "C" fn add_noprotect(a:Sexp<f64>,b:Sexp<f64>) -> Owned<f64> {
    handle_panic(||{
        let mut c=Owned::new(1);
        c[0]=a[0]+b[0];
        c.into()
    })
}

/// raise panic.
#[no_mangle]
pub extern "C" fn panic() -> Owned<f64> {
    handle_panic(||{
        panic!("error occurs")
    })
}

/// with macro
/// macro will register this function, thus R will check whether all parameters are missing
#[export]
fn macro_will_expand_and_register_it(a:Sexp<f64>)->Owned<f64>{
    let mut b=Owned::new(1);
    b[0]=a.data().sum();
}
done!();// in case you're using macros, adding a done! is necessary, this done call generate the
        // register routine, which will ensure the expanded code is checked.
fn main() {} // just makes compiler happy
```

The program above could be tested with test command
```bash
export LOAD="dyn.load('target/release/examples/libcompare_rmin.so');addnp=getNativeSymbolInfo('add_noprotect');addp=getNativeSymbolInfo('add_protect');panic=getNativeSymbolInfo('panic')" ; LC_ALL=C r -e "$LOAD;system.time(sapply(1:100000,function(x)tryCatch(.Call(wrap__panic),error=I)))" 2>/dev/null ; LC_ALL=C r -e "$LOAD;system.time(sapply(1:1000000,function(x).Call(addp,1.,2.)));system.time(sapply(1:1000000,function(x).Call(addp,1.,2.)))"
```
*/
#![cfg_attr(not(have_std),
    feature(lang_items, rustc_attrs, core_intrinsics, panic_unwind, std_internals, strict_provenance, exposed_provenance),
    // feature(c_unwind,impl_trait_in_assoc_type),
    no_std
)]
#![cfg_attr(feature = "const_type_name", feature(const_type_name))]
#![feature(decl_macro)]
#![cfg_attr(any(doc, test), feature(doc_cfg, rustdoc_missing_doc_code_examples))]
#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    // rustdoc::missing_doc_code_examples
)]
#![allow(internal_features)]
#![feature(associated_type_defaults)]

#[cfg(not(have_std))]
extern crate panic_unwind;

#[cfg(doc)]
use base::s::r_type::lib_r::SEXPTYPE;

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
/**
Prelude, the only thing you could (and should) use.

Originally, to prevent the misuse of things like putting [`Protected<T>`](crate::base::s::Protected) into a FFI interface, most of the structs are private and hided. And the crate is designed to function just with the visible prelude module.

Currently, a feature gate `public-all` is added, and thus users could see all of the docs, which helps debugging with this crate.
But, since the `public-all` feature gate is added to the base module, all the type are marked as **Available on crate feature `public-all` only**

For those who have `public-all` feature, to really checks what you could use directly, all the doc that prelude imports are inlined, and the prelude crate are marked as **Available on crate feature `always-avaliable-with-prelude-accesses` only.**

Here, `always-avaliable-with-prelude-accesses` is a dummy feature, it is enabled by default and does nothing (thus disable it makes nothing happens). The usage of that feature is that, when you see that feature,
you could use it without enable `public-all`. **(But other feature are still affected (such as `have_std` and `rmin-macros` and `register-routines`))**

Sometimes, you may obtain *private* things such as [`Protected<T>`](crate::base::s::Protected) (from a [`SExt`]`::<_>::`[`new`](SExt::new)`(_)`), that's OK, since the R ffi does not accept a [`Protected<T>`](crate::base::s::Protected),
and you cannot visit [`Protected<T>`](crate::base::s::Protected) directly without `public-all`, you could only keep that type until rust drop it automatically.

As for normal usage, it make no different to write `use rmin::*` or `use rmin::prelude::*`.

Choose your favorite one:)

# Example
## 1. use prelude
```
use rmin::prelude::*;
```
## 2. use a shorter version
```
use rmin::*;
```
*/
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
        s::{Owned, Sexp, OptionSexp, SExt, R5},
    };

    #[doc(inline)]
    pub use crate::base::s::r_type::alias as R;

    #[doc(inline)]
    pub use crate::base::s::r_type::define as Rdef;

    #[doc(inline)]
    #[cfg(not(feature = "min-import"))]
    #[cfg_attr(doc, doc(cfg(not(feature = "min-import"))))]
    pub use crate::base::s::r_type::alias::*;

    #[doc(inline)]
    pub use crate::base::macros::*;

    #[doc(inline)]
    #[cfg(not(have_std))]
    #[cfg_attr(doc, doc(cfg(not(have_std))))]
    pub use crate::base::no_std::{
        Box, String, ToString, Vec,
    };

    // #[cfg(have_std)]
    // pub(crate) use std::{
    //     boxed::Box,
    //     string::{String, ToString},
    //     vec::Vec
    // };

    /// a simple (re-exported) module for R routines registration.
    #[cfg_attr(doc, doc(cfg(feature = "register-routines")))]
    #[cfg(any(doc, feature = "register-routines"))]
    pub mod reg {
        pub use crate::base::s::r_type::lib_r::{
            DllInfo, R_CallMethodDef, R_forceSymbols, R_registerRoutines, R_useDynamicSymbols,
        };
    }
    /// re-exported macros: done
    #[doc(inline)]
    #[cfg_attr(doc, doc(cfg(feature = "rmin-macros")))]
    #[cfg(any(doc, feature = "rmin-macros"))]
    pub use rmin_macros::done;
    /// re-exported macros: export
    #[doc(inline)]
    #[cfg_attr(doc, doc(cfg(feature = "rmin-macros")))]
    #[cfg(any(doc, feature = "rmin-macros"))]
    pub use rmin_macros::export;
}
pub use prelude::*;
