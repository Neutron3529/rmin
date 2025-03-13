/// Basic Data Type for R
///
/// Should be invisible for users
#[path = "base.s.r-type.rs"]
pub mod r_type;
use r_type::{
    alias::*,
    error,
    lib_r::{
        R_ClassSymbol, R_MakeExternalPtr, R_NilValue, Rf_setAttrib
    },
    RDefault, RType, RTypeFrom, RTypeMut, SEXPext, SEXP, SEXPTYPE,
};

#[cfg(doc)]
use r_type::define::*;
use r_type::{print, eprint}; // for doc;
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
use core::{ffi::c_void, marker::PhantomData};
use macros::impl_sext_index;
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

/// ReadOnly [`SEXP`], which might be missing.
///
/// The only thing you could do is [`.into_option()`], which check the missingness and then convert it into a regular [`Sexp`] in case it is not missing.
/// Currently, this is strongly discouraged since you cannot really send optional variable with registered routines.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct OptionSexp<T: RType> {
    sexp: Sexp<T>
}
impl<T: RType> Sexp<T> {
    /// Sexp means the item is not missing, using [`OptionSexp<T>`] if the sexp is optional
    /// (The best practice is, never use this!)
    #[deprecated = "Sexp<T> cannot missing (while the item is registered), using OptionSexp<T> instead."]
    pub const unsafe fn missing(self)->bool {
        false
    }
}
impl<T: RType> OptionSexp<T> {
    /// Convert the `OptionSexp<T>` into `Option<Sexp<T>>` item.
    /// should be unsafe since there is no guarateen whether the missing function could really being called.
    #[inline(always)]
    pub unsafe fn into_option(self)->Option<Sexp<T>> {
        if unsafe {self.missing()} {
            None
        } else {
            Some(self.sexp)
        }
    }

    /// indicate whether a sexp is missing, should call manually.
    /// This function is not provided to [`Owned`] or [`Protected`], since they are allocated by Rust and should not missing.
    #[inline(always)]
    pub unsafe fn missing(&self) -> bool {
        // only OptionSexp may missing.
        // SAFETY: ffi.
        self.missingness() != 0
    }
    /// wrapper for R `MISSING` function.
    #[inline(always)]
    pub unsafe fn missingness(&self) -> i32 {
        // only Sexp may missing.
        // SAFETY: ffi.
        unsafe { self.sexp.sexp.missing() as i32 }
    }
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
/// Convient trait that generate a null external ptr.
/// It is called R5, but still needs a lot of treatment from being real R5 class
pub trait R5: Sized + 'static {
    // /// The class R will used (if it is not empty)
    // const R_CLASS: &'static str = "";
    /// The class name R5 method will use.
    #[cfg(feature = "const_type_name")]
    const CLASS_NAME: &'static str = core::any::type_name::<Self>();
    /// The class name R5 method will use.
    #[cfg(not(feature = "const_type_name"))]
    const CLASS_NAME: &'static str;
    
    /// Create empty ExternalPtr
    fn new() -> Owned<externalptr> {
        #[cfg(feature = "create_new_class_symbol")]
        #[allow(non_snake_case)]
        let R_ClassSymbol = unsafe {
            Owned::raw_from_str("class")
                .protect()
                .as_sexp(Owned::raw_from_str(<Self as R5>::CLASS_NAME).protect())
        };
        let name = Owned::raw_from_str(<Self as R5>::CLASS_NAME).protect();
        let ret = Owned::<externalptr> {
            sexp: unsafe {
                R_MakeExternalPtr(core::ptr::null_mut() as *mut c_void, R_NilValue, R_NilValue)
            },
            _marker: [],
        }.protect();
        unsafe { Rf_setAttrib(ret.sexp, R_ClassSymbol, name.sexp) };
        ret.into()
    }
}

