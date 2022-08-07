
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use lualite::parser;
use lualite::compiler;
use lualite::runtime::{VirtualMachine, Value, RuntimeError};

const SOURCE_CODE: &str = r#"
function gcd(a, b)
  while a != b do
    if a > b then
      a = a - b
    else
      b = b - a
    end
  end
  return a
end
"#;

fn parse_compile_run_gcd(a: i64, b: i64) -> Result<Value, RuntimeError> {
  let (_, fn_decl) = parser::declaration::function_decl(SOURCE_CODE).unwrap();
  let main_procedure = compiler::compile_function(&fn_decl);
  let mut vm = VirtualMachine::with_functions([
    (fn_decl.name.0.clone(), main_procedure),
  ]);
  vm.run("gcd", [a.into(), b.into()])
}

pub fn bench_gcd(c: &mut Criterion) {
  c.bench_function("gcd", |b| b.iter(|| parse_compile_run_gcd(black_box(25000), black_box(135))));
}

criterion_group!(benches, bench_gcd);
criterion_main!(benches);

