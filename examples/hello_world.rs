
use lualite::parser;
use lualite::compiler;
use lualite::runtime::VirtualMachine;

const SOURCE_CODE: &str = r#"
function main()
  return "hello world"
end
"#;

fn main() {
  let (_, main_ast) = parser::declaration::function_decl(SOURCE_CODE).unwrap();
  let main_procedure = compiler::compile_function(&main_ast);
  let mut vm = VirtualMachine::with_functions([
    ("main".to_owned(), main_procedure),
  ]);
  let result = vm.run("main", []);
  match result {
    Ok(inner) => println!("result: {inner}"),
    Err(error) => println!("error: {error:?}"),
  }
}
