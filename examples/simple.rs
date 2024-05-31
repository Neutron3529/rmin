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
