use std::marker::PhantomData;
const PAGE_SIZE:usize = 0x1000;
#[cfg(target_family = "unix")]
const MAP_ANYNOMUS:c_int = 0x20;
const MAP_PRIVATE:c_int = 0x2;
#[cfg(target_family = "unix")]
const NO_FILE:c_int = -1;
pub trait ReadPremisionMarker {
    #[cfg(target_family = "unix")]
    fn bitmask()->c_int;
}
pub trait WritePremisionMarker {
    #[cfg(target_family = "unix")]
    fn bitmask()->c_int;
}
pub trait ExecPremisionMarker {
    #[cfg(target_family = "unix")]
    fn bitmask()->c_int;
}
pub struct AllowRead;
impl ReadPremisionMarker for AllowRead{
    #[cfg(target_family = "unix")]
    fn bitmask()->c_int{
        0x1
    }
}
pub struct DenyRead;
impl ReadPremisionMarker for DenyRead{
    #[cfg(target_family = "unix")]
    fn bitmask()->c_int{
        0
    }
}
pub struct AllowWrite;
impl WritePremisionMarker for AllowWrite{
    #[cfg(target_family = "unix")]
    fn bitmask()->c_int{
        0x2
    }
}
pub struct DenyWrite;
impl WritePremisionMarker for DenyWrite{
    #[cfg(target_family = "unix")]
    fn bitmask()->c_int{
        0
    }
}
pub struct AllowExec;
impl ExecPremisionMarker for AllowExec{
    #[cfg(target_family = "unix")]
    fn bitmask()->c_int{
        0x4
    }
}
pub struct DenyExec;
impl ExecPremisionMarker for DenyExec{
    #[cfg(target_family = "unix")]
    fn bitmask()->c_int{
        0
    }
}
use std::ffi::{c_void,c_int};
#[cfg(target_family = "unix")]
extern "C"{
    fn mmap(addr:*mut c_void,length:usize,prot:c_int,flags:c_int,fd:c_int,offset:usize)->*mut c_void;
    fn munmap(addr:*mut c_void,length:usize)->c_int;
    fn mprotect(addr:*mut c_void,len:usize,prot:c_int)->c_int;
    fn strerror(errnum:c_int)->*const i8;
}
pub struct Page<R:ReadPremisionMarker,W:WritePremisionMarker,E:ExecPremisionMarker> {
    ptr: *mut u8,
    len: usize,
    read: PhantomData<R>,
    write: PhantomData<W>,
    exec: PhantomData<E>,
}
#[cfg(target_family = "unix")]
fn erno()->c_int{
    #[cfg(target_os="linux")]
    {
        extern "C" {fn __errno_location()->*mut c_int;}
        unsafe{*__errno_location()}
    }
}
#[cfg(target_family = "unix")]
fn errno_msg()->String{
    let cstr = unsafe{std::ffi::CStr::from_ptr(strerror(erno()))};
    String::from_utf8_lossy(cstr.to_bytes()).to_string()
}
impl<R:ReadPremisionMarker,W:WritePremisionMarker,E:ExecPremisionMarker> Page<R,W,E>{
    #[cfg(target_family = "unix")]
    fn bitmask()->c_int{
        R::bitmask() | W::bitmask() | E::bitmask()
    }
    #[cfg(target_family = "unix")]
    pub fn new(length:usize)->Self{
        assert_ne!(length,0,"0 - sized allcations are not allowed!");
        let len = (length/PAGE_SIZE + 1)*PAGE_SIZE;
        let prot_mask = Self::bitmask();
        let ptr = unsafe{mmap(std::ptr::null_mut(),len,prot_mask,MAP_ANYNOMUS|MAP_PRIVATE,NO_FILE,0)} as *mut u8;
        if ptr as usize == usize::MAX{
            let erno = errno_msg();
            panic!("mmap error, erno:{erno:?}!");
        }
        Self{ptr,len,read:PhantomData,write:PhantomData,exec:PhantomData}
    }
    #[cfg(target_family = "unix")]
    fn set_prot(&mut self){
        let mask = Self::bitmask();
        if unsafe{mprotect(self.ptr as *mut c_void,self.len,mask)} != -1 && erno() != 0{
            let err = errno_msg();
            panic!("Failed to change memory protection mode:'{err}'!");
        }
    }
}
impl<W:WritePremisionMarker,E:ExecPremisionMarker> std::ops::Index<usize> for Page<AllowRead,W,E>
{
    type Output = u8;
    fn index(&self,index:usize)->&u8{
        unsafe{&std::slice::from_raw_parts(self.ptr,self.len)[index]}
    }
}
impl<E:ExecPremisionMarker> std::ops::IndexMut<usize> for Page<AllowRead,AllowWrite,E>
{
    fn index_mut(&mut self,index:usize)->&mut u8{
        unsafe{&mut std::slice::from_raw_parts_mut(self.ptr,self.len)[index]}
    }
}
impl<R:ReadPremisionMarker,W:WritePremisionMarker> Page<R,W,AllowExec>{
    fn get_fn_ptr(&self,offset:usize)->*mut u8{
        unsafe{&mut std::slice::from_raw_parts_mut(self.ptr,self.len)[offset] as *mut u8}
    }
}
impl<R:ReadPremisionMarker,W:WritePremisionMarker> Page<R,W,AllowExec>{
    pub fn deny_exec(self)->Page<R,W,DenyExec>{
        let mut res = Page{ptr:self.ptr,len:self.len,read:PhantomData,write:PhantomData,exec:PhantomData};
        std::mem::forget(self);
        res.set_prot();
        res
    }
}
impl<R:ReadPremisionMarker,W:WritePremisionMarker> Page<R,W,DenyExec>{
    pub fn allow_exec(self)->Page<R,W,AllowExec>{
        let mut res = Page{ptr:self.ptr,len:self.len,read:PhantomData,write:PhantomData,exec:PhantomData};
        std::mem::forget(self);
        res.set_prot();
        res
    }
}
impl<R:ReadPremisionMarker,E:ExecPremisionMarker> Page<R,DenyWrite,E>{
    pub fn allow_write(self)->Page<R,AllowWrite,E>{
        let mut res = Page{ptr:self.ptr,len:self.len,read:PhantomData,write:PhantomData,exec:PhantomData};
        std::mem::forget(self);
        res.set_prot();
        res
    }
}
impl<R:ReadPremisionMarker,E:ExecPremisionMarker> Page<R,AllowWrite,E>{
    pub fn deny_write(self)->Page<R,DenyWrite,E>{
        let mut res = Page{ptr:self.ptr,len:self.len,read:PhantomData,write:PhantomData,exec:PhantomData};
        std::mem::forget(self);
        res.set_prot();
        res
    }
}
impl<W:WritePremisionMarker,E:ExecPremisionMarker> Page<DenyRead,W,E>{
    pub fn allow_read(self)->Page<AllowRead,W,E>{
        let mut res = Page{ptr:self.ptr,len:self.len,read:PhantomData,write:PhantomData,exec:PhantomData};
        std::mem::forget(self);
        res.set_prot();
        res
    }
}
impl<W:WritePremisionMarker,E:ExecPremisionMarker> Page<AllowRead,W,E>{
    pub fn deny_read(self)->Page<DenyRead,W,E>{
        let mut res = Page{ptr:self.ptr,len:self.len,read:PhantomData,write:PhantomData,exec:PhantomData};
        std::mem::forget(self);
        res.set_prot();
        res
    }
}
impl<R:ReadPremisionMarker,W:WritePremisionMarker,E:ExecPremisionMarker> Drop for Page<R,W,E>{
    fn drop(&mut self){
        #[cfg(target_family = "unix")]
        unsafe{
            let res = munmap(self.ptr as *mut c_void,self.len);
            if res == -1{
                let err = errno_msg();
                panic!("Unampping memory pages failed. Reason:{err}");
            }
        }
    }
}
#[test]
fn test_alloc_rwe(){
    let page:Page<AllowRead,AllowWrite,AllowExec> = Page::new(256);
}
#[test]
fn test_alloc_rw(){
    let page:Page<AllowRead,AllowWrite,DenyExec> = Page::new(256);
}
#[test]
fn test_alloc_r(){
    let page:Page<AllowRead,DenyWrite,DenyExec> = Page::new(256);
}
#[test]
fn test_alloc_e(){
    let page:Page<DenyRead,DenyWrite,AllowExec> = Page::new(256);
}
#[test]
fn test_alloc_re(){
    let page:Page<AllowRead,DenyWrite,AllowExec> = Page::new(256);
}
#[test]
fn test_acces_rw(){
    let mut page:Page<AllowRead,AllowWrite,DenyExec> = Page::new(256);
    for i in 0..256{
        page[i] = i as u8;
    }
    for i in 0..256{
        assert_eq!(page[i],i as u8);
    }
}
#[test]
fn test_acces_r(){
    let mut page:Page<AllowRead,DenyWrite,DenyExec> = Page::new(256);
    for i in 0..256{
        assert_eq!(page[i],0);
    }
}
#[test]#[cfg(target_arch="x86_64")]
fn test_exec(){
    let mut page:Page<AllowRead,AllowWrite,AllowExec> = Page::new(256);
    //NOP
    page[0] = 0xC3;
    //Add 2 u64s
    page[1] = 0x48;
    page[2] = 0x8d;
    page[3] = 0x04;
    page[4] = 0x37;
    page[5] = 0xC3;
    let nop:fn() = unsafe{std::mem::transmute(page.get_fn_ptr(0))};
    nop();
    let add:fn(u64,u64)->u64 = unsafe{std::mem::transmute(page.get_fn_ptr(1))};
    for i in 0..256{
        for j in 0..256{
            assert_eq!(i+j,add(i,j));
        }
    }
}
#[test]#[cfg(target_arch="x86_64")]
fn test_allow_exec(){
    let mut page:Page<AllowRead,AllowWrite,DenyExec> = Page::new(256);
    //NOP
    page[0] = 0xC3;
    //Add 2 u64s
    page[1] = 0x48;
    page[2] = 0x8d;
    page[3] = 0x04;
    page[4] = 0x37;
    page[5] = 0xC3;
    let page = page.allow_exec();
    let nop:fn() = unsafe{std::mem::transmute(page.get_fn_ptr(0))};
    nop();
    let add:fn(u64,u64)->u64 = unsafe{std::mem::transmute(page.get_fn_ptr(1))};
    for i in 0..256{
        for j in 0..256{
            assert_eq!(i+j,add(i,j));
        }
    }
}
