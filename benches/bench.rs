use anyhow::Result;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use desmos_compiler::{
    expressions::{ExpressionId, Expressions},
    lang::codegen::{
        ir::{IRType, SegmentKey},
        jit::{
            function::{ExplicitFn, ExplicitJitFn, PointValue},
            ExecutionEngine,
        },
        IRGen,
    },
};

use cranelift_backend::CraneliftBackend;

criterion_group!(benches, criterion_main_entry);
criterion_main!(benches);

fn bench_parsing(c: &mut Criterion) {
    let cases = vec![
        ("simple_addition", "2 + 2"),
        ("variable_multiply", "x * 3"),
        ("list_addition", "[x, x + 1, x + 2]"),
        ("nested_ops", "[(x+2)^{2}, (x-1)^{2}]"),
        ("power_list", "[2, 3, 4]^{3}"),
        (
            "for_loop_cartesian",
            "(a, b) \\operatorname{for} a = [1, 2], b = [10, 20]",
        ),
    ];

    for (label, expr) in cases {
        c.bench_function(&format!("parse/{}", label), |b| {
            b.iter(|| {
                let mut expressions = Expressions::new();
                let id = ExpressionId(0);
                expressions.insert_expr(id, black_box(expr)).unwrap();
                let (_ir, errors) = IRGen::generate_ir(&expressions);
                assert!(errors.is_empty());
            });
        });
    }
}

fn bench_evaluation(c: &mut Criterion) -> Result<()> {
    let cases = vec![
        ("variable_multiply", "x * 3", vec![10.0]),
        ("list_addition", "[x, x + 1, x + 2]", vec![5.0]),
        ("nested_ops", "[(x+2)^{2}, (x-1)^{2}]", vec![2.0]),
        ("scalar_add_list", "10 + [1, 2, 3]", vec![]),
        (
            "for_loop_cartesian",
            "(a, b) \\operatorname{for} a = [1, 2], b = [10, 20]",
            vec![],
        ),
    ];

    for (label, expr, inputs) in cases {
        let mut expressions = Expressions::new();
        expressions.insert_expr(ExpressionId(0), expr)?;
        let (ir, errors) = IRGen::generate_ir(&expressions);
        assert!(errors.is_empty());

        let mut backend = CraneliftBackend::new()?;
        backend.compile_module(&ir)?;

        let args = vec![IRType::NUMBER; 1];
        let key = SegmentKey::new("explicit_0".to_string(), args);
        let ty = ir.get_segment(&key).and_then(|s| s.ret()).unwrap().ty();
        let x = *inputs.last().unwrap_or(&0.0);
        let func = backend.get_explicit_fn(&key.to_string(), &ty).unwrap();
        c.bench_function(&format!("eval/{}", label), |b| {
            b.iter(|| {
                match &func {
                    ExplicitJitFn::Number(f) => {
                        black_box(f.call(x));
                    }
                    ExplicitJitFn::Point(f) => {
                        black_box(f.call(x));
                    }
                    ExplicitJitFn::NumberList(f) => {
                        black_box::<Vec<f64>>(f.call(x));
                    }
                    ExplicitJitFn::PointList(f) => {
                        black_box::<Vec<PointValue>>(f.call(x));
                    }
                };
            });
        });
    }

    Ok(())
}

fn criterion_main_entry(c: &mut Criterion) {
    eprintln!("hi");
    bench_parsing(c);
    bench_evaluation(c).unwrap();
}
