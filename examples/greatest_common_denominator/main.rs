
use lualite::parser;
use lualite::compiler;
use lualite::runtime::{VirtualMachine, Value};

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

fn main() {
  let (_, fn_decl) = parser::declaration::function_decl(SOURCE_CODE).unwrap();
  let procedure = compiler::compile_function(&fn_decl);
  let name = fn_decl.name.0;
  println!("{SOURCE_CODE}");
  println!("{name:?}:\n{procedure}");
  let mut vm = VirtualMachine::with_functions([
    (name, procedure),
  ]);
  let a: Value = 250.into();
  let b: Value = 135.into();
  println!("running gcd with {a} and {b}");
  let result = vm.run("gcd", [a, b]);
  match result {
    Ok(inner) => println!("result: {inner}"),
    Err(error) => println!("error: {error:?}"),
  }
}

