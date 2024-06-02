use rmin::prelude::*;
/// Return a+b to R.
#[no_mangle]
pub extern "C-unwind" fn add_protect(a:SEXP<f64>,b:SEXP<f64>) -> Owned<f64> {
    handle_panic(||{
        let mut c=Owned::new(1).protect();
        c[0]=a[0]+b[0];
        c.into()
    })
}
#[no_mangle]
pub extern "C-unwind" fn add_noprotect(a:SEXP<f64>,b:SEXP<f64>) -> Owned<f64> {
    handle_panic(||{
        let mut c=Owned::new(1);
        c[0]=a[0]+b[0];
        c
    })
}

/// raise panic.
#[no_mangle]
pub extern "C-unwind" fn panic() -> Owned<f64> {
    handle_panic(||{
        panic!("error occurs")
    })
}
/* R test code
 * export LOAD="dyn.load('target/release/examples/libcompare_rmin.so');addnp=getNativeSymbolInfo('add_noprotect');addp=getNativeSymbolInfo('add_protect');panic=getNativeSymbolInfo('panic')" ; LC_ALL=C r -e "$LOAD;system.time(sapply(1:100000,function(x)tryCatch(.Call(wrap__panic),error=I)))" 2>/dev/null ; LC_ALL=C r -e "$LOAD;system.time(sapply(1:1000000,function(x).Call(addnp,1.,2.)));system.time(sapply(1:1000000,function(x).Call(addnp,1.,2.)))"
 */
