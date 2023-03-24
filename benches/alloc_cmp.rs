#[macro_use]
extern crate bencher;
use bencher::Bencher;
use std::alloc::GlobalAlloc;

const SMALL_ALLOC_SIZE: usize = 0x1FFE001;
const BIG_ALLOC_SIZE: usize =   0x3000000;

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
    bench.iter(|| {
        let _page: Pages<AllowRead, AllowWrite, DenyExec> = Pages::new(BIG_ALLOC_SIZE);
    })
}
fn small_system_alloc(bench: &mut Bencher) {
    let layout = std::alloc::Layout::from_size_align(SMALL_ALLOC_SIZE, 1).unwrap();
    bench.iter(|| {
        let ptr = unsafe { std::alloc::System.alloc(layout) };
        assert_ne!(ptr as usize, 0);
        unsafe { std::alloc::System.dealloc(ptr, layout) }
    })
}
fn small_page_alloc(bench: &mut Bencher) {
    use pages::*;
    bench.iter(|| {
        let _page: Pages<AllowRead, AllowWrite, DenyExec> = Pages::new(BIG_ALLOC_SIZE);
    })
}
fn push_10_000_000_f64_pv(bench: &mut Bencher) {
    use pages::*;
    let mut vec = PagedVec::new(1_000_000);
    bench.iter(|| {
        for _ in 0..1_000 {
            vec.push(vec.len() as f32);
        }
        if vec.len() == 10_000_000 {
            vec = PagedVec::new(1_000_000);
            // This hint slows the test down. It demonstrates that not all hints are worth the cost.
            // vec.advise_use_seq();
        }
        bencher::black_box(&mut vec);
    });
    bencher::black_box(vec);
}
fn push_10_000_000_f64_v(bench: &mut Bencher) {
    use pages::*;
    let mut vec = Vec::with_capacity(1_000_000);
    bench.iter(|| {
        for _ in 0..1_000 {
            vec.push(vec.len() as f64);
        }
        if vec.len() == 10_000_000 {
            vec = Vec::with_capacity(1_000_000)
        }
        bencher::black_box(&mut vec);
    });
    bencher::black_box(vec);
}
struct TestArray([f64; 4]);
impl TestArray {
    fn new(src: f64) -> Self {
        Self([src, 0.0, src, 23.444])
    }
}
const TEST_SIZE: usize = 10_000_000;
fn push_test_arr_pv(bench: &mut Bencher) {
    use pages::*;
    bench.bench_n((TEST_SIZE / 1_000) as u64, body);
    fn body(bench: &mut Bencher) {
        let mut vec = PagedVec::new(1_000_000);
        //vec.advise_use_soon(TEST_SIZE);
        bench.iter(|| {
            for _ in 0..1_000 {
                vec.push(TestArray::new(vec.len() as f64));
            }
            if vec.len() >= TEST_SIZE {
                vec = PagedVec::new(1_000_000);
                //vec.advise_use_soon(10_000_000);
                // vec.advise_use_seq();
            }
            bencher::black_box(&mut vec);
        });
        bencher::black_box(vec);
    }
}
fn push_test_arr_v(bench: &mut Bencher) {
    use pages::*;
    bench.bench_n((TEST_SIZE / 1_000) as u64, body);
    fn body(bench: &mut Bencher) {
        let mut vec = Vec::with_capacity(1_000_000);
        bench.iter(|| {
            for _ in 0..1_000 {
                vec.push(TestArray::new(vec.len() as f64));
            }
            if vec.len() == TEST_SIZE {
                vec = Vec::with_capacity(1_000_000);
            }
            bencher::black_box(&mut vec);
        });
        bencher::black_box(&mut vec);
    };
}
fn random_rw_pv(bench: &mut Bencher){
    use pages::*;
    //vec.advise_use_rnd();
    fn prep()->PagedVec<f64>{
        let mut vec = PagedVec::new(0x1000_000);
        for i in 0..vec.capacity(){
            let val = (i as f64)*3.14159;
            vec.push(val * val*2.1224);
        }
        vec
    }
    let mut vec = prep();
    let mut idx = 0;
    bench.iter(|| {
        for _ in 0..100_000{
            vec[idx^17963] = vec[(idx^28428)];
            idx = (idx+1).min(vec.len());
        }
    });
}
fn random_rw_v(bench: &mut Bencher){
    fn prep()->Vec<f64>{
        let mut vec =  Vec::with_capacity(0x1000_000);
        for i in 0..vec.capacity(){
            let val = (i as f64)*3.14159;
            vec.push(val * val*2.1224);
        }
        vec
    }
    let mut vec = prep();
    let mut idx = 0;
    bench.iter(|| {
        for _ in 0..100_000{
            vec[idx^17963] = vec[(idx^28428)];
            idx = (idx+1).min(vec.len());
        }
    });
}
benchmark_group!(
    benches,
    system_alloc,
    page_alloc,
    small_system_alloc,
    small_page_alloc,
    push_10_000_000_f64_pv,
    push_10_000_000_f64_v,
    push_test_arr_pv,
    push_test_arr_v,
    random_rw_pv,
    random_rw_v,
);
benchmark_main!(benches);
