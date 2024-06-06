#![cfg_attr(not(feature = "std"), no_std)]
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
pub extern "C" fn protect_and_unprotect(a: Owned<i32>, b: Owned<i32>) -> Owned<()> {
    handle_panic(|| {
        let (len, loops) = (a[0], b[0]);
        let mut pv: Vec<_> = (0..len).map(|_| Owned::<f64>::new(1)).collect(); // Protect<T>
        let mut vec: Vec<Owned<f64>>;
        for _ in 0..loops {
            vec = pv.into_iter().map(|x| x.into()).collect();
            pv = vec.into_iter().map(|x| x.into()).collect()
        }
        Owned::raw(0)
    })
}

/// raise panic.
#[no_mangle]
pub extern "C" fn panic() -> Owned<f64> {
    handle_panic(|| {
        #[derive(Debug)]
        struct Guard();
        impl Drop for Guard {
            fn drop(&mut self){
                println!("Guard dropped.")
            }
        }
        let g=Guard();
        panic!("error occurs {g:?}")
    })
}

/// check whether memory leak happens.
#[no_mangle]
pub extern "C" fn panic_with_8MB_resource_allocated(var: Sexp<i32>) -> Owned<i32> {
    handle_panic(|| {
        let mut val = var[0];
        let mut a = Vec::with_capacity(1048576);
        a.iter_mut().for_each(|x: &mut i32| {
            *x += val;
            val += 1 + *x
        });
        if val == 0 {
            core::hint::black_box(&mut a);
            panic!("let me try")
        }
        core::hint::black_box(&mut a);
        Owned::raw(1)
    })
}

/* R test code
export LOAD="dyn.load('target/release/examples/libcompare_rmin.so');addnp=getNativeSymbolInfo('add_noprotect');addp=getNativeSymbolInfo('add_protect');panic=getNativeSymbolInfo('panic');pup=getNativeSymbolInfo('protect_and_unprotect');leak=getNativeSymbolInfo('panic_with_8MB_resource_allocated')" ; LC_ALL=C r -e "$LOAD;system.time(sapply(1:100000,function(x)tryCatch(.Call(wrap__panic),error=I)))" 2>/dev/null ; LC_ALL=C r -e "$LOAD;srystem.time(sapply(1:1000000,function(x).Call(addp,1.,2.)));system.time(sapply(1:1000000,function(x).Call(addp,1.,2.)))" ; LC_ALL=C r -e "$LOAD;system.time(.Call(pup,1000L,1000L))" ; LC_ALL=C r -e "$LOAD;system.time(.Call(leak,0L))"
 */
