
use lualite::parser;
use lualite::compiler;
use lualite::runtime::{VirtualMachine, Value};

const SOURCE_CODE: &str = r#"

function binary_search(array, length, needle)
  first = 0
  last = length - 1
  while first <= last do
    mid = (first + last) / 2
    if needle < array[mid] then
      last = mid - 1
    elseif needle > array[mid] then
      first = mid + 1
    else
      return mid
    end
  end
  return false
end

"#;


fn main() {
  let (_, declarations) = parser::parse_file(SOURCE_CODE).unwrap();
  let functions = compiler::compile_declarations(declarations.iter());
  let mut vm = VirtualMachine::with_functions(functions);

  let rust_array = [1, 3, 4, 6, 8, 9, 10, 11, 14, 15];
  let length = Value::try_from(rust_array.len()).expect("value conversion error");
  let array = Value::from_iter(rust_array);
  println!("array: {array}\n");
  for needle in 0..17 {
    let needle_value = Value::from(needle);
    let result = vm.run("binary_search", [array.clone(), length.clone(), needle_value]);
    match result {
      Ok(inner) => println!("search for {needle}, index is {inner}"),
      Err(error) => println!("error: {error:?}"),
    }
  }
}

