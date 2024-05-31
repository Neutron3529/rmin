# rmin - A minimal Rust lib for writting R extensions

This is a very early version, only support vector type, and thus its overhead is minimized.

Compare to the well-knowned `rextendr`, This crate although with some limitations but could provide a faster implementation, a smaller code size, and a faster compile time ( could generate a release build in 0.45s ).

Since it is small, you could vendor this crate easily into your CRAN package.

# Usage

Version 0.1.0 provides a fastest (but ugly) way to achieve about 2x speedup on with functions. If you really care about speed, you should only using the ugly gammar.

Further grammar might be added at some cost (mostly cost my spare time.. but some will also cost compile time or R execution time (for example, wrapping function directly. calling wrapped `test(a,b)` is slower than `.Call(wrap__test,a,b)` directly.))

## 0.1.0, the beginning

The grammar provided by 0.1.0 is assumed to be the base and might not change unless it do harm to the further development.

In later version, I may adding proc-macro support thus we could write better program.

Currently, we have to write program by hand.

### grammar

```rust
#![no_std] // this is required. At least it is required in v0.1.0
use rmin::prelude::*;

/// without `#[no_mangle]` and `extern`, you might not seen this function in R.
/// the signature should be a lot of SEXP to a Owned SEXP.
/// please do not consider send a `Protected` back to R, since it would cause memory leak.
/// Sometimes you may send SEXP directly, that's OK.
#[no_mangle]
pub extern fn test(a:SEXP, b:SEXP) -> Owned {
    let (a_data,b_data,c_data);
    let mut c=Owned::new_real(1);
    unsafe {
        a_data=a.as_real_slice_unchecked();
        b_data=b.as_real_slice_unchecked();
        c_data=c.as_mut_real_slice_unchecked();
    }
    c_data[0]=a_data[0]+b_data[0];
    c
}
/// No need to register the function again (as what we should do in rextendr)
/// instead, we should register them with R code, at least now it should.
```
The program above with test command
```bash
LC_ALL=C R -q -e "dyn.load('target/release/examples/libsimple.so');system.time(sapply(1:100000,function(x).Call('test',1.,2.)));system.time(sapply(1:100000,function(x).Call('test',1.,2.)))"
```
# benchmark
0.1.0 yields
```R
> dyn.load('target/release/examples/libsimple.so');system.time(sapply(1:100000,function(x).Call('test',1.,2.)));system.time(sapply(1:100000,function(x).Call('test',1.,2.)))
   user  system elapsed
  0.161   0.007   0.168
   user  system elapsed
  0.148   0.000   0.149
>
>
```
To further speedup calculation, we could use
```bash
LC_ALL=C R -q -e "dyn.load('target/release/examples/libsimple.so')"\
  -e "system.time({wrap__test=getNativeSymbolInfo('test')"\
  -e "test=function(a,b).Call(wrap__test,as.double(a),as.double(b))"\
  -e "sapply(1:100000,function(x)test(1,2))});"\
  -e "system.time(sapply(1:100000,function(x)test(1,2)))"\
  -e "system.time(sapply(1:100000,function(x).Call(wrap__test,as.double(1),as.double(2))))"\
  -e "system.time(sapply(1:100000,function(x).Call(wrap__test,as.double(1),as.double(2))))"
```
which yields
```R
> dyn.load('target/release/examples/libsimple.so')
> system.time({wrap__test=getNativeSymbolInfo('test')
  test=function(a,b).Call(wrap__test,as.double(a),as.double(b))
  sapply(1:100000,function(x)test(1,2))});
   user  system elapsed
  0.178   0.000   0.178
> system.time(sapply(1:100000,function(x)test(1,2)))
   user  system elapsed
  0.149   0.003   0.152
> system.time(sapply(1:100000,function(x).Call(wrap__test,as.double(1),as.double(2))))
   user  system elapsed
  0.109   0.000   0.108
> system.time(sapply(1:100000,function(x).Call(wrap__test,as.double(1),as.double(2))))
   user  system elapsed
   0.11    0.00    0.11
>
>
```
Compare with `rextendr`:

```bash
LC_ALL=C R -q -e "path='/me/fine'"\
  -e "setwd(path)"\
  -e "usethis::create_package('.')"\
  -e "rextendr::use_extendr()"\
  -e "cat('use extendr_api::prelude::*;\n#[extendr]\nfn test(a:i32,b:i32)->i32 { a + b }\nextendr_module! {\n    mod fine;\n    fn test;\n}\n', file=paste(path,'src/rust/src/lib.rs',sep='/'))"\
  -e "rextendr::document()"\
  -e "system.time(sapply(1:100000,function(x)test(1,2)))"\
  -e "system.time(sapply(1:100000,function(x)test(1,2)))"\
  -e "system.time(sapply(1:100000,function(x).Call(wrap__test,1,2)))"\
  -e "system.time(sapply(1:100000,function(x).Call(wrap__test,1,2)))"
```
Ignored some unhappy error (I have no idea why `path='/me/fine'` compiles, `path=/me/notfine` yield an error about `Failed to generate wrapper functions`.), we could got:
```
> path='/me/fine'
> setwd(path)
> usethis::create_package('.')
v Setting active project to '/me/fine'
v Creating 'R/'
v Writing 'DESCRIPTION'
Package: fine
Title: What the Package Does (One Line, Title Case)
Version: 0.0.0.9000
Authors@R (parsed):
    * First Last <first.last@example.com> [aut, cre] (YOUR-ORCID-ID)
