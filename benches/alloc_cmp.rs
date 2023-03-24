use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::alloc::GlobalAlloc;

const SMALL_ALLOC_SIZE: usize = 0x1FFE001;
const BIG_ALLOC_SIZE: usize = 0x3000000;
struct TestType([f64; 4]);
impl TestType {
    fn new(src: f64) -> Self {
        Self([src, 0.0, src, 23.444])
    }
}
const TEST_SIZE: usize = 10_000_000;

fn system_alloc(bench: &mut Criterion) {
    let layout = std::alloc::Layout::from_size_align(BIG_ALLOC_SIZE, 1).unwrap();
    bench.bench_function("system_alloc", |b| {
        b.iter(|| {
            let ptr = unsafe { std::alloc::System.alloc(layout) };
            assert_ne!(ptr as usize, 0);
            unsafe { std::alloc::System.dealloc(ptr, layout) }
        })
    });
}
fn page_alloc(bench: &mut Criterion) {
    use memory_pages::*;
    bench.bench_function("page_alloc", |b| {
        b.iter(|| {
            let _page: Pages<AllowRead, AllowWrite, DenyExec> = Pages::new(BIG_ALLOC_SIZE);
        })
    });
}

fn small_system_alloc(bench: &mut Criterion) {
    let layout = std::alloc::Layout::from_size_align(SMALL_ALLOC_SIZE, 1).unwrap();
    bench.bench_function("small_system_alloc", |b| {
        b.iter(|| {
            let ptr = unsafe { std::alloc::System.alloc(layout) };
            assert_ne!(ptr as usize, 0);
            unsafe { std::alloc::System.dealloc(ptr, layout) }
        })
    });
}
fn small_page_alloc(bench: &mut Criterion) {
    use memory_pages::*;
    bench.bench_function("small_page_alloc", |b| {
        b.iter(|| {
            let _page: Pages<AllowRead, AllowWrite, DenyExec> = Pages::new(BIG_ALLOC_SIZE);
        })
    });
}

fn push_10m_f64_pv(bench: &mut Criterion) {
    use memory_pages::*;
    let mut vec = PagedVec::new(1_000_000);
    bench.bench_function("push_10m_f64_pv", |b| {
        b.iter(|| {
            for _ in 0..1_000 {
                vec.push(vec.len() as f32);
            }
            if vec.len() == 10_000_000 {
                vec = PagedVec::new(1_000_000);
                // This hint slows the test down. It demonstrates that not all hints are worth the cost.
                // vec.advise_use_seq();
            }
            black_box(&mut vec);
        })
    });
    black_box(vec);
}
fn push_10m_f64_v(bench: &mut Criterion) {
    use memory_pages::*;
    let mut vec = Vec::with_capacity(1_000_000);
    bench.bench_function("push_10m_f64_v", |b| {
        b.iter(|| {
            for _ in 0..1_000 {
                vec.push(vec.len() as f64);
            }
            if vec.len() == 10_000_000 {
                vec = Vec::with_capacity(1_000_000)
            }
            black_box(&mut vec);
        })
    });
    black_box(vec);
}

fn push_test_type_pv(bench: &mut Criterion) {
    use memory_pages::*;
    let mut vec = PagedVec::new(1_000_000);
    bench.bench_function("push_test_type_pv", |b| {
        b.iter(|| {
            vec.push(TestType::new(vec.len() as f64));
            if vec.len() == TEST_SIZE {
                vec.clear();
            }
        })
    });
    black_box(&mut vec);
}
fn push_test_type_v(bench: &mut Criterion) {
    use memory_pages::*;
    let mut vec = Vec::with_capacity(1_000_000);
    bench.bench_function("push_test_type_v", |b| {
        b.iter(|| {
            vec.push(TestType::new(vec.len() as f64));
            if vec.len() == TEST_SIZE {
                vec.clear();
            }
        })
    });
    black_box(&mut vec);
}
fn random_rw_pv(bench: &mut Criterion) {
    use memory_pages::*;
    fn prep() -> PagedVec<usize> {
        let mut vec = PagedVec::new(0x1000_000);
        for i in 0..vec.capacity() {
            let val = i;
            vec.push(val);
        }
        vec.advise_use_rnd();
        vec
    }
    let mut vec = prep();
    let mut idx = 0;
    bench.bench_function("random_rw_pv", |b| {
        let mut prev = idx;
        b.iter(|| {
            vec[idx] = vec[prev];
            idx = (idx + 1).min(vec.len() - 1);
        })
    });
}
fn random_rw_v(bench: &mut Criterion) {
    fn prep() -> Vec<usize> {
        let mut vec = Vec::with_capacity(0x1000_000);
        for i in 0..vec.capacity() {
            let val = i;
            vec.push(val);
        }
        vec
    }
    let mut vec = prep();
    let mut idx = 0;
    bench.bench_function("random_rw_v", |b| {
        let mut prev = idx;
        b.iter(|| {
            vec[idx] = vec[prev];
            idx = (idx + 1).min(vec.len() - 1);
        })
    });
}
criterion_group!(
    benches,
    random_rw_pv,
    random_rw_v,
    push_test_type_pv,
    push_test_type_v,
    system_alloc,
    page_alloc,
    small_system_alloc,
    small_page_alloc,
    push_10m_f64_pv,
    push_10m_f64_v,
);
criterion_main!(benches);
