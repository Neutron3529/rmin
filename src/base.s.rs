/// Basic Data Type for R
///
/// Should be invisible for users
#[path = "base.s.r-type.rs"]
pub mod r_type;
use r_type::{RType, RTypeMut, RDefault, RTypeFrom, error, alias::*, SEXPext, SEXP, SEXPTYPE};
#[cfg(not(feature="std"))]
use r_type::print;
#[cfg(doc)]
use r_type::define::*; // for doc;
/// impl Index and IndexMut for a type.
pub mod macros {
    /// internal impl.
    pub macro impl_sext_index (SExt=$SExt:tt SEXP=$SEXP:tt RType=$RType:tt RTypeMut=$RTypeMut:tt Mutable=$Mutable:tt,$($tt:tt)*) {
        use core::ops::{Index,IndexMut};
        $(
            impl<T: $RType> $SExt for $tt<T> {
                type Data = T;
                unsafe fn as_sexp(&self) -> $SEXP {
                    self.sexp
                }
            }
            impl<T: $RType> Index<usize> for $tt<T> where $tt<T>:$SExt  {
                type Output = <<$tt<T> as $SExt>::Data as $RType>::Data;
                fn index(&self, index: usize) -> &<<$tt<T> as $SExt>::Data as $RType>::Data {
                    self.data().index(index)
                }
            }
            impl<T: $RTypeMut> IndexMut<usize> for $tt<T> where $tt<T>:$Mutable, <$tt<T> as $SExt>::Data:$RTypeMut {
                fn index_mut(&mut self, index: usize) -> &mut <<$tt<T> as $SExt>::Data as $RType>::Data {
                    self.data_mut().index_mut(index)
                }
            }
        )*
    }
}
use macros::impl_sext_index;
use core::marker::PhantomData;
#[derive(Copy, Clone)]
/// ReadOnly [`SEXP`], should not be changed.
///
/// This is the preferred way for accept ffi values
/// Since R marked the input values as `readonly`.
#[repr(transparent)]
pub struct Sexp<T: RType> {
    sexp: SEXP,
    _marker: [PhantomData<T>; 0],
}
/// Owned [`SEXP`], allocated by Rust code.
///
/// You could modify object as a slice object, and return it as a R vector.
/// Once it is created, you should take care about calling R function.
/// The ideal way is call R functions before it is allocated (e.g., convert all `Sexp<T>` into `&[T]`)
/// then allocate only 1 Owned object.
#[repr(transparent)]
pub struct Owned<T: RType> {
    sexp: SEXP,
    _marker: [PhantomData<T>; 0],
}
impl<T: RType> Owned<T> {
    /// This function provides a way to protect any vector from R's GC
    /// You should take care about that, any allocation might recycle the previous allocated vector
    /// In the current fasion, we only alloc 1 return vector, thus protect is not necessary
    /// In case you want to communicate with R functions, you must convert all your [Owned](Owned<T>) values to
    /// the [Protected](Protected<T>) version.
    pub fn protect(self) -> Protected<T> {
        self.into()
    }
}
/// Protected [`SEXP`], should not be transferred across FFI boundary.
///
/// Since return a protected vector back to R will cause memory leak, this struct is marked as private
/// The interface of [`Proteted<T>`](Self) and [`Owned<T>`] is almost the same (expect [`Owned<character>`] have an extra [`.error()`](Owned::error)
/// function call that handle error message, but such call is extremely unsafe, you should use [`panic`] instead.)
/// for FFI, you should only use
/// ```
/// extern "C" fn(Sexp<T1>, Sexp<T2>, ...)->Owned<TRet>
/// ```
/// and should never put [`Protected<T>`] in either side.
#[repr(transparent)]
pub struct Protected<T: RType> {
    sexp: SEXP,
    _marker: [PhantomData<T>; 0],
}
/// [`SEXP`] extensions
///
/// Contains all the operation that a normal SEXP could execute.
pub trait SExt: Sized {
    /// [`SEXP`] Associated data
    ///
    /// normally [`numeric`]->[`f64`], [`integer`]->[`i32`], [`logical`]->[`u32`], [`character`]->[`u8`], [`NULL`]->[`()`](unit) and [`SEXP`]->[`list`]
    ///
    /// check impls of [`RType`] for the full supported list
    type Data: RType;

