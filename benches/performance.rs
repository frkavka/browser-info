use browser_info::{ExtractionMethod, get_browser_info_safe, get_browser_info_with_method};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use tokio::runtime::Runtime;

fn bench_extraction_methods(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("browser_info_extraction");

    // PowerShell方式
    group.bench_function("powershell_method", |b| {
        b.iter(|| {
            // ブラウザが開いてない場合のエラーは無視
            let _ = black_box(get_browser_info_safe());
        })
    });

    // DevTools方式
    #[cfg(feature = "devtools")]
    group.bench_function("devtools_method", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _ = black_box(get_browser_info_with_method(ExtractionMethod::DevTools).await);
            })
        })
    });

    // 自動選択
    group.bench_function("auto_method", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _ = black_box(get_browser_info_with_method(ExtractionMethod::Auto).await);
            })
        })
    });

    group.finish();
}

fn bench_browser_detection(c: &mut Criterion) {
    use browser_info::is_browser_active;

    c.bench_function("browser_detection", |b| {
        b.iter(|| black_box(is_browser_active()))
    });
}

criterion_group!(benches, bench_extraction_methods, bench_browser_detection);
criterion_main!(benches);
