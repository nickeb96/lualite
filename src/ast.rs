//! Abstract Syntax Tree types
//!
//! These types have no inherent methods or functions on them, instead they represent
//! a state in the `parse -> compile -> interpret` pipeline.
//!
//! They are the output of the [parser](../parser/index.html) and the input
//! of the [compiler](../compiler/index.html).  More information on how source code is
//! mapped to AST components can be found in the parser documentation.

/// Top-level declarations in a file
#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
  Function(FunctionDecl),
  Static(StaticDecl),
}

/// Function declaration and definition
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
  /// Name of the function
  pub name: Identifier,
  /// List of parameter names
  pub params: Vec<Identifier>,
  /// List of statements in the function body
  pub body: Vec<Statement>,
}

/// Static variable/constant declaration
#[derive(Debug, Clone, PartialEq)]
pub struct StaticDecl {
  pub name: Identifier,
  pub value: Option<Expression>,
}

/// Statement in a function
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
  /// A single expression as a statement
  /// ## Example:
  /// ```text
  /// f(x + 1, z, true)
  /// ```
  SingleStatement(Expression),
  /// Assignment to an identifier from the result of an expression
  /// ## Example:
  /// ```text
  /// y = (x + 1) * 2
  /// ```
  AssignStatement(Identifier, Expression),
  /// Assignment to an indexed slot in a container
  /// ## Example:
  /// ```text
  /// items[i - 1] = g(x) + 9
  /// ```
  IndexAssignStatement {
    /// Destination container to put `value` into
    table: Expression,
    /// Index or key into `table`
    index: Expression,
    /// Expression to assign from
    value: Expression,
  },
  /// Return statement with an optional expression
  /// ## Example:
  /// ```text
  /// return
  /// ```
  /// or
  /// ```text
  /// return x * 2
  /// ```
  ReturnStatement(Option<Expression>),
  /// While loop
  /// ## Example:
  /// ```text
  /// while a < b do
  ///   b = b - f(a)
  /// end
  /// ```
  WhileStatement {
    /// Conditional expression to determine if iteration should continue
    condition: Expression,
    /// Statement body of the loop
    body: Vec<Statement>,
  },
  /// If (else) statement
  ///
  /// If statements can have zero or more `elseif` clauses after the `if` clause and an
  /// optional else clause at the end.
  /// ## Example:
  /// ```text
  /// if x >= 5 then
  ///   f(x)
  /// end
  /// ```
  /// or
  /// ```text
  /// if x == 9 then
  ///   f(x)
  /// elseif x == 7 then
  ///   g(x)
  /// elseif x <= 0 then
  ///   h(0)
  /// else
  ///   h(x)
  /// end
  /// ```
  IfStatement {
    /// Conditional expression to determine if the `if` clause should run
    condition: Expression,
    /// Statement body of the `if` clause
    body: Vec<Statement>,
    /// Optional statement body of the `else` clause
    else_body: Option<Vec<Statement>>,
  },
}

/// Expression in a statement
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
  /// Identifier as an expression
  Identifier(Identifier),
  /// Integer literal
  Integer(IntegerLiteral),
  /// Float literal
  Float(FloatLiteral),
  /// Boolean literal
  Boolean(BooleanLiteral),
  /// String literal
  String(StringLiteral),
  /// Unary prefix operator expressions
  Unary {
    /// Unary operator
    op: UnaryOperator,
    /// Child expression
    right: Box<Expression>,
  },
  /// Binary operator expressions
  Binary {
    /// Left child expression
    left: Box<Expression>,
    /// Binary operator
    op: BinaryOperator,
    /// Right child expression
    right: Box<Expression>,
  },
  /// Call a function with an argument list
  FunctionCall {
    /// Callable expression (usually an identifier)
    left: Box<Expression>,
    /// List of expressions
    args: Vec<Expression>,
  },
  /// Retrieve an element from a container
  Index {
    /// Container (i.e. arrays and hashmaps) to index
    left: Box<Expression>,
    /// Index
    index: Box<Expression>,
  }
}

/// Identifier for local variable names, function names, statics, etc.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Identifier(pub String);

/// Integer literal
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct IntegerLiteral(pub i64);

/// Float literal
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct FloatLiteral(pub f64);

/// Boolean literal
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct BooleanLiteral(pub bool);

/// String literal
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct StringLiteral(pub String);

/// Unary operators
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UnaryOperator {
  Neg,
}

/// Binary Operators
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BinaryOperator {
  Pow,
  Mul,
  Div,
  Rem,
  Add,
  Sub,
  Eq,
  Ne,
  Lt,
  Gt,
  Le,
  Ge,
}

