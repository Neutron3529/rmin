macro_rules! syntax_group {
    ($($tt:tt)*) => { $($tt)* };
}
#[cfg(not(feature="std"))]
syntax_group!{
    #[macro_export]
    /// Print things into R
    macro_rules! println {
        ($($tt:tt)*) => {
            let mut x=String::new();
            ::core::fmt::write(&mut x, format_args!($($tt)*)).and_then(|_|::core::fmt::write(&mut x, format_args!("\n\0"))).expect("failed to write string");
            #[allow(unused_unsafe)]
            unsafe{ crate::libR::Rprintf(x.as_ptr() as *const ::core::ffi::c_char) }
        }
    }
}
macro_rules! impl_index {
    ($($tt:tt)*) => {
        $(
            impl<T: RType> core::ops::Index<usize> for $tt<T> where $tt<T>:SExt  {
                type Output = <<$tt<T> as SExt>::Data as RType>::Data;
                fn index(&self, index: usize) -> &<<$tt<T> as SExt>::Data as RType>::Data {
                    self.data().index(index)
                }
            }
            impl<T: RTypeMut> core::ops::IndexMut<usize> for $tt<T> where $tt<T>:Mutable, <$tt<T> as SExt>::Data:RTypeMut {
                fn index_mut(&mut self, index: usize) -> &mut <<$tt<T> as SExt>::Data as RType>::Data {
                    self.data_mut().index_mut(index)
                }
            }
        )*
    }
}