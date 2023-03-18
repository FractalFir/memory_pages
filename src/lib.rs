#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]
mod extern_fn_ptr;
use core::fmt::Pointer;
use extern_fn_ptr::ExternFnPtr;
use std::borrow::{Borrow, BorrowMut};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
// TODO: maybe make allow/deny for all `Pages`?
#[cfg(target_family = "windows")]
use winapi::um::memoryapi::*;
#[cfg(target_family = "windows")]
use winapi::um::winnt::{
    MEM_COMMIT, MEM_RELEASE, PAGE_EXECUTE, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE,
    PAGE_NOACCESS, PAGE_READONLY, PAGE_READWRITE,
};
const fn next_page_boundary(size:usize)->usize{
    ((size + PAGE_SIZE - 1)/PAGE_SIZE)*PAGE_SIZE
}
const PAGE_SIZE: usize = 0x1000;
#[cfg(target_family = "unix")]
const MAP_ANYNOMUS: c_int = 0x20;
#[cfg(target_family = "unix")]
const MAP_PRIVATE: c_int = 0x2;
#[cfg(target_family = "unix")]
const NO_FILE: c_int = -1;
#[cfg(target_family = "unix")]
use std::ffi::{c_int, c_void};
#[cfg(target_family = "unix")]
extern "C" {
    fn mmap(
        addr: *mut c_void,
        length: usize,
        prot: c_int,
        flags: c_int,
        fd: c_int,
        offset: usize,
    ) -> *mut c_void;
    fn munmap(addr: *mut c_void, length: usize) -> c_int;
    fn mprotect(addr: *mut c_void, len: usize, prot: c_int) -> c_int;
    fn strerror(errnum: c_int) -> *const i8;
}
/// Marks if a [`Pages`] can be read from.
pub trait ReadPremisionMarker {
    #[cfg(all(target_family = "unix"))]
    #[doc(hidden)]
    fn bitmask() -> c_int;
    #[cfg(target_family = "windows")]
    #[doc(hidden)]
    fn allow_read() -> bool;
}
/// Marks if a [`Pages`] can be written into.
pub trait WritePremisionMarker {
    #[cfg(target_family = "unix")]
    #[doc(hidden)]
    fn bitmask() -> c_int;
    #[cfg(target_family = "windows")]
    #[doc(hidden)]
    fn allow_write() -> bool;
}
/// Marks if native CPU instructions stored inside [`Pages`] can jumped to and executed. 
pub trait ExecPremisionMarker {
    #[cfg(target_family = "unix")]
    #[doc(hidden)]
    fn bitmask() -> c_int;
    #[cfg(target_family = "windows")]
    #[doc(hidden)]
    fn allow_exec() -> bool;
}
/// Marks [`Pages`] as allowing to be read from.
pub struct AllowRead;
impl ReadPremisionMarker for AllowRead {
    #[cfg(target_family = "unix")]
    fn bitmask() -> c_int {
        0x1
    }
    #[cfg(target_family = "windows")]
    fn allow_read() -> bool {
        true
    }
}
/// Marks [`Pages`] as forbidding all reads(causing SIGSEGV if read attempted).
pub struct DenyRead;
impl ReadPremisionMarker for DenyRead {
    #[cfg(target_family = "unix")]
    fn bitmask() -> c_int {
        0
    }
    #[cfg(target_family = "windows")]
    fn allow_read() -> bool {
        false
    }
}
/// Marks [`Pages`] as allowing to be modified.
pub struct AllowWrite;
impl WritePremisionMarker for AllowWrite {
    #[cfg(target_family = "unix")]
    fn bitmask() -> c_int {
        0x2
    }
    #[cfg(target_family = "windows")]
    fn allow_write() -> bool {
        true
    }
}
/// Marks [`Pages`] as forbidding all writes(causing SIGSEGV if write attempted).
pub struct DenyWrite;
impl WritePremisionMarker for DenyWrite {
    #[cfg(target_family = "unix")]
    fn bitmask() -> c_int {
        0
    }
    #[cfg(target_family = "windows")]
    fn allow_write() -> bool {
        false
    }
}
/// Marks [`Pages`] as allowing execution.
/// **WARNING** do *NOT* set this permission if not necessary!
/// # Safety
/// Set [`AllowExec`] permission  only if you can be sure that:
/// 1. Native instructions inside this Pages are 100% safe
/// 2. Native instructions inside this Pages may only ever be changed by a 100% safe code. Preferably, set Pages to allow execution only when writes are disabled. To do this flip in one call, use [`Pages::set_protected_exec`].  
pub struct AllowExec;
impl ExecPremisionMarker for AllowExec {
    #[cfg(target_family = "unix")]
    fn bitmask() -> c_int {
        0x4
    }
    #[cfg(target_family = "windows")]
    fn allow_exec() -> bool {
        true
    }
}
/// Prevents data inside [`Pages`] from being executed. Do *NOT* change from this value if not 100% sure what you are doing.
pub struct DenyExec;
impl ExecPremisionMarker for DenyExec {
    #[cfg(target_family = "unix")]
    fn bitmask() -> c_int {
        0
    }
    #[cfg(target_family = "windows")]
    fn allow_exec() -> bool {
        false
    }
}
/// A bunch of memory pages acquired directly from the kernel, contiguously laid out in memory, with certain access permissions set.
pub struct Pages<R: ReadPremisionMarker, W: WritePremisionMarker, E: ExecPremisionMarker> {
    ptr: *mut u8,
    len: usize,
    read: PhantomData<R>,
    write: PhantomData<W>,
    exec: PhantomData<E>,
}
#[cfg(target_family = "unix")]
fn erno() -> c_int {
    #[cfg(any(target_os = "linux",target_os = "redox"))]
    {
        extern "C" {
            fn __errno_location() -> *mut c_int;
        }
        unsafe { *__errno_location() }
    }
    #[cfg(any(target_os = "solaris",target_os = "illumos"))]
    {
        extern "C" {
            fn ___errno() -> *mut c_int;
        }
        unsafe { *___errno() }
    }
    #[cfg(any(target_os = "macos",target_os = "ios",target_os = "freebsd"))]
    {
        extern "C" {
            fn __error() -> *mut c_int;
        }
        unsafe { *__error() }
    }
}
#[cfg(target_family = "unix")]
fn errno_msg() -> String {
    let cstr = unsafe { std::ffi::CStr::from_ptr(strerror(erno())) };
    String::from_utf8_lossy(cstr.to_bytes()).to_string()
}
impl<R: ReadPremisionMarker, W: WritePremisionMarker, E: ExecPremisionMarker> Pages<R, W, E> {
    #[cfg(target_family = "unix")]
    fn bitmask() -> c_int {
        R::bitmask() | W::bitmask() | E::bitmask()
    }
    #[cfg(target_family = "windows")]
    fn flProtect() -> u32 {
        let mask = (R::allow_read() as u8 * 0x1)
            | (W::allow_write() as u8 * 0x2)
            | (E::allow_exec() as u8 * 0x4);
        match mask {
            0x0 => PAGE_NOACCESS,
            0x1 => PAGE_READONLY,
            0x2 => PAGE_READWRITE, //On windows, it is impossible to have a write-only page, but `Pages` must have
            // AllowRead to be read from, so there are no issues here.
            0x3 => PAGE_READWRITE,
            0x4 => PAGE_EXECUTE,
            0x5 => PAGE_EXECUTE_READ,
            0x6 => PAGE_EXECUTE_READWRITE, //On windows, it is impossible to have a write but not read page, but `Pages` already
            // must have AllowRead to be read from, so there are no issues here.
            0x7 => PAGE_EXECUTE_READWRITE,
            0x8..=0xFF => panic!("Invalid protection mask:{mask}"),
        }
    }
    /// Allocates new [`Pages`] of size at least length, rounded up to next Page boundary if necessary.
    /// # Panics
    /// Panics when a 0-sized allocation is attempted, or if kernel can't/refuses to allocate requested Pages(Should never happen).
    /// # Examples
    /// Allocating pages works with sizes divisible by size of the page:
    ///```
    /// # use pages::*;
    /// let memory:Pages<AllowRead,AllowWrite,DenyExec> = Pages::new(0x8000);
    /// assert_eq!(memory.len(),0x8000);
    ///```
    /// And allocation sized not divisible by the size of the page:
    ///```
    /// # use pages::*;
    /// let memory:Pages<AllowRead,AllowWrite,DenyExec> = Pages::new(0x1234);
    /// // Rounds up to the next page boundary, so that length of the actual allocation
    /// // may never be less than requested length.
    /// assert_eq!(memory.len(),0x2000);
    ///```
    /// 0-sized allocations will always fail.
    /// ```should_panic
    /// # use pages::*;
    /// let memory:Pages<AllowRead,AllowWrite,DenyExec> = Pages::new(0);
    ///```
    #[must_use]
    pub fn new(length: usize) -> Self {
        Self::new_native(length)
    }
    #[cfg(target_family = "windows")]
    fn new_native(length: usize) -> Self {
        assert_ne!(length, 0, "0 - sized allcations are not allowed!");
        let len = next_page_boundary(length);
        let ptr =
            unsafe { VirtualAlloc(std::ptr::null_mut(), length, MEM_COMMIT, Self::flProtect()) }
                .cast::<u8>();
        if ptr as usize == 0 {
            let err = unsafe { winapi::um::errhandlingapi::GetLastError() };
            panic!("Allocation using VirtualAlloc failed with error code:{err}!");
        }
        Self {
            ptr,
            len,
            read: PhantomData,
            write: PhantomData,
            exec: PhantomData,
        }
    }
    #[cfg(target_family = "unix")]
    fn new_native(length: usize) -> Self {
        assert_ne!(length, 0, "0 - sized allcations are not allowed!");
        let len = next_page_boundary(length);
        let prot_mask = Self::bitmask();
        let ptr = unsafe {
            mmap(
                std::ptr::null_mut(),
                len,
                prot_mask,
                MAP_ANYNOMUS | MAP_PRIVATE,
                NO_FILE,
                0,
            )
        }
        .cast::<u8>();
        if ptr as usize == usize::MAX {
            let erno = errno_msg();
            panic!("mmap error, erno:{erno:?}!");
        }
        Self {
            ptr,
            len,
            read: PhantomData,
            write: PhantomData,
            exec: PhantomData,
        }
    }
    #[cfg(target_family = "unix")]
    fn set_prot(&mut self) {
        let mask = Self::bitmask();
        if unsafe { mprotect(self.ptr.cast::<c_void>(), self.len, mask) } != -1 && erno() != 0 {
            let err = errno_msg();
            panic!("Failed to change memory protection mode:'{err}'!");
        }
    }
    #[cfg(target_family = "windows")]
    fn set_prot(&mut self) {
        let mut _old: u32 = 0;
        let res = unsafe {
            winapi::um::memoryapi::VirtualProtect(
                self.ptr.cast::<winapi::ctypes::c_void>(),
                self.len,
                Self::flProtect(),
                &mut _old as *mut _,
            )
        };
        if res == 0 {
            let err = unsafe { winapi::um::errhandlingapi::GetLastError() };
            panic!("Changing memory protection using using VirtualProtect failed with error code:{err}!");
        }
    }
    fn into_prot<TR: ReadPremisionMarker, TW: WritePremisionMarker, TE: ExecPremisionMarker>(
        self,
    ) -> Pages<TR, TW, TE> {
        let mut res = Pages {
            ptr: self.ptr,
            len: self.len,
            read: PhantomData,
            write: PhantomData,
            exec: PhantomData,
        };
        std::mem::forget(self);
        res.set_prot();
        res
    }
}

impl<W: WritePremisionMarker, E: ExecPremisionMarker> std::ops::Index<usize>
    for Pages<AllowRead, W, E>
{
    type Output = u8;
    fn index(&self, index: usize) -> &u8 {
        let slice: &[u8] = self;
        &slice[index]
    }
}
impl<W: WritePremisionMarker, E: ExecPremisionMarker> Borrow<[u8]> for Pages<AllowRead, W, E> {
    fn borrow(&self) -> &[u8] {
        self
    }
}
impl<W: WritePremisionMarker, E: ExecPremisionMarker> Deref for Pages<AllowRead, W, E> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}
impl<E: ExecPremisionMarker> DerefMut for Pages<AllowRead, AllowWrite, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}
impl<E: ExecPremisionMarker> BorrowMut<[u8]> for Pages<AllowRead, AllowWrite, E> {
    fn borrow_mut(&mut self) -> &mut [u8] {
        self
    }
}
impl<E: ExecPremisionMarker> std::ops::IndexMut<usize> for Pages<AllowRead, AllowWrite, E> {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        unsafe { &mut std::slice::from_raw_parts_mut(self.ptr, self.len)[index] }
    }
}
impl<R: ReadPremisionMarker, W: WritePremisionMarker> Pages<R, W, AllowExec> {
    /// Returns a pointer to executable code at *offset*. Works similary to getting a pointer using [`Self::get_ptr`] or
    /// [`Self::get_ptr_mut`] but ensures that execute permission is set to allow(if not this function is unavailable), and
    /// clearly conveys the intent of programmer. Returned pointer may not be read from/written into, can be only cast as a function pointer.
    /// # Panics
    /// Will panic if offset larger than length.
    /// # Examples
    /// Getting a pointer with offset smaller than length of pages is OK.
    ///```
    /// # use pages::*;
    /// let memory:Pages<DenyRead,DenyWrite,AllowExec> = Pages::new(0x1000);
    /// let ptr = memory.get_fn_ptr(0);
    /// let ptr2 = memory.get_fn_ptr(2);
    ///```
    /// Getting a pointer with offset greater than length of Pages causes a panic.
    ///```should_panic
    /// # use pages::*;
    /// let memory:Pages<DenyRead,DenyWrite,AllowExec> = Pages::new(0x1000);
    /// let ptr = memory.get_fn_ptr(0x1000);
    /// let ptr = memory.get_fn_ptr(0x1001);
    ///```
    /// Defencing a pointer acquired from calling `get_fn_ptr` on `Pages` with DenyRead is an UB and may cause a segfault on some systems.
    ///```should_panic
    /// # use pages::*;
    /// let memory:Pages<DenyRead,DenyWrite,AllowExec> = Pages::new(0x1000);
    /// let ptr = memory.get_fn_ptr(0x1);
    /// let some_data:u8 = unsafe{*(ptr as *const u8)};
    /// # panic!("This may or may not be illegal");
    ///```
    #[must_use] 
    pub fn get_fn_ptr(&self, offset: usize) -> *const () {
        unsafe { std::ptr::addr_of!(std::slice::from_raw_parts(self.ptr, self.len)[offset]).cast() }
    }
    /// Gets a pointer to function at offset in [`Pages`]. Function must be an `extern "C" fn`.
    /// # Safety
    /// The bytes at offset must represent native instructions creating a function with a matching signature to function pointer
    /// type  F.
    /// # Panics
    /// Will panic if offset larger than length.
    /// # Example
    /// A function that just returns, and does nothing. This example is architecture specific.
    /// ```no_run
    /// # use pages::*; 
    /// let mut memory:Pages<AllowRead,AllowWrite,DenyExec> = Pages::new(0x4000);
    /// // X86_64 assembly instruction `RET`
    /// memory[0] = 0xC3;
    /// let memory = memory.set_protected_exec();
    /// let nop:extern "C" fn() = unsafe{memory.get_fn(0)};
    /// nop();
    /// ```
    /// A function that adds 2 numbers. It is architecture specific, and works on x86_64 linux.
    /// ```no_run
    /// # use pages::*; 
    /// let mut memory:Pages<AllowRead,AllowWrite,DenyExec> = Pages::new(0x4000);
    /// // encoded X86_64 assembly for adding 2 numbers
    /// memory[0] = 0x48;
    /// memory[1] = 0x8d;
    /// memory[2] = 0x04;
    /// memory[3] = 0x37;
    /// memory[4] = 0xC3;
    /// let memory = memory.set_protected_exec();
    /// let add:extern "C" fn(u64,u64)->u64 = unsafe{memory.get_fn(0)};
    /// assert_eq!(add(43,34),77);
    /// ```
    #[must_use]
    pub unsafe fn get_fn<F: ExternFnPtr>(&self, offset: usize) -> F
    where
        F: Copy + Pointer + Sized,
    {
        let fn_ptr = self.get_fn_ptr(offset);
        let f:F = *(std::ptr::addr_of!(fn_ptr).cast::<F>());
        let _ = fn_ptr;
        f
    }
}
impl<R: ReadPremisionMarker, W: WritePremisionMarker> Pages<R, W, AllowExec> {
    /// Sets the permission on [`Pages`] to [`DenyExec`], forbidding execution.
    #[must_use]
    pub fn deny_exec(self) -> Pages<R, W, DenyExec> {
        self.into_prot()
    }
}
impl<R: ReadPremisionMarker, W: WritePremisionMarker> Pages<R, W, DenyExec> {
    /// Sets the permission on [`Pages`] to [`AllowExec`], allowing execution.
    /// # Safety
    /// This should **NEVER** be set if not needed, because if used improperly, it may lead to Arbitrary Code Execution 
    /// exploits. Use *only* if you know what you are doing. [`Self::set_protected_exec`] is a safer alternative, that prevents 
    /// most ways an ACE exploit could occur.
    #[must_use]
    pub fn allow_exec(self) -> Pages<R, W, AllowExec> {
        self.into_prot()
    }
    /// Sets the permission on [`Pages`] to [`AllowExec`] and [`DenyWrite`] to prevent changing of instructions inside      
    /// [`Pages`]. To re-enable writes, use [`Self::allow_write_no_exec`] to ensure both [`AllowExec`] and [`AllowExec`] are 
    /// never set at the same time.
    #[must_use]
    pub fn set_protected_exec(self) -> Pages<R, DenyWrite, AllowExec> {
        self.into_prot()
    }
}
impl<R: ReadPremisionMarker, E: ExecPremisionMarker> Pages<R, DenyWrite, E> {
    /// Allows writing to this page. If dealing with executable pages(`AllowExecute`) use [`Self::allow_write_no_exec`] for additional safety.
    #[must_use]
    pub fn allow_write(self) -> Pages<R, AllowWrite, E> {
        self.into_prot()
    }
    #[must_use]
    /// Sets the [`AllowExec`], while ensuring that the [`DenyExec`] is set, to prevent potential mistakes. 
    /// Preferred over [`Self::allow_write`] if dealing with executable pages, otherwise just use [`Self::allow_write`]. 
    pub fn allow_write_no_exec(self) -> Pages<R, AllowWrite, DenyExec> {
        self.into_prot()
    }
}
impl<R: ReadPremisionMarker, E: ExecPremisionMarker> Pages<R, AllowWrite, E> {
    /// Sets the [`DenyWrite`], making data inside this [`Pages`] immutable.
    #[must_use]
    pub fn deny_write(self) -> Pages<R, DenyWrite, E> {
        self.into_prot()
    }
}
impl<W: WritePremisionMarker, E: ExecPremisionMarker> Pages<DenyRead, W, E> {
    /// Sets the [`AllowWrite`], making data inside this [`Pages`] mutable.
    #[must_use]
    pub fn allow_read(self) -> Pages<AllowRead, W, E> {
        self.into_prot()
    }
}
impl<W: WritePremisionMarker, E: ExecPremisionMarker> Pages<AllowRead, W, E> {
    /// Sets the [`DenyRead`], making data inside page unreadable.
    #[must_use]
    pub fn deny_read(self) -> Pages<DenyRead, W, E> {
        self.into_prot()
    }
}
impl<W: WritePremisionMarker, E: ExecPremisionMarker> Pages<AllowRead, W, E> {
    /// Sets the [`AllowRead`], making data inside page readable. 
    /// # Panics
    /// Panics if offset larger than length of [`Pages`].
    #[must_use]
    pub unsafe fn get_ptr(&self, offset: usize) -> *const u8 {
        std::ptr::addr_of!(self[offset])
    }
}
impl<R: ReadPremisionMarker, E: ExecPremisionMarker> Pages<R, AllowWrite, E> {
    /// Gets a pointer to data inside page at `offset`.
    /// # Safety
    /// This pointer may be only written into, and while reading data from it may work on some systems, it is an UB which may cause crashes.
    pub unsafe fn get_ptr_mut(&mut self, offset: usize) -> *mut u8 {
        unsafe {
            std::ptr::addr_of_mut!(std::slice::from_raw_parts_mut(self.ptr, self.len)[offset])
        }
    }
}
impl<R: ReadPremisionMarker, W: WritePremisionMarker, E: ExecPremisionMarker> Drop
    for Pages<R, W, E>
{
    fn drop(&mut self) {
        #[cfg(target_family = "unix")]
        unsafe {
            let res = munmap(self.ptr.cast::<c_void>(), self.len);
            if res == -1 {
                let err = errno_msg();
                panic!("Unampping memory Pages failed. Reason:{err}");
            }
        }
        #[cfg(target_family = "windows")]
        unsafe {
            let res = VirtualFree(self.ptr.cast::<winapi::ctypes::c_void>(), 0, MEM_RELEASE);
            if res == 0 {
                let err = winapi::um::errhandlingapi::GetLastError();
                panic!("Allocation using VirtualFree failed with error code:{err}!");
            }
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_alloc_rwe() {
        let _pages: Pages<AllowRead, AllowWrite, AllowExec> = Pages::new(256);
    }
    #[test]
    fn test_alloc_rw() {
        let _pages: Pages<AllowRead, AllowWrite, DenyExec> = Pages::new(256);
    }
    #[test]
    fn test_alloc_r() {
        let _pages: Pages<AllowRead, DenyWrite, DenyExec> = Pages::new(256);
    }
    #[test]
    fn test_alloc_e() {
        let _pages: Pages<DenyRead, DenyWrite, AllowExec> = Pages::new(256);
    }
    #[test]
    fn test_alloc_re() {
        let _pages: Pages<AllowRead, DenyWrite, AllowExec> = Pages::new(256);
    }
    #[test]
    fn test_acces_rw() {
        let mut pages: Pages<AllowRead, AllowWrite, DenyExec> = Pages::new(256);
        for i in 0..256 {
            pages[i] = i as u8;
        }
        for i in 0..256 {
            assert_eq!(pages[i], i as u8);
        }
    }
    #[test]
    fn test_acces_r() {
        let pages: Pages<AllowRead, DenyWrite, DenyExec> = Pages::new(256);
        for i in 0..256 {
            assert_eq!(pages[i], 0);
        }
    }
    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_exec() {
        let mut pages: Pages<AllowRead, AllowWrite, AllowExec> = Pages::new(256);
        //NOP
        pages[0] = 0xC3;
        //Add 2 u64s
        #[cfg(target_family = "unix")]
        {
            pages[1] = 0x48;
            pages[2] = 0x8d;
            pages[3] = 0x04;
            pages[4] = 0x37;
            pages[5] = 0xC3;
        }
        #[cfg(target_family = "windows")]
        {
            pages[1] = 0x8d;
            pages[2] = 0x04;
            pages[3] = 0x11;
            pages[4] = 0xC3;
        }
        let nop: extern "C" fn() = unsafe { pages.get_fn(0) };
        nop();
        let add: extern "C" fn(u64, u64) -> u64 = unsafe { pages.get_fn(1) };
        for i in 0..256 {
            for j in 0..256 {
                assert_eq!(i + j, add(i, j));
            }
        }
    }
    #[test]
    fn test_allow_read() {}
    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_allow_exec() {
        let mut pages: Pages<AllowRead, AllowWrite, DenyExec> = Pages::new(256);
        //NOP
        pages[0] = 0xC3;
        //Add 2 u64s
        #[cfg(target_family = "unix")]
        {
            pages[1] = 0x48;
            pages[2] = 0x8d;
            pages[3] = 0x04;
            pages[4] = 0x37;
            pages[5] = 0xC3;
        }
        #[cfg(target_family = "windows")]
        {
            pages[1] = 0x8d;
            pages[2] = 0x04;
            pages[3] = 0x11;
            pages[4] = 0xC3;
        }
        let pages = pages.allow_exec().deny_write();
        let nop: extern "C" fn(()) = unsafe { pages.get_fn(0) };
        nop(());
        let add: extern "C" fn(u64, u64) -> u64 = unsafe { pages.get_fn(1) };
        for i in 0..256 {
            for j in 0..256 {
                assert_eq!(i + j, add(i, j));
            }
        }
    }
}
