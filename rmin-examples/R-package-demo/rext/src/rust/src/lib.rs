#![no_std]

use rmin::*;
// #[no_mangle]
// pub extern "C" fn test()->Owned<character> {
//     Owned::raw_from_str("im .1u test")
// }

/**fine

@description
a simple function tell I'm fine

@details
blah blah

@param id integer.
*/
#[export]
pub fn fine(id: Sexp<i32>)->Owned<character> {
    Owned::raw_from_str(format!("I'm fine{}",id[0]))
}

/// fine2
///
/// @description
/// a simple function tell I'm fine2
///
#[export]
pub fn fine2()->Owned<character> {
    Owned::raw_from_str("I'm fine2")
}

struct S4;
impl R5 for S4 {}
impl Drop for S4 {
    fn drop(&mut self){
        println!("S4 dropped")
    }
}

#[export]
pub fn s4()->Owned<externalptr>{
    S4::new()
}

/**add

@description
add a and b then return the result.

@details
blah blah

@param a,b double.*/
#[export]
pub fn add(a: Sexp<f64>, b: Sexp<f64>)->Owned<f64> {
    Owned::raw_from(&[a[0]+b[0]])
}

/**add

@description
add 4 numbers together.

@details
blah blah

@param a,b,c,d double.*/
#[export]
pub fn add4(a: Sexp<f64>, b: Sexp<f64>, c: Sexp<f64>, d:Sexp<f64>)->Owned<f64> {
    Owned::raw_from(&[a[0]+b[0]+c[0]+d[0]])
}

/// another_sum
///
/// a function that compare the sum speed between Rust and R. R should be faster.
/**another_sum

@description
a function that compare the sum speed between Rust and R. R should be faster..

@details
blah blah

@param a double vector.*/
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
