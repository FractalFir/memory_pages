pub trait ExternFnPtr {}
impl<Ret> ExternFnPtr for unsafe extern "C" fn() -> Ret {}
impl<Arg1, Ret> ExternFnPtr for unsafe extern "C" fn(Arg1) -> Ret {}

impl<Arg1, Arg2, Ret> ExternFnPtr for unsafe extern "C" fn(Arg1, Arg2) -> Ret {}

impl<Arg1, Arg2, Arg3, Ret> ExternFnPtr for unsafe extern "C" fn(Arg1, Arg2, Arg3) -> Ret {}
impl<Arg1, Arg2, Arg3, Arg4, Ret> ExternFnPtr
    for unsafe extern "C" fn(Arg1, Arg2, Arg3, Arg4) -> Ret
{
}
impl<Arg1, Arg2, Arg3, Arg4, Arg5, Ret> ExternFnPtr
    for unsafe extern "C" fn(Arg1, Arg2, Arg3, Arg4, Arg5) -> Ret
{
}
impl<Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Ret> ExternFnPtr
    for unsafe extern "C" fn(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6) -> Ret
{
}
impl<Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Ret> ExternFnPtr
    for unsafe extern "C" fn(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7) -> Ret
{
}
impl<Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Ret> ExternFnPtr
    for unsafe extern "C" fn(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8) -> Ret
{
}
impl<Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Ret> ExternFnPtr
    for unsafe extern "C" fn(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9) -> Ret
{
}
impl<Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10, Ret> ExternFnPtr
    for unsafe extern "C" fn(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10) -> Ret
{
}
impl<Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10, Arg11, Ret> ExternFnPtr
    for unsafe extern "C" fn(
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
    ) -> Ret
{
}
impl<Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10, Arg11, Arg12, Ret> ExternFnPtr
    for unsafe extern "C" fn(
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
    ) -> Ret
{
}
impl<Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10, Arg11, Arg12, Arg13, Ret>
    ExternFnPtr
    for unsafe extern "C" fn(
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
    ) -> Ret
{
}
impl<
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
        Ret,
    > ExternFnPtr
    for unsafe extern "C" fn(
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
    ) -> Ret
{
}
impl<
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
        Ret,
    > ExternFnPtr
    for unsafe extern "C" fn(
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
    ) -> Ret
{
}
impl<
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
        Ret,
    > ExternFnPtr
    for unsafe extern "C" fn(
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
    ) -> Ret
{
}
