#![cfg_attr(feature = "no_std", no_std)]
use rmin::prelude::*;
#[no_mangle]
pub extern "C" fn add_noprotect(a: OptionSexp<f64>, b: OptionSexp<f64>) -> Owned<f64> {
    handle_panic(|| {
        unsafe {
            if a.missing() || b.missing() {
                panic!(
                    "Parameter missing detected, a:{} b:{}",
                    a.missing(),
                    b.missing()
                )
            }
        }
        let mut c = Owned::raw(1);
        // note: dealing with unregistered function with missing could be really unsafe!
        c[0] = unsafe {a.into_option().unwrap()[0] + b.into_option().unwrap()[0]};
        c
    })
}

#[no_mangle]
/// lets suppose a and b are not missing.
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

/// lets suppose a.missing could detect the missingness of a.
/// Actually there might be no such guarateens.
/// raise panic.
#[no_mangle]
pub extern "C" fn panic(a: OptionSexp<i32>) -> Owned<f64> {
    handle_panic(|| unsafe {
        #[derive(Debug)]
        struct Guard();
        impl Drop for Guard {
            fn drop(&mut self) {
                println!("%Guard dropped%.")
            }
        }
        let g = Guard();
        if a.missing() {
            panic!("%error occurs%d.")
        }
        println!("{g:?} is still alive, %a.missing is {}%, ", a.missingness());
        Sexp::raw(0)
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

/// The following code (including this line) shows a way to call macros directly
#[export] // rmin::macros::export (or rmin::export with feature rmin-macros enabled)
/// the location of #[export] is not so important.
/// Return a+b to R.
/// R will check whether a or b is missing, but R do not guarateen the type is f64, nor guarateen the length >0.
fn add(a: Sexp<f64>, b: Sexp<f64>) -> Owned<f64> {
    Owned::raw_from(&[a[0] + b[0]])
}
// Here, the done macro accept a single token as the name of dll R entry
// this could be omit if package name in Cargo.toml is valid (i.e., does not contain symbols like `-`)
// here, since the name is rmin_examples, the extra `rmin_examples` cannot be omitted.
// it is worth mention that, it is `librmin_examples.so` rather than `rmin_examples.so` is generated.
// the entry defined here is for `dyn.load('librmin_examples.so')` in R.
done!(rmin_examples);
