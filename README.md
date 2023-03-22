# Pages: High level API for low level memory management
While using low-level memory management can provide substantial benefits to projects, it is very often cumbersome. The APIs differ between OS-s and have many pitfalls one can fall into. But what if all of the unsafety of those APIs, all of the differences could be abstracted away at little to no cost? What if you could do fine-grain memory management without ever seeing a pointer? This crate provides such an API.
## Safety by strict typing
One of the common pitfalls of using low-level APIs is the ease of making mistakes regarding page permissions. Mixing them up can at best lead to a segfault, and at worst introduce serious security vulnerabilities. This crate leverages rusts type system(zero sized marker types) to make certain kinds of errors simply impossible. Acquiring a mutable  reference to data that was marked as read-only is not going to be a sudden surprise at runtime. A collection of memory pages, called, unsurprisingly `Pages`, must have a `AllowWrite` marker type. This turns all sorts of runtime errors into compile-time ones, making it impossible to miss them.
## Holds your hand, but does not hold you back. 
This crate provides safe APIs for almost every functionality it has, besides parts of the API that are inherently unsafe.
99% of things you can be using this crate for can be done conveniently without ever uttering the forbidden word `unsafe`.

But what if you are one of the 1% of people who really need those `unsafe` features, or do you just like shooting yourself in the foot?

This crate is going to say: Here's the gun. It has safety measures that prevent accidental misuse, but you do you.

Do you feel really dicey and want some of those juicy potential security vulnerabilities? Just toggle a feature and you are good to go.
## Fine grain adjustments
Are you one of those poor souls that work with enormous data sets that also need processing as fast as possible? Does every nanosecond count? 

Oh boy, have I got a crate for you. 

Pages provides access to **extermaly** fine grain adjustments that can save time. Is the meory use going to be sequential or random?

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


