
use lualite::parser;
use lualite::compiler;
use lualite::runtime::{VirtualMachine, Value};

const SOURCE_CODE: &str = r#"

function binary_search_helper(array, first, last, needle)
  if first <= last then
    mid = (first + last) / 2
    mid_value = array[mid]
    if needle < mid_value then
      return binary_search_helper(array, first, mid - 1, needle)
    elseif needle > mid_value then
      return binary_search_helper(array, mid + 1, last, needle)
    else
      return mid
    end
  else
    return false
  end
end

function binary_search(array, length, needle)
  binary_search_helper(array, 0, length - 1, needle)
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

