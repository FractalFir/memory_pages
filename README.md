# What is this crate?
`pages` is a small crate providing a cross-platform API to request pages from kernel with certain permission modes 
set(read,write,execute). It provides an very safe API to aid in many use cases, mainly:
1. Speeds up operating on large data sets: `PagedVec` provides speed advantages over standard `Vec` for large data 
types.
2. Simplifies dealing with page permissions and allows for additional levels of safety: Pages with `DennyWrite` cannot be 
written into without their permissions being changed, which allows for certain kinds of bugs to cause segfaults insted of overwriting data. 
3. Simplifies JITs - while dealing with memory pages is simple compared to difficulty of the task, which is writing a 
Just-In-Time compiler, this crate abstracts the platform specific differences away and adds additional measures to prevent 
/some security issues, allowing you to focus on writing the compiler itself, without worrying about those low-level details
# Error-proof API
`pages` API tries its best to actively prevent you from misusing it. It is, for example, impossible to acquire a mutable reference to a memory page which does not allow writes. Built-in marker types ensure those restrictions are represented an enforced within the rust type system, making all of those checks occur at compile time!
# Examples
## Dealing with pages directly
### x86_64 function assembled at run-time
This example does not work on windows, due to differences in the calling conventions
```rust
use pages::*; 
let mut memory:Pages<AllowRead,AllowWrite,DenyExec> = Pages::new(0x4000);

// hex-encoded X86_64 assembly for adding 2 numbers
// lea     rax, [rdi+rsi]
memory[0] = 0x48;
memory[1] = 0x8d;
memory[2] = 0x04;
memory[3] = 0x37;
// ret
memory[4] = 0xC3;
// Sets execution to allow and write to denny to prevent exploits
let memory = memory.set_protected_exec();
//TODO: this should check for lifetimes!
let add:extern "C" fn(u64,u64)->u64 = unsafe{memory.get_fn(0)};
assert_eq!(add(43,34),77);
```
## PagedVec