Description: What the package does (one paragraph).
License: `use_mit_license()`, `use_gpl3_license()` or friends to
    pick a license
Encoding: UTF-8
Roxygen: list(markdown = TRUE)
RoxygenNote: 7.3.1
v Writing 'NAMESPACE'
v Setting active project to '<no active project>'
> rextendr::use_extendr()
i First time using rextendr. Upgrading automatically...
i Setting `Config/rextendr/version` to "0.3.1"
v Creating src/rust/src.
v Setting active project to '/me/fine'
v Writing 'src/entrypoint.c'
v Writing 'src/Makevars'
v Writing 'src/Makevars.win'
v Writing 'src/Makevars.ucrt'
v Writing 'src/.gitignore'
v Writing src/rust/Cargo.toml
v Writing 'src/rust/src/lib.rs'
v Writing 'src/fine-win.def'
v Writing 'R/extendr-wrappers.R'
v Finished configuring extendr for package fine.
* Please update the system requirement in DESCRIPTION file.
* Please run `rextendr::document()` for changes to take effect.
> cat('use extendr_api::prelude::*;\n#[extendr]\nfn test(a:i32,b:i32)->i32 { a + b }\nextendr_module! {\n    mod fine;\n    fn test;\n}\n', file=paste(path,'src/rust/src/lib.rs',sep='/'))
> rextendr::document()
i Generating extendr wrapper functions for package: fine.
i Re-compiling fine (debug build)
-- R CMD INSTALL -------------------------------------------------------------------------------------------------------------------------------------------------------------------
-  installing *source* package 'fine' ...
   ** using staged installation
   ** libs
   using C compiler: 'gcc (GCC) 14.1.1 20240522'
   rm -Rf fine.so ./rust/target/release/libfine.a entrypoint.o
   gcc -I"/usr/include/R/" -DNDEBUG   -I/usr/local/include    -fpic  -O2 -march=native -pipe -pipe -fno-plt -fexceptions         -Wp,-D_FORTIFY_SOURCE=2 -Wformat -Werror=format-security         -fstack-clash-protection -fcf-protection -g -ffile-prefix-map=/build/r/src=/usr/src/debug/r -flto=auto -ffat-lto-objects  -UNDEBUG -Wall -pedantic -g -O0 -fdiagnostics-color=always -c entrypoint.c -o entrypoint.o
   # In some environments, ~/.cargo/bin might not be included in PATH, so we need
   # to set it here to ensure cargo can be invoked. It is appended to PATH and
   # therefore is only used if cargo is absent from the user's PATH.
   if [ "true" != "true" ]; then \
        export CARGO_HOME=/me/fine/src/.cargo; \
   fi && \
        export PATH="/usr/local/sbin:/usr/local/bin:/usr/bin:/opt/cuda/bin:/opt/cuda/nsight_compute:/opt/cuda/nsight_systems/bin:/usr/bin/site_perl:/usr/bin/vendor_perl:/usr/bin/core_perl:/usr/lib/rustup/bin:/home/neutron/.cargo/bin" && \
        cargo build --lib --release --manifest-path=./rust/Cargo.toml --target-dir ./rust/target
       Updating `ustc` index
        Locking 10 packages to latest compatible versions
      Compiling proc-macro2 v1.0.84
      Compiling unicode-ident v1.0.12
      Compiling libR-sys v0.6.0
      Compiling paste v1.0.15
      Compiling extendr-api v0.6.0
      Compiling once_cell v1.19.0
      Compiling quote v1.0.36
      Compiling syn v2.0.66
      Compiling extendr-macros v0.6.0
      Compiling fine v0.1.0 (/me/fine/src/rust)
       Finished `release` profile [optimized] target(s) in 34.90s
   if [ "true" != "true" ]; then \
        rm -Rf /me/fine/src/.cargo && \
        rm -Rf ./rust/target/release/build; \
   fi
   gcc -shared -L/usr/lib64/R/lib -Wl,-O1,--sort-common,--as-needed,-z,relro,-z,now -flto=auto -o fine.so entrypoint.o -L./rust/target/release -lfine -L/usr/lib64/R/lib -lR
   installing to /tmp/RtmpMMeEfN/devtools_install_c08929da6615/00LOCK-fine/00new/fine/libs
   ** checking absolute paths in shared objects and dynamic libraries
-  DONE (fine)
v Writing 'R/extendr-wrappers.R'
i Updating fine documentation
Writing NAMESPACE
i Loading fine
x extendr-wrappers.R:12: `@docType "package"` is deprecated.
i Please document "_PACKAGE" instead.
Writing fine-package.Rd
> system.time(sapply(1:100000,function(x)test(1,2)))
   user  system elapsed
  0.188   0.017   0.205
> system.time(sapply(1:100000,function(x)test(1,2)))
   user  system elapsed
  0.174   0.003   0.178
> system.time(sapply(1:100000,function(x).Call(wrap__test,1,2)))
   user  system elapsed
  0.147   0.007   0.155
> system.time(sapply(1:100000,function(x).Call(wrap__test,1,2)))
   user  system elapsed
  0.142   0.000   0.142
>
>
```
besides a longer compiling time, the final program is slower than this crate. This is the main reason why I wrote this crate.

# misc

Feel free to file an issue in case you meet some problem :)

I'm a Linux user, but I'll try my best to solve some windows only problem.
