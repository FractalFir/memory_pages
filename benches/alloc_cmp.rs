#[macro_use]
extern crate bencher;
use bencher::Bencher;
use std::alloc::GlobalAlloc;
const SMALL_ALLOC_SIZE: usize = 0x1FFFFFF;
const BIG_ALLOC_SIZE: usize = 0x2000000;
fn system_alloc(bench: &mut Bencher) {
    let layout = std::alloc::Layout::from_size_align(BIG_ALLOC_SIZE, 1).unwrap();
    bench.iter(|| {
        let ptr = unsafe { std::alloc::System.alloc(layout) };
        assert_ne!(ptr as usize, 0);
        unsafe { std::alloc::System.dealloc(ptr, layout) }
    })
}
fn page_alloc(bench: &mut Bencher) {
    use pages::*;
    let layout = std::alloc::Layout::from_size_align(BIG_ALLOC_SIZE, 1).unwrap();
    bench.iter(|| {
        let page: Page<AllowRead, AllowWrite, AllowExec> = Page::new(BIG_ALLOC_SIZE);
    })
}
benchmark_group!(benches, system_alloc, page_alloc);
benchmark_main!(benches);
