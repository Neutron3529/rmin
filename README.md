# rmin

## rmin - A minimal Rust lib for writting R extensions

This is a very early version, only support vector type, and thus its overhead is minimized.

Compare to the well-knowned `rextendr`, This crate although with some limitations but could provide a faster implementation, a smaller code size, and a faster compile time.

Since it is small enough, you could vendor this crate easily into your CRAN package easily.

## Status

The recent usable version is v0.3.1, DO NOT USE THE GIT VERSION

Please notice that, I am not familar with switching branches, the commit directly into the main branch is highly untrustable. Github is a repo only

## Upcoming breaking changes in v0.4.0:

[`character`] re-bind to [`Sexp<char>`], which has [`SEXPTYPE`] binds to [`STRSXP`](Rdef::STREXP)

The current [`character`] binding moves to [`char`], since R tells me the returned [`CHARSXP`] type has the name

adding an extra '%s\0' to printf and rf_errorcall, which prevent formating errors

## Features

Need at least one of these feature: `cfg-if`(for no_std environment) or `std`(for normal usage).

<details>
<summary>Details:</summary>

## `panic-info-message`

Enable rust feature `panic_info_message`, will bring Rust panic messages back to R, might be useful for debugging. Enabled by default.

## `std`

Most of the rust crates are rely on `std::*`, if you want to use other crate, you should enable this feature. It takes ~1s compile the whole crate without `lto`, but if you enable `lto` for a faster executing speed, it might takes ~5s to finish compiling it.

## `public-all`

The most evil and dangerous feature. Better not to enable it. Most of the useful functions have a marker feature named `public-by-default-even-public-all-is-not-set`, that feature is a marker feature, do nothing but only tells you what function you could obtain from [`prelude`] module.

## `min-import`

For [`prelude`] module. Since all the [`RType`](base::s::r_type::RType) aliases could be access from [`crate::prelude::R`], this feature disable import the aliases into [`prelude`] module.

## `cfg-if`

Enable by default since compile the exception handling functtion for `no_std` environment need `cfg-if`. If you are using `std` feature, this could be disabled.

## `public-by-default-even-public-all-is-not-set`

Dummy feature. Nothing happens if you disable it with `--no-default-feature`.

</details>

## Note

Please switch to [`prelude`] module page for a first glance, since I want to show all docs, most of the private things are documented with a `public-all` feature flag.
Please do not use them directly since most of them have a safe wrapper, and it is dangerous to use them directly.

## Usage

Version 0.1.0 provides a fastest (but ugly) way to achieve about 2x speedup on with functions. They are discarded in 0.2.* since they are really unsafe and may cause memory leak.

The currently 0.3.0 version is slightly different from 0.2.0, which rename `SEXP<T>` to [`Sexp<T>`], and (will) support things like [`Sexp`]<[`numeric_list`]>(R::numeric_list) or even an arbitrary list `Sexp<(T1,T2)>`.

Note: In the upcoming 0.4.0, all the decl_macro might be moved into a seperate crate which provide macros and proc_macros. This might only affect users with default no_std environment.

### 0.3.0, bring `#[no_std]` back!

In 0.3.0, feature `std` is optional again, which will give us a faster code generating speed.

#### Changes:

1. \[ x \] currently, new method and from (rust type) method goes to SExt, you could still write [`Owned<T>`]`::`[`new`](crate::prelude::Owned::new)`()`, but a [`Protected<T>`](crate::base::s::Protected) yields.
2. \[ x \] Add a [`catch_unwind`](crate::base::no_std::unwind::catch_unwind) for `no_std`.
3. \[ x \] Move `SEXP<T>` to [`Sexp<T>`] thus SEXP and Sexp could be occur in the same situation
4. \[ x \] Using macro 2.0 to hide most of the struct and method from user interface, but remains the doc for debug purpose.
5. \[   \] Adding support for lists (partially done.)

#### grammar
```rust
#![no_std]
use rmin::*;
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
fn main() {} // just makes compiler happy
```

The program above could be tested with test command
```bash
export LOAD="dyn.load('target/release/examples/libcompare_rmin.so');addnp=getNativeSymbolInfo('add_noprotect');addp=getNativeSymbolInfo('add_protect');panic=getNativeSymbolInfo('panic')" ; LC_ALL=C r -e "$LOAD;system.time(sapply(1:100000,function(x)tryCatch(.Call(wrap__panic),error=I)))" 2>/dev/null ; LC_ALL=C r -e "$LOAD;system.time(sapply(1:1000000,function(x).Call(addp,1.,2.)));system.time(sapply(1:1000000,function(x).Call(addp,1.,2.)))"
```