// /// a trait for handling R5 objects.
// /// This trait will convert Rust objects into R5 objects with registering its finalizer.
// pub trait ClassObject: R5 {
//     /// boxing self and converting it into externalptr
//     fn boxed_to_sexp(self) -> Owned<externalptr> {
//         Box::new(self).into_sexp()
//     }
//     /// convert boxed self into externalptr
//     fn into_sexp(self: Box<Self>) -> Owned<externalptr> {
//         #[cfg(feature = "create_new_class_symbol")]
//         #[allow(non_snake_case)]
//         let R_ClassSymbol = unsafe {
//             Owned::raw_from_str("class")
//                 .protect()
//                 .as_sexp(Owned::raw_from_str(<Self as R5>::CLASS_NAME).protect())
//         };
//         let name = Owned::raw_from_str(<Self as R5>::CLASS_NAME).protect();
//         let ret = Owned::<externalptr> {
//             sexp: unsafe {
//                 R_MakeExternalPtr(Box::into_raw(self) as *mut c_void, R_NilValue, R_NilValue)
//             },
//             _marker: [],
//         }
//         .protect();
//         unsafe { R_RegisterCFinalizer(ret.sexp, <Self as ClassObject>::fin) }
//         unsafe { Rf_setAttrib(ret.sexp, R_ClassSymbol, name.sexp) };
//         ret.into()
//     }
//     /// Finalizer, might not be called.
//     extern "C" fn fin(item: SEXP) {
//         #[cfg(feature = "create_new_class_symbol")]
//         #[allow(non_snake_case)]
//         let R_ClassSymbol = unsafe { Owned::raw_from_str("class").protect().as_sexp() };
//         let class : Owned<character> = unsafe {Owned{sexp:item.get_attr(R_ClassSymbol),_marker:[]}};

//         if class.len() == 0 {
//             panic!("Expect class {CLASS_NAME}, but an object with no class specific is sent to the finalizer!", CLASS_NAME = <Self as R5>::CLASS_NAME);
//         } else if let Ok(cls) = core::str::from_utf8(class[0].data()) {
//             if cls != <Self as R5>::CLASS_NAME {
//                 panic!("Expect class {CLASS_NAME}, but class[0] = {cls} is sent to the finalizer!", CLASS_NAME = <Self as R5>::CLASS_NAME)
//             } else {
//                 // SAFETY: the pointer is created from Rust and not dropped (since into_sexp requires it directly)
//                 // Thus dereference it is safe.
//                 drop(unsafe { Box::from_raw(R_ExternalPtrAddr(item) as *mut Self) }) // then safely drops it.
//             }
//         } else {
//             panic!("Expect class {CLASS_NAME}, but class[0] cannot convert to valid utf8 str", CLASS_NAME = <Self as R5>::CLASS_NAME)
//         }
//     }
// }

