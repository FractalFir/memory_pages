use crate::*;
/// A reference to a function inside [`Pages`]. It enforces that it may never outlive the [`Pages`] it is contained in,
/// preventing lifetime related errors. Additionally, it enforces that if [`Pages`] permissions are changes, all [`FnRef`]
/// referencing it will be invalidated, preventing exploits related to page permissions.
pub struct FnRef<'a, F: ExternFnPtr> {
    fnc: F,
    pd: PhantomData<&'a ()>,
}
impl<'a, F: ExternFnPtr> FnRef<'a, F> {
    pub(crate) fn new<R: ReadPremisionMarker, W: WritePremisionMarker>(
        fnc: F,
        _page: &'a Pages<R, W, AllowExec>,
    ) -> Self {
        Self {
            fnc,
            pd: PhantomData,
        }
    }
}
impl<'a, F: ExternFnPtr + Copy> FnRef<'a, F> {
    /// Returns the internal function.
    /// # Safety
    /// It is up to the user to ensure [`Pages`] returned function resides in lives long enough.
    /// # Examples
    /// ```
    /// # use memory_pages::*;
    /// let mut memory:Pages<AllowRead,AllowWrite,DenyExec> = Pages::new(0x4000);
    /// // X86_64 assembly instruction `RET`
    /// memory[0] = 0xC3;
    /// let memory = memory.set_protected_exec();
    /// let nop:unsafe extern "C" fn() = unsafe{memory.get_fn(0).internal_fn()};
    /// // Since nothing is known about functions inside this page during
    /// // the compilation process, calling a function from a page is inherently unsafe.
    /// unsafe{nop()};
    /// ```
    pub unsafe fn internal_fn(&self) -> F {
        self.fnc
    }
}
/// Trait representing an unsafe function that may be called.
pub trait UnsafeCallable<Args> {
    /// Return type of represented function
    type Ret;
    /// Calls the underlying function.
    unsafe fn call(&self, args: Args) -> Self::Ret;
}
impl<'a, Ret> UnsafeCallable<()> for FnRef<'a, unsafe extern "C" fn() -> Ret> {
    type Ret = Ret;
    unsafe fn call(&self, _args: ()) -> Ret {
        (self.fnc)()
    }
}
impl<'a, Ret, Arg1> UnsafeCallable<Arg1> for FnRef<'a, unsafe extern "C" fn(Arg1) -> Ret> {
    type Ret = Ret;
    unsafe fn call(&self, args: Arg1) -> Ret {
        (self.fnc)(args)
    }
}
impl<'a, Ret, Arg1, Arg2> UnsafeCallable<(Arg1, Arg2)>
    for FnRef<'a, unsafe extern "C" fn(Arg1, Arg2) -> Ret>
{
    type Ret = Ret;
    unsafe fn call(&self, args: (Arg1, Arg2)) -> Ret {
        (self.fnc)(args.0, args.1)
    }
}
impl<'a, Ret, Arg1, Arg2, Arg3> UnsafeCallable<(Arg1, Arg2, Arg3)>
    for FnRef<'a, unsafe extern "C" fn(Arg1, Arg2, Arg3) -> Ret>
{
    type Ret = Ret;
    unsafe fn call(&self, args: (Arg1, Arg2, Arg3)) -> Ret {
        (self.fnc)(args.0, args.1, args.2)
    }
}
impl<'a, Ret, Arg1, Arg2, Arg3, Arg4> UnsafeCallable<(Arg1, Arg2, Arg3, Arg4)>
    for FnRef<'a, unsafe extern "C" fn(Arg1, Arg2, Arg3, Arg4) -> Ret>
{
    type Ret = Ret;
    unsafe fn call(&self, args: (Arg1, Arg2, Arg3, Arg4)) -> Ret {
        (self.fnc)(args.0, args.1, args.2, args.3)
    }
}
impl<'a, Ret, Arg1, Arg2, Arg3, Arg4, Arg5> UnsafeCallable<(Arg1, Arg2, Arg3, Arg4, Arg5)>
    for FnRef<'a, unsafe extern "C" fn(Arg1, Arg2, Arg3, Arg4, Arg5) -> Ret>
{
    type Ret = Ret;
    unsafe fn call(&self, args: (Arg1, Arg2, Arg3, Arg4, Arg5)) -> Ret {
        (self.fnc)(args.0, args.1, args.2, args.3, args.4)
    }
}
impl<'a, Ret, Arg1, Arg2, Arg3, Arg4, Arg5, Arg6>
    UnsafeCallable<(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6)>
    for FnRef<'a, unsafe extern "C" fn(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6) -> Ret>
{
    type Ret = Ret;
    unsafe fn call(&self, args: (Arg1, Arg2, Arg3, Arg4, Arg5, Arg6)) -> Ret {
        (self.fnc)(args.0, args.1, args.2, args.3, args.4, args.5)
    }
}
impl<'a, Ret, Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7>
    UnsafeCallable<(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7)>
    for FnRef<'a, unsafe extern "C" fn(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7) -> Ret>
{
    type Ret = Ret;
    unsafe fn call(&self, args: (Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7)) -> Ret {
        (self.fnc)(args.0, args.1, args.2, args.3, args.4, args.5, args.6)
    }
}
impl<'a, Ret, Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8>
    UnsafeCallable<(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8)>
    for FnRef<'a, unsafe extern "C" fn(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8) -> Ret>
{
    type Ret = Ret;
    unsafe fn call(&self, args: (Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8)) -> Ret {
        (self.fnc)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7,
        )
    }
}
impl<'a, Ret, Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9>
    UnsafeCallable<(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9)>
    for FnRef<'a, unsafe extern "C" fn(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9) -> Ret>
{
    type Ret = Ret;
    unsafe fn call(&self, args: (Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9)) -> Ret {
        (self.fnc)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8,
        )
    }
}
impl<'a, Ret, Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10>
    UnsafeCallable<(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10)>
    for FnRef<
        'a,
        unsafe extern "C" fn(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10) -> Ret,
    >
{
    type Ret = Ret;
    unsafe fn call(
        &self,
        args: (Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10),
    ) -> Ret {
        (self.fnc)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9,
        )
    }
}
impl<'a, Ret, Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10, Arg11>
    UnsafeCallable<(
        Arg1,
        Arg2,
        Arg3,
        Arg4,
        Arg5,
        Arg6,
        Arg7,
        Arg8,
        Arg9,
        Arg10,
        Arg11,
    )>
    for FnRef<
        'a,
        unsafe extern "C" fn(
            Arg1,
            Arg2,
            Arg3,
            Arg4,
            Arg5,
            Arg6,
            Arg7,
            Arg8,
            Arg9,
            Arg10,
            Arg11,
        ) -> Ret,
    >
{
    type Ret = Ret;
    unsafe fn call(
        &self,
        args: (
            Arg1,
            Arg2,
            Arg3,
            Arg4,
            Arg5,
            Arg6,
            Arg7,
            Arg8,
            Arg9,
            Arg10,
            Arg11,
        ),
    ) -> Ret {
        (self.fnc)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9, args.10,
        )
    }
}
impl<'a, Ret, Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10, Arg11, Arg12>
    UnsafeCallable<(
        Arg1,
        Arg2,
        Arg3,
        Arg4,
        Arg5,
        Arg6,
        Arg7,
        Arg8,
        Arg9,
        Arg10,
        Arg11,
        Arg12,
    )>
    for FnRef<
        'a,
        unsafe extern "C" fn(
            Arg1,
            Arg2,
            Arg3,
            Arg4,
            Arg5,
            Arg6,
            Arg7,
            Arg8,
            Arg9,
            Arg10,
            Arg11,
            Arg12,
        ) -> Ret,
    >
{
    type Ret = Ret;
    unsafe fn call(
        &self,
        args: (
            Arg1,
            Arg2,
            Arg3,
            Arg4,
            Arg5,
            Arg6,
            Arg7,
            Arg8,
            Arg9,
            Arg10,
            Arg11,
            Arg12,
        ),
    ) -> Ret {
        (self.fnc)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9,
            args.10, args.11,
        )
    }
}
impl<'a, Ret, Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10, Arg11, Arg12, Arg13>
    UnsafeCallable<(
        Arg1,
        Arg2,
        Arg3,
        Arg4,
        Arg5,
        Arg6,
        Arg7,
        Arg8,
        Arg9,
        Arg10,
        Arg11,
        Arg12,
        Arg13,
    )>
    for FnRef<
        'a,
        unsafe extern "C" fn(
            Arg1,
            Arg2,
            Arg3,
            Arg4,
            Arg5,
            Arg6,
            Arg7,
            Arg8,
            Arg9,
            Arg10,
            Arg11,
            Arg12,
            Arg13,
        ) -> Ret,
    >
{
    type Ret = Ret;
    unsafe fn call(
        &self,
        args: (
            Arg1,
            Arg2,
            Arg3,
            Arg4,
            Arg5,
            Arg6,
            Arg7,
            Arg8,
            Arg9,
            Arg10,
            Arg11,
            Arg12,
            Arg13,
        ),
    ) -> Ret {
        (self.fnc)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9,
            args.10, args.11, args.12,
        )
    }
}
impl<
        'a,
        Ret,
        Arg1,
        Arg2,
        Arg3,
        Arg4,
        Arg5,
        Arg6,
        Arg7,
        Arg8,
        Arg9,
        Arg10,
        Arg11,
        Arg12,
        Arg13,
        Arg14,
    >
    UnsafeCallable<(
        Arg1,
        Arg2,
        Arg3,
        Arg4,
        Arg5,
        Arg6,
        Arg7,
        Arg8,
        Arg9,
        Arg10,
        Arg11,
        Arg12,
        Arg13,
        Arg14,
    )>
    for FnRef<
        'a,
        unsafe extern "C" fn(
            Arg1,
            Arg2,
            Arg3,
            Arg4,
            Arg5,
            Arg6,
            Arg7,
            Arg8,
            Arg9,
            Arg10,
            Arg11,
            Arg12,
            Arg13,
            Arg14,
        ) -> Ret,
    >
{
    type Ret = Ret;
    unsafe fn call(
        &self,
        args: (
            Arg1,
            Arg2,
            Arg3,
            Arg4,
            Arg5,
            Arg6,
            Arg7,
            Arg8,
            Arg9,
            Arg10,
            Arg11,
            Arg12,
            Arg13,
            Arg14,
        ),
    ) -> Ret {
        (self.fnc)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9,
            args.10, args.11, args.12, args.13,
        )
    }
}
impl<
        'a,
        Ret,
        Arg1,
        Arg2,
        Arg3,
        Arg4,
        Arg5,
        Arg6,
        Arg7,
        Arg8,
        Arg9,
        Arg10,
        Arg11,
        Arg12,
        Arg13,
        Arg14,
        Arg15,
    >
    UnsafeCallable<(
        Arg1,
        Arg2,
        Arg3,
        Arg4,
        Arg5,
        Arg6,
        Arg7,
        Arg8,
        Arg9,
        Arg10,
        Arg11,
        Arg12,
        Arg13,
        Arg14,
        Arg15,
    )>
    for FnRef<
        'a,
        unsafe extern "C" fn(
            Arg1,
            Arg2,
            Arg3,
            Arg4,
            Arg5,
            Arg6,
            Arg7,
            Arg8,
            Arg9,
            Arg10,
            Arg11,
            Arg12,
            Arg13,
            Arg14,
            Arg15,
        ) -> Ret,
    >
{
    type Ret = Ret;
    unsafe fn call(
        &self,
        args: (
            Arg1,
            Arg2,
            Arg3,
            Arg4,
            Arg5,
            Arg6,
            Arg7,
            Arg8,
            Arg9,
            Arg10,
            Arg11,
            Arg12,
            Arg13,
            Arg14,
            Arg15,
        ),
    ) -> Ret {
        (self.fnc)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9,
            args.10, args.11, args.12, args.13, args.14,
        )
    }
}
impl<
        'a,
        Ret,
        Arg1,
        Arg2,
        Arg3,
        Arg4,
        Arg5,
        Arg6,
        Arg7,
        Arg8,
        Arg9,
        Arg10,
        Arg11,
        Arg12,
        Arg13,
        Arg14,
        Arg15,
        Arg16,
    >
    UnsafeCallable<(
        Arg1,
        Arg2,
        Arg3,
        Arg4,
        Arg5,
        Arg6,
        Arg7,
        Arg8,
        Arg9,
        Arg10,
        Arg11,
        Arg12,
        Arg13,
        Arg14,
        Arg15,
        Arg16,
    )>
    for FnRef<
        'a,
        unsafe extern "C" fn(
            Arg1,
            Arg2,
            Arg3,
            Arg4,
            Arg5,
            Arg6,
            Arg7,
            Arg8,
            Arg9,
            Arg10,
            Arg11,
            Arg12,
            Arg13,
            Arg14,
            Arg15,
            Arg16,
        ) -> Ret,
    >
{
    type Ret = Ret;
    unsafe fn call(
        &self,
        args: (
            Arg1,
            Arg2,
            Arg3,
            Arg4,
            Arg5,
            Arg6,
            Arg7,
            Arg8,
            Arg9,
            Arg10,
            Arg11,
            Arg12,
            Arg13,
            Arg14,
            Arg15,
            Arg16,
        ),
    ) -> Ret {
        (self.fnc)(
            args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9,
            args.10, args.11, args.12, args.13, args.14, args.15,
        )
    }
}

/*
#[cfg(feature = "fn_traits")]
impl<Args,F:ExternFnPtr> std::ops::FnOnce<Args> for &FnRef<'_,F>
    where for<'a> FnRef<'a,F>:UnsafeCallable<Args>, Args: std::marker::Tuple
    {
    type Output = <Self as UnsafeCallable<Args>>::Ret;
    extern "rust-call" fn call_once(&self,args:Args)->Self::Output{
        self.call(args)
    }
}*/
