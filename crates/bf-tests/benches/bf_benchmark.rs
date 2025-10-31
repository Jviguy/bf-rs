use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use bf_macros::bf;

fn bench_hello_world(c: &mut Criterion) {
    c.bench_function("bf transpiled hello world", |b| {
        // b.iter(...) runs the code inside many times to get
        // a stable average.
        b.iter(|| {
            // Call your macro here.
            // black_box() is important: it tells the compiler
            // "don't optimize this value away".
            let result = bf!(<u8, 30000, 0>
                ++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.
                >---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
            );

            // Pass the result to black_box to ensure the
            // entire operation is benchmarked.
            black_box(result);
        })
    });
}

/// This ones bad it runs around 5.3b steps.
fn bench_counter_killer(c: &mut Criterion) {
    c.bench_function("bf transpiled counter killer", |b| {
        b.iter(|| {
            let result = bf!(<u8, 62, 0>
                +>>+++++++++++++++++++++++++++++<<[>>>[>>]<[[>>+<<-]>>-[<<]]>+<<[-<<]<]>
                +>>[-<<]<+++++++++[>++++++++>+<<-]>-.----.>.
            );
            black_box(result);
        })
    });
}

criterion_group!(benches, bench_hello_world, bench_counter_killer);
criterion_main!(benches);