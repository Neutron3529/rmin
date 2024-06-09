#![no_std]

use rmin::*;
// #[no_mangle]
// pub extern "C" fn test()->Owned<character> {
//     Owned::raw_from_str("im .1u test")
// }

/// fine
///
/// a simple function tell I'm fine2
#[export]
pub fn fine(a: Sexp<i32>)->Owned<character> {
    Owned::raw_from_str(format!("I'm fine{}",a[0]))
}

/// fine2
///
/// a simple function tell I'm fine2
#[export]
pub fn fine2()->Owned<character> {
    Owned::raw_from_str("I'm fine2")
}

/// add
///
/// add a and b then return the result.
#[export]
pub fn add(a: Sexp<f64>, b: Sexp<f64>)->Owned<f64> {
    Owned::raw_from(&[a[0]+b[0]])
}


/// add4
///
/// add 4 numbers together
#[export]
pub fn add4(a: Sexp<f64>, b: Sexp<f64>, c: Sexp<f64>, d:Sexp<f64>)->Owned<f64> {
    Owned::raw_from(&[a[0]+b[0]+c[0]+d[0]])
}

/// another_sum
///
/// a function that compare the sum speed between Rust and R. R should be faster.
#[export]
pub fn another_sum(a: Sexp<f64>)->Owned<f64> {
    Owned::raw_from(&[a.data().into_iter().copied().sum::<f64>()])
}

// const R_CALL_METHOD:&[reg::R_CallMethodDef]=&[
//     reg::R_CallMethodDef {name:c"._1u".as_ptr(), fun:test as *const _, numArgs:0},
//     reg::R_CallMethodDef {name:core::ptr::null(), fun:core::ptr::null(), numArgs:0}
// ];
// #[no_mangle]
// extern fn R_init_librext(info:*mut reg::DllInfo){
//     println!("called librext.");
//     unsafe {
//         reg::R_registerRoutines(
//             info,
//             core::ptr::null(),
//             R_CALL_METHOD.as_ptr(),
//             core::ptr::null(),
//             core::ptr::null()
//         );
//     }
// }
done!(rext);
