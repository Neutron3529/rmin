#![allow(non_snake_case)]
use rmin::{*, println};
use std::sync::atomic::{AtomicI32, Ordering};
unsafe extern "C" {
    fn openblas_set_num_threads(i: i32);
    fn openblas_get_num_threads() -> i32;
    // fn omp_get_max_threads() -> i32;
}
/// @name corenum
/// @param logical whether return logical core count
/// @returns logical core count or physical core count
/// @examples
/// PS::cores(TRUE)
#[export]
pub fn corenum(
    logical: Sexp<logical>, // indicate whether threeparttable is used.
) -> Owned<integer> {
    // println!("{}", unsafe { openblas_get_num_threads() });
    if let Some(1) = logical.data().get(0) {
        Owned::raw_from([get() as i32]) // logical
    } else {
        Owned::raw_from([get_physical() as i32]) // physical
    }
}

static PHYSICAL: AtomicI32 = AtomicI32::new(-1);
static LOGICAL: AtomicI32 = AtomicI32::new(-1);
#[inline]
fn myget(val:&AtomicI32, get: fn()->usize)->i32{
    let mut cnt = val.load(Ordering::Relaxed);
    if cnt <= 0 {
        cnt = get() as i32;
        if cnt <= 0 {
            println!("Cannot got CPU count");
            return 0
        }
        val.store(cnt, Ordering::Relaxed);
    }
    cnt
}
fn get() -> i32 {
    myget(&LOGICAL, num_cpus::get)
}
fn get_physical() -> i32 {
    myget(&PHYSICAL, num_cpus::get_physical)
}
#[export]

/// @name set_threads
/// @param threads the parallel thread count.
/// @examples
/// PS::set_threads(1L)
pub fn set_threads(
    threads: Sexp<integer>, // indicate whether threeparttable is used.
) {
    // println!("{}", unsafe { openblas_get_num_threads() });
    if let Some(&num) = threads.data().get(0) {
        if num > 0 && num <= get() {
            unsafe {
                openblas_set_num_threads(num);
            }
        } else {
            println!("Cannot set the threads count more than {}", get());
        }
    } else {
        println!("Input `threads` does not exists or not valid.");
    }
}

/// @name get_threads
/// @param get current parallel threads
/// @examples
/// PS::get_threads()
#[export]
pub fn get_threads() -> Owned<integer> {
    unsafe{Owned::raw_from([openblas_get_num_threads()])}
}

done!();
