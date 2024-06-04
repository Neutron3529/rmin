use rmin::prelude::*;
/// Return a+b to R.
#[no_mangle]
pub extern "C" fn add_protect(a: Sexp<f64>, b: Sexp<f64>) -> Owned<f64> {
    handle_panic(|| {
        let mut c = Owned::raw(1);
        c[0] = a[0] + b[0];
        c
    })
}
#[no_mangle]
pub extern "C" fn add_noprotect(a: Sexp<f64>, b: Sexp<f64>) -> Owned<f64> {
    handle_panic(|| {
        let mut c = Owned::raw(1);
        c[0] = a[0] + b[0];
        c
    })
}


#[no_mangle]
pub extern "C" fn protect_and_unprotect(a:Owned<i32>, b:Owned<i32>) -> Owned<()> {
    handle_panic(|| {
        let (len,loops)=(a[0],b[0]);
        let mut pv:Vec<_>=(0..len).map(|_|Owned::<f64>::new(1)).collect(); // Protect<T>
        let mut vec:Vec<Owned<f64>>;
        for _ in 0..loops {
            vec=pv.into_iter().map(|x|x.into()).collect();
            pv=vec.into_iter().map(|x|x.into()).collect()
        }
        Owned::raw(0)
    })
}

/// raise panic.
#[no_mangle]
pub extern "C-unwind" fn panic() -> Owned<f64> {
    handle_panic(|| panic!("error occurs"))
}
/* R test code
export LOAD="dyn.load('target/release/examples/libcompare_rmin.so');addnp=getNativeSymbolInfo('add_noprotect');addp=getNativeSymbolInfo('add_protect');panic=getNativeSymbolInfo('panic');pup=getNativeSymbolInfo('protect_and_unprotect')" ; LC_ALL=C r -e "$LOAD;system.time(sapply(1:100000,function(x)tryCatch(.Call(wrap__panic),error=I)))" 2>/dev/null ; LC_ALL=C r -e "$LOAD;system.time(sapply(1:1000000,function(x).Call(addp,1.,2.)));system.time(sapply(1:1000000,function(x).Call(addp,1.,2.)))" ; LC_ALL=C r -e "$LOAD;system.time(.Call(pup,1000L,1000L))"
 */