    /// get the inner [`SEXP`].
    ///
    /// SAFETY:
    /// The [`SEXP`] object it yields could only be send into R FFI calls.
    /// You should not use [`SEXP`] in Rust interface.
    unsafe fn as_sexp(&self) -> SEXP;

    /// got [`SEXPTYPE`]((r_type::SEXPTYPE)) of the wrapped [`SEXP`] type.
    fn stype(&self)->SEXPTYPE {
        // SAFETY: sexp send directly to R FFI.
        unsafe {self.as_sexp().stype()}
    }

    /// allocate a new [`SEXP`] object with length, and protect it immeditely
    ///
    /// for [`u8`] ([`CHARSXP`]), new with length is not allowed
    /// thus for [`CHARSXP`], it accepts a &str object.
    fn new(len: usize) -> Protected<Self::Data>
    where
    Self::Data: RDefault,
    {
        Protected::<Self::Data> {
            // SAFETY: wrap and protect the sexp, thus safe.
            sexp: unsafe {<Self::Data as RType>::new(len).protect()},
            _marker: [],
        }
        .into()
    }

    /// allocate a new SEXP object with length, and does not protect it.
    ///
    /// for `u8` ([`CHARSXP`]), new with length is not allowed
    /// thus for [`CHARSXP`], it accepts a &str object.
    fn raw(len: usize) -> Owned<Self::Data>
    where
    Self::Data: RDefault,
    {
        Owned::<Self::Data> {
            sexp: unsafe {<Self::Data as RType>::new(len)},
            _marker: [],
        }
    }

    /// Create a protected R copy from rust object.
    ///
    /// The returned pointer is protected.
    fn from(data: impl core::convert::AsRef<[<Self::Data as RType>::Data]>) -> Protected<Self::Data>
    where
    Self::Data: RTypeFrom,
    {
        Protected::<Self::Data> {
            // SAFETY: wrap and protect the sexp, thus safe.
            sexp: unsafe {<Self::Data as RType>::from(data)},
            _marker: [],
        }
        .into()
    }

    /// Create a unprotected R copy from rust object.
    ///
    /// The returned pointer is unprotected.
    fn raw_from(data: impl core::convert::AsRef<[<Self::Data as RType>::Data]>) -> Owned<Self::Data>
    where
    Self::Data: RTypeFrom,
    {
        Owned::<Self::Data> {
            sexp: unsafe {<Self::Data as RType>::from(data)},
            _marker: [],
        }
    }

