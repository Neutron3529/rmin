#![no_std]
use rmin::*;
#[no_mangle]
extern "C" fn test()->*mut core::ffi::c_void{
    unsafe {SExt::as_sexp(&Sexp::<character>::raw_from([<Owned<Rchar> as Into<Sexp<Rchar>>>::into(Owned::raw_from("im fine."))])) as *mut core::ffi::c_void}
}

const R_CALL_METHOD:&[reg::R_CallMethodDef]=&[
    reg::R_CallMethodDef {name:"test".as_ptr() as *const _, fun:test as *const _, numArgs:0},
    reg::R_CallMethodDef {name:core::ptr::null(), fun:core::ptr::null(), numArgs:0}
];
#[no_mangle]
extern fn R_init_rext(info:*mut reg::DllInfo){
    println!("called init.");
    unsafe {
        let res=reg::R_registerRoutines(
            info,
            core::ptr::null(),
            R_CALL_METHOD.as_ptr(),
            core::ptr::null(),
            core::ptr::null()
        );
        println!("registered with result {res}");
    }
}