/// Protected [`SEXP`], should not be transferred across FFI boundary.
///
/// Since return a protected vector back to R will cause memory leak, this struct is marked as private
/// The interface of [`Proteted<T>`](Self) and [`Owned<T>`] is almost the same, but
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
    /// normally [`numeric`]->[`f64`], [`integer`]->[`i32`], [`logical`]->[`u32`], [`Rchar`]->[`u8`], [`NULL`]->[`()`](unit) and [`SEXP`]->[`list`]
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
    fn stype(&self) -> SEXPTYPE {
        // SAFETY: sexp send directly to R FFI.
        unsafe { self.as_sexp().stype() }
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
            sexp: unsafe { <Self::Data as RType>::new(len).protect() },
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
            sexp: unsafe { <Self::Data as RType>::new(len) },
            _marker: [],
        }
    }

    /// Create a protected R copy from rust object.
    ///
    /// The returned pointer is protected.
    fn from(data: impl AsRef<[<Self::Data as RType>::Data]>) -> Protected<Self::Data>
    where
        Self::Data: RTypeFrom,
    {
        Protected::<Self::Data> {
            // SAFETY: wrap and protect the sexp, thus safe.
            sexp: unsafe { <Self::Data as RType>::from(data) },
            _marker: [],
        }
        .into()
    }

    /// Create a unprotected R copy from rust object.
    ///
    /// The returned pointer is unprotected.
    fn raw_from(data: impl AsRef<[<Self::Data as RType>::Data]>) -> Owned<Self::Data>
    where
        Self::Data: RTypeFrom,
    {
        Owned::<Self::Data> {
            sexp: unsafe { <Self::Data as RType>::from(data) },
            _marker: [],
        }
    }

    // /// Create a list like object
    // ///
    // /// The returned pointer is unprotected.
    // unsafe fn from_protected(data: Protected<Self::Data>) -> (Owned<Sexp<<Self as SExt>::Data>>, Protected<Self::Data>)
    // where
    // Self::Data: RTypeFrom,
    // Sexp<<Self as SExt>::Data>:RDefault+RTypeMut,
    // Sexp<<Self as SExt>::Data> : AsRef<<Sexp<<Self as SExt>::Data> as RType>::Data>,
    // [Sexp<<Self as SExt>::Data>] : AsRef<[<Sexp<<Self as SExt>::Data> as RType>::Data]>
    // {
    //     let sexp = unsafe { Sexp::<<Self as SExt>::Data>{ sexp:data.as_sexp(),_marker:[] }};
    //     (Owned::<Sexp<Self::Data>> {
    //         // <Sexp<<Self as s::SExt>::Data> as RType>::Data
    //         // Sexp<<Self as s::SExt>::Data>
    //         sexp: unsafe {<Sexp<Self::Data> as RType>::from([sexp].as_slice())},
    //         _marker: [],
    //     },data)
    // }

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
        unsafe { self.as_sexp().len() as usize }
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
unsafe impl Sync for Sexp<Rchar> {}
unsafe impl Send for Sexp<Rchar> {}
impl Sexp<Rchar> {
    /// REALLY UNSAFE FUNCTION
    ///
    /// see [`error`] for more informations.
    pub unsafe fn error(self) -> ! {
        unsafe { error(Rchar::data(self.as_sexp())) }
    }
    /// print the content to R... with R's print function.
    ///
    /// It is more preferred than `std::println!` since output with `std::println!` might be ignored with windows `Rgui.exe`.
    pub fn print(self) {
        // SAFETY: ffi calls, the input type is Rchar with `\0` terminator, thus is OK.
        unsafe { print(Rchar::data(self.as_sexp())) }
    }
    /// print the content to R... with R's eprint function.
    pub fn eprint(self) {
        // SAFETY: ffi calls, the input type is Rchar with `\0` terminator, thus is OK.
        unsafe { eprint(Rchar::data(self.as_sexp())) }
    }
}

impl Protected<character> {
    fn raw_from_strs(a: impl ExactSizeIterator<Item:AsRef<[u8]>>) -> Self {
        let mut ret = Owned{sexp: unsafe {<character as RType>::new(a.len())}, _marker:[]}.protect();
        let item = ret.data_mut();
        item.iter_mut().zip(a).for_each(|(i,a)|*i=Owned::raw_from(a).into());
        ret
    }
}

impl Owned<character> {
    /// simple wrapper for chants that could generate a STRSXP (with length 1) quickly.
    pub fn raw_from_strs(a: impl ExactSizeIterator<Item:AsRef<[u8]>>) -> Self {
        Protected::<character>::raw_from_strs(a).into()
    }
    /// simple wrapper for chants that could generate a STRSXP (with length 1) quickly.
    pub fn raw_from_str(a: impl AsRef<[u8]>) -> Self {
        Self::raw_from([<Owned<Rchar> as Into<Sexp<Rchar>>>::into(Owned::raw_from(
            a,
        ))])
    }
    /// Nothing more than a notation.
    #[deprecated(
        since = "0.0.0",
        note = "Please use `raw_from_str` to ensure you are generate a `raw Owned` rather than `normal protected` type."
    )]
    pub fn from_str(a: impl AsRef<[u8]>) -> Self {
        Self::raw_from_str(a)
    }
}
impl_sext_index! {SExt=SExt SEXP=SEXP RType=RType RTypeMut=RTypeMut Mutable=Mutable, Sexp Owned Protected}
