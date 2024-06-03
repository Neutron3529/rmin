use crate::{libR::{TYPEOF, Rf_protect, Rf_unprotect_ptr}, macros::impl_index};
pub use crate::RType::{RType, RTypeMut};
use core::{ops::{Index, IndexMut}, marker::PhantomData};
#[derive(Copy,Clone)]
/// ReadOnly SEXP, should not be changed.
///
/// This is the preferred way for accept ffi values
/// Since R marked the input values as `readonly`.
#[repr(transparent)]
pub struct SEXP<T:RType>{
    sexp:crate::libR::SEXP,
    _marker:[PhantomData<T>;0]
}
/// Owned [SEXP](crate::libR::SEXP), allocated by Rust code.
///
/// You could modify object as a slice object, and return it as a R vector.
/// Once it is created, you should take care about calling R function.
/// The ideal way is call R functions before it is allocated (e.g., convert all `SEXP<T>` into `&[T]`)
/// then allocate only 1 Owned object.
#[repr(transparent)]
pub struct Owned<T:RType>{
    sexp:crate::libR::SEXP,
    _marker:[PhantomData<T>;0]
}
impl<T:RType> Owned<T> {
    /// This function provides a way to protect any vector from R's GC
    /// You should take care about that, any allocation might recycle the previous allocated vector
    /// In the current fasion, we only alloc 1 return vector, thus protect is not necessary
    /// In case you want to communicate with R functions, you must convert all your [Owned](Owned<T>) values to
    /// the [Protected](Protected<T>) version.
    pub fn protect(self)->Protected<T> {
        self.into()
    }
}
/// Protected SEXP, should not be transferred across FFI boundary.
///
/// Since return a protected vector back to R will cause memory leak, this struct is marked as private
/// The interface of [`Proteted<T>`](Self) and [`Owned<T>`] is almost the same (expect [`Owned<T>`] have an extra [`.protect()`](Owned::protect) function call that protected the owned object to a [`Protected<T>`])
/// for FFI, you should only use
/// ```
/// extern "C-unwind" fn(SEXP<T1>, SEXP<T2>, ...)->Owned<TRet>
/// ```
/// and should never put [`Protected<T>`] in either side.
// pub mod protected {}
#[repr(transparent)]
pub struct Protected<T:RType>{
    sexp:crate::libR::SEXP,
    _marker:[PhantomData<T>;0]
}
/// SEXP extensions
///
/// Contains all the operation that a normal SEXP could execute.
pub trait SExt:Sized {
    /// SEXP Associated data, normally double->[f64], integer->[i32], logical->[u32], character->[u8],
    /// check [`RType`](trait.RType.html#foreign-impls) for the full supported list
    type Data:RType;
    /// get the inner SEXP.
    fn as_sexp(&self) -> crate::libR::SEXP;
    /// allocate a new SEXP object with length
    /// for `u8` (CHARcrate::libR::SEXP), new with length is not allowed
    /// thus CHARcrate::libR::SEXP accept a String object.
    fn new(len: <Self::Data as RType>::New ) -> Self where Self:Newable;
    /// check whether the data has the desired type
    #[inline(always)]
    fn is_correct_type(&self) -> bool {
        Self::Data::is_type(self.as_sexp())
    }
    /// get length of this SEXP.
    #[inline(always)]
    fn len(&self) -> usize {
        crate::RType::len(self.as_sexp()) as usize
    }
    /// got the read-only data of a SEXP object
    /// SAFETY: caller should ensure this SEXP has data with
    ///   1. SEXP.is_correct_type()
    ///   2. Self::Data::len(sexp)>0
    #[inline(always)]
    unsafe fn data_unchecked(&self) -> &[<Self::Data as RType>::Data] {
        // SAFETY:
        //       : with SEXP.is_correct_type(), data is valid for len * mem::size_of::<T>() many bytes.
        //       : with Self::Data::len(sexp)>0, data is non-null and aligned.
        //       : the Data is Rtype thus Data is Copy, no drop would be called.
        unsafe { core::slice::from_raw_parts(Self::Data::data(self.as_sexp()), self.len()) }
    }
    /// got the read-only data of a SEXP object
    #[inline(always)]
    fn data(&self) -> &[<Self::Data as RType>::Data] {
        if self.len()==0 {
            &[]
        } else {
            if self.is_correct_type() {
                // SAFETY : len>0 and with correct type.
                unsafe {self.data_unchecked()}
            } else {
                panic!(
                    "data has the type {}, but {}(={}) is required.",
                    // SAFETY: FFI calls.
                    unsafe{TYPEOF(self.as_sexp())},
                    core::any::type_name::<<Self::Data as RType>::Data>(),
                    <Self::Data as RType>::SEXPTYPE,
                )
            }
        }
    }
    /// got the read-only data of a SEXP object
    /// SAFETY: caller should ensure this SEXP has data with
    ///   1. SEXP.is_correct_type()
    ///   2. Self::Data::len(sexp)>0
    #[inline(always)]
    unsafe fn data_unchecked_mut(&self) -> &mut [<Self::Data as RType>::Data] where <Self as SExt>::Data: RTypeMut, Self:Mutable {
        // SAFETY:
        //       : with SEXP.is_correct_type(), data is valid for len * mem::size_of::<T>() many bytes.
        //       : with Self::Data::len(sexp)>0, data is non-null and aligned.
        //       : the Data is Rtype thus Data is Copy, no drop would be called.
        unsafe { core::slice::from_raw_parts_mut(Self::Data::data_mut(self.as_sexp()), self.len()) }
    }
    /// got the read-only data of a SEXP object
    #[inline(always)]
    fn data_mut(&mut self) -> &mut [<Self::Data as RType>::Data] where <Self as SExt>::Data: RTypeMut, Self:Mutable {
        if self.len()==0 {
            &mut []
        } else {
            if self.is_correct_type() {
                // SAFETY : len>0 and with correct type.
                unsafe {self.data_unchecked_mut()}
            } else {
                panic!(
                    "data has the type {}, but {}(={}) is required.",
                       // SAFETY: FFI calls.
                       unsafe{TYPEOF(self.as_sexp())},
                       core::any::type_name::<Self::Data>(),
                       <Self::Data as RType>::SEXPTYPE,
                )
            }
        }
    }
}
/// marked SEXP as newable
pub trait Newable {}
impl<T:RType> Newable for Owned<T>{}
/// A marker suggeest whether the SEXP is mutable
///
/// Could not obtained manually since it is behind a `macro` invocation
///
/// # Example
/// ```
/// use rmin::Mutable;
/// ```
pub trait Mutable {}
impl<T:RTypeMut> Mutable for Owned<T>{}
impl<T:RTypeMut> Mutable for Protected<T>{}
impl<T:RType> SExt for SEXP<T> {
    type Data=T;
    fn as_sexp(&self) -> crate::libR::SEXP {
        self.sexp
    }
    fn new(len:<Self::Data as RType>::New)->Self{
        Self{sexp:Self::Data::new(len),_marker:[]}
    }
}
impl<T:RType> SExt for Owned<T> {
    type Data=T;
    fn as_sexp(&self) -> crate::libR::SEXP {
        self.sexp
    }
    fn new(len:<Self::Data as RType>::New)->Self{
        Self{sexp:Self::Data::new(len),_marker:[]}
    }
}
impl<T:RType> SExt for Protected<T> {
    type Data=T;
    fn as_sexp(&self) -> crate::libR::SEXP {
        self.sexp
    }
    fn new(len:<Self::Data as RType>::New)->Self{
        // SAFETY: FFI calls.
        Self{sexp:unsafe { Rf_protect(Self::Data::new(len)) },_marker:[]}
    }
}
impl<T:RType> From<Owned<T>> for Protected<T> {
    #[inline(always)]
    fn from(s: Owned<T>) -> Self {
        // SAFETY: FFI calls.
        Self{sexp:unsafe { Rf_protect(s.sexp) },_marker:[]}
    }
}
impl<T:RType> From<Protected<T>> for Owned<T> {
    #[inline(always)]
    fn from(s: Protected<T>) -> Self {
        Self{sexp:s.sexp ,_marker:[]}
        // s is dropped and thus unprotected.
    }
}
impl<T:RType> From<Protected<T>> for SEXP<T> {
    #[inline(always)]
    fn from(s: Protected<T>) -> Self {
        Self{sexp:s.sexp,_marker:[]}
        // s is dropped and thus unprotected.
    }
}
impl<T:RType> From<Owned<T>> for SEXP<T> {
    #[inline(always)]
    fn from(s: Owned<T>) -> Self {
        Self{sexp:s.sexp,_marker:[]}
    }
}
impl<T:RType> Drop for Protected<T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { Rf_unprotect_ptr(self.sexp) }
    }
}
impl_index!{SExt=SExt RType=RType RTypeMut=RTypeMut Mutable=Mutable, SEXP Owned Protected}
