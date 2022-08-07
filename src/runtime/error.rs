
#[derive(Debug)]
pub enum RuntimeError {
  InvalidRegister,
  InvalidPc,
  EmptyCallStack,
  MissingFunction,
  MissingConstant,
}
