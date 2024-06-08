#![no_std]

use rmin::*;
// #[no_mangle]
// pub extern "C" fn test()->Owned<character> {
//     Owned::raw_from_str("im .1u test")
// }

#[export]
pub fn fine2()->Owned<character> {
    Owned::raw_from_str("im fine2")
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
done!(librext);
