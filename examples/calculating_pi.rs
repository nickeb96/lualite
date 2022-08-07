
use lualite::parser;
use lualite::compiler;
use lualite::runtime::{VirtualMachine, Value};

const SOURCE_CODE: &str = r#"

# pi = 3 + 4/(2*3*4) - 4/(4*5*6) + 4/(6*7*8) - 4/(8*9*10) +- ...etc.
function nilakantha_series_sum(n)
  sum = 3.0
  x = 3.0
  add = true
  while n >= 0 do
    temp = (4.0 / ((x - 1.0) * x * (x + 1.0)))
    if add then
      sum = sum + temp
      add = false
    else
      sum = sum - temp
      add = true
    end
    # TODO: use 'add = not add' instead of above once 'not' is implemented
    x = x + 2.0
    n = n - 1
  end
  return sum
end

function calculate_pi()
  return nilakantha_series_sum(100)
end

"#;


fn main() {
  let (_, declarations) = parser::parse_file(SOURCE_CODE).unwrap();
  let functions = compiler::compile_declarations(declarations.iter());
  let mut vm = VirtualMachine::with_functions(functions);

  let result = vm.run("calculate_pi", []);
  match result {
    Ok(Value::Float(pi)) => {
      println!("result: {pi}");
      let difference = f64::abs(pi - std::f64::consts::PI);
      println!("difference of {difference} to true pi ");
    }
    other => println!("unexpected {other:?}"),
  }
}

