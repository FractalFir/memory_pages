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
## The biggest benefit and curse is...
Fine grain adjustments. If used without any care in the world, this crate has very low to none real performance gain. But, if used propely, the gains can be very noticeable. Speed of allocating a very large(>35MB) data structure can be cut down to half the time. A great example of those gains is the `PagedVec`. A drop-in replacement for `Vec` meant for storing large amount of data. In the test case, both `PagedVec` and `Vec` were provided a very rough estimate for required capacity(1/10 of the final push count). This hint was meant to give both types even playing field, while presenting a real world use case for `PagedVec`. In tests, `PagedVec` was around 40% faster than regular `Vec`. While real-world performance gains will be usually smaller, this shows the  potential coming from more involved memory management.
## Kernel memory usage hints
Is the memory use going to be sequential or random? Provide optional hints to the kernel which may improve performance in some cases.
# Examples
## Dealing with pages directly
### x86_64 function assembled at run-time
This example does not work on windows, due to differences in the calling conventions
```rust
use pages::*;
let mut memory:Pages<AllowRead,AllowWrite,DenyExec> = Pages::new(0x4000);

// hex-encoded X86_64 assembly for adding 2 numbers
// lea 	rax, [rdi+rsi]
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

# Benchmark setup
Benchmark setup has been altered to run benchmarks for much longer times(10x) in order to reduce noise.