    /// check whether the data has the desired type
    #[inline(always)]
    fn is_correct_type(&self) -> bool {
        // SAFETY: stype is a R FFI call.
        self.stype() == Self::Data::SEXPTYPE
    }
    /// get length of this [`SEXP`].
    #[inline(always)]
    fn len(&self) -> usize {
        // SAFETY: stype is a R FFI call.
        unsafe {self.as_sexp().len() as usize}
    }
    /// got the read-only data of a [`SEXP`] object
    ///
    /// SAFETY: caller should ensure this [`SEXP`] has data with
    ///   1. passes `self.`[`is_correct_type`]()`()` check
    ///   2. has `self.`[`len`]()`()>0`
    #[inline(always)]
    unsafe fn data_unchecked(&self) -> &[<Self::Data as RType>::Data] {
        // SAFETY:
        //       : with SEXP.is_correct_type(), data is valid for len * mem::size_of::<T>() many bytes.
        //       : with Self::Data::len(sexp)>0, data is non-null and aligned.
        //       : the Data is Rtype thus Data is Copy, no drop would be called.
        //       : self.as_sexp() is sent to a ffi call.
        unsafe { core::slice::from_raw_parts(Self::Data::data(self.as_sexp()), self.len()) }
    }
    /// got the read-only data of a SEXP object
    #[inline(always)]
    fn data(&self) -> &[<Self::Data as RType>::Data] {
        if self.len() == 0 {
            &[]
        } else {
            if self.is_correct_type() {
                // SAFETY : len>0 and with correct type.
                unsafe { self.data_unchecked() }
            } else {
                panic!(
                    "data has the type {}, but {}(={}) is required.",
                    // SAFETY: FFI calls.
                    self.stype(),
                    core::any::type_name::<<Self::Data as RType>::Data>(),
                    <Self::Data as RType>::SEXPTYPE,
                )
            }
        }
    }
    /// got the read-only data of a [`SEXP`] object
    ///
    /// SAFETY: caller should ensure this [`SEXP`] has data with
    ///   1. passes `self.`[`is_correct_type`]()`()` check
    ///   2. has `self.`[`len`]()`()>0`
    #[inline(always)]
    unsafe fn data_unchecked_mut(&self) -> &mut [<Self::Data as RType>::Data]
    where
        <Self as SExt>::Data: RTypeMut,
        Self: Mutable,
    {
        // SAFETY:
        //       : with SEXP.is_correct_type(), data is valid for len * mem::size_of::<T>() many bytes.
        //       : with Self::Data::len(sexp)>0, data is non-null and aligned.
        //       : the Data is Rtype thus Data is Copy, no drop would be called.
        unsafe { core::slice::from_raw_parts_mut(Self::Data::data_mut(self.as_sexp()), self.len()) }
    }
    /// got the read-only data of a [`SEXP`] object
    #[inline(always)]
    fn data_mut(&mut self) -> &mut [<Self::Data as RType>::Data]
    where
        <Self as SExt>::Data: RTypeMut,
        Self: Mutable,
    {
        if self.len() == 0 {
            &mut []
        } else {
            if self.is_correct_type() {
                // SAFETY : len>0 and with correct type.
                unsafe { self.data_unchecked_mut() }
            } else {
                panic!(
                    "data has the type {}, but {}(={}) is required.",
                    // SAFETY: FFI calls.
                    self.stype(),
                    core::any::type_name::<Self::Data>(),
                    <Self::Data as RType>::SEXPTYPE,
                )
            }
        }
    }
}
/// marked [`SEXP`] as newable
pub trait Newable {}
impl<T: RType> Newable for Owned<T> {}
/// A marker suggeest whether the SEXP is mutable
///
/// Could not obtained manually since it is behind a `macro` invocation
///
/// # Example
/// ```
/// use rmin::Mutable;
/// ```
pub trait Mutable {}
impl<T: RTypeMut> Mutable for Owned<T> {}
impl<T: RTypeMut> Mutable for Protected<T> {}
impl<T: RType> From<Owned<T>> for Protected<T> {
    #[inline(always)]
    fn from(s: Owned<T>) -> Self {
        // SAFETY: FFI calls.
        Self {
            sexp: unsafe { s.sexp.protect() },
            _marker: [],
        }
    }
}
impl<T: RType> From<Protected<T>> for Owned<T> {
    #[inline(always)]
    fn from(s: Protected<T>) -> Self {
        Self {
            sexp: s.sexp,
            _marker: [],
        }
        // s is dropped and thus unprotected.
    }
}
impl<T: RType> From<Protected<T>> for Sexp<T> {
    #[inline(always)]
    fn from(s: Protected<T>) -> Self {
        Self {
            sexp: s.sexp,
            _marker: [],
        }
        // s is dropped and thus unprotected.
    }
}
impl<T: RType> From<Owned<T>> for Sexp<T> {
    #[inline(always)]
    fn from(s: Owned<T>) -> Self {
        Self {
            sexp: s.sexp,
            _marker: [],
        }
    }
}
impl<T: RType> Drop for Protected<T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { self.sexp.unprotect() }
    }
}
unsafe impl Sync for Sexp<character> {}
unsafe impl Send for Sexp<character> {}
impl Sexp<character> {
    /// REALLY UNSAFE FUNCTION
    ///
    /// see [`error`] for more informations.
    pub unsafe fn error(self)->!{
        unsafe {error(character::data(self.as_sexp()))}
    }
    #[cfg_attr(doc, doc(cfg(not(feature = "std"))))] #[cfg(not(feature = "std"))]
    /// print the content to R... with R's print function. Only available in no_std mode
    ///
    /// For std user, Rust builtin [`println!`]() could be better.
    pub fn print(self){
        // SAFETY: ffi calls, the input type is character with `\0` terminator, thus is OK.
        unsafe {print(character::data(self.as_sexp()))}
    }
}
impl_sext_index! {SExt=SExt SEXP=SEXP RType=RType RTypeMut=RTypeMut Mutable=Mutable, Sexp Owned Protected}
