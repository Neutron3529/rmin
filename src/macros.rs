#[doc(cfg(not(feature = "std")))]
/// Print things into R
pub macro println ($($tt:tt)*) {
    let mut x=String::new();
    ::core::fmt::write(&mut x, format_args!($($tt)*)).and_then(|_|::core::fmt::write(&mut x, format_args!("\n\0"))).expect("failed to write string");
    #[allow(unused_unsafe)]
    unsafe{ crate::libR::Rprintf(x.as_ptr() as *const ::core::ffi::c_char) }
}
