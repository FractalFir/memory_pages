# memory_pages: High level API for low level memory management
While using low-level memory management in a project can provide substantial benefits, it is very often cumbersome. The API-s differ significantly between OS-s and have many pitfalls one can easily fall into. But what if, all the unsafety, all the platform-specific differences could be simply abstracted away? What if you could do very fine-grain memory adjustments without ever seeing a pointer? This crate provides such zero-cost abstractions.
## For who it is for?
This crate is mostly meant for performance critical projects, especially ones dealing with huge amounts of data. The APIs and types provided by this crate are not universal solutions to all problems. Since this crate is designed with those applications in mind, it can achieve great improvements(2x faster allocations) over using standard memory management. Another example of strengths of this crate is `PagedVec`-s `clear_decommit` function that not only clears the vector - it also informs the OS about the vec being cleared. This information allows the physical memory pages storing the data to be decommited, making the `PagedVec` occupy no space in the RAM until it is written into, while still having the required space reserved.
# What does `memory_pages` provide?
## Safety by strict typing
One of the common pitfalls of using low-level APIs is the ease of making mistakes regarding page permissions. Mixing them up can at best lead to a segfault, and at worst introduce serious security vulnerabilities. This crate leverages rusts type system(zero sized marker types) to make certain kinds of errors simply impossible. Acquiring a mutable  reference to data that was marked as read-only is not going to lead to a *rapid unplanned program finish* at runtime. A collection of memory pages, called, unsurprisingly, `Pages`, must have a `AllowWrite` marker type, in order to be written into. This turns all sorts of horrible runtime errors into compile-time ones, making it impossible to miss them.
## Holds your hand, but does not hold you back. 
This crates core philosophy is to always guide, never restrict. Almost everything that can be done with this crate can be done without ever seeing the word `unsafe`. While references and the special `FnRef` type are automatically invalidated on permission changes, those safety restrictions can be easily subverted by using unsafe functions and pointers. 
There are some **very** unsafe APIs, which are locked behind feature gates.
## Provide useful hints to the kernel
Use functions such as `adivse_use_soon`, `advise_use_seq`, `adivse_use_rng` to provide memory usage hints.
## With great power comes great responsibility
Just as this crate can greatly improve performance and reduce memory usage, it can also decrease performance and increase memory usage. While achieving results substantially worse than using default allocators is pretty hard and requires being *very* *very* sloppy, it is harder to squeeze everything out of this crate. Gain from usage depends on the competence of the user. As a good example, in some tests, a 2x allocation time reduction was achieved. But those examples were extensively tweaked, to find out the maximal potential performance gain.
## Kernel memory usage hints
Is the memory use going to be sequential or random? Provide optional hints to the kernel which may improve performance in some cases.
# Examples
## Dealing with pages directly
### Data storage
```rust
use memory_pages::*;
let mut memory = Pages<AllowRead,AllowWrite,DenyExec> = Pages::new(0x40000);
read_data(&mut memory).unwrap();
validate_data(&mut memory);
```
### Prevent writes
```rust
use memory_pages::*;
let mut memory = Pages<AllowRead,AllowWrite,DenyExec> = Pages::new(0x40000);
read_data(&mut memory).unwrap();
let mut memory.deny_write();
// `memory` is now read-only and a write attempt would case a segfault
// Because of that this function is now not avalible, so this would not compile if used
// write_data(&mut memory);
```
### x86_64 function assembled at run-time
This example does not work on windows, due to differences in the calling conventions
```rust
use memory_pages::*;
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
### Create new vec
```rust
let mut vec = PagedVec::new(0x10000);
for _ in 1_000_000{
    vec.push(102.32);
}
```
### Clear and deccomit
```rust
let mut vec = PagedVec::new(0x10000);
for _ in 1_000_000{
    vec.push(102.32);
}
// Clears vec, keeping the capacity but freeing the phisical memory,
// which is automaticaly reclaimed as it is filled back up.
vec.clear_decommit();
```

