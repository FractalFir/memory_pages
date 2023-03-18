# What is pages
`pages` is a small library provinig a cross-platform API to request pages from kernel with certain permission modes set(read,write,execute). It simplifies writing JIT compilers, by proving a way to allocate executable memory and change memory protection on almost any system(*Windows* and most POSIX-compliant systems(Linux,Redox,FreeBSD,MacOS,most other BSDs).). But not only JIT compilers may benefit from this crate. While slow for small allocations, it is 2 times faster to allocate and deallocate memory chunks larger than 34MB. 
# Error-proof API
`pages` API tries its best to actively prevent you from misusing it. It is, for example, impossible to acquire a mutable reference to a memory page which does not allow writes. Built-in marker types ensure those restrictions are represented an enforced within the rust type system. 
# Examples
