pub mod ast;
pub mod error;
pub mod parser;

pub use ast::*;
pub use error::{ParseError, ParseResult};
pub use parser::Parser;

use neba_lexer::tokenize as lex;

pub fn parse(source: &str) -> (Program, Vec<neba_lexer::LexError>, Vec<ParseError>) {
    let (tokens, lex_errors) = lex(source);
    let mut parser = Parser::new(tokens);
    let program = parser.parse();
    (program, lex_errors, parser.errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ok(src: &str) -> Program {
        let (program, lex_errors, parse_errors) = parse(src);
        assert!(lex_errors.is_empty(),   "Lex errors: {:?}", lex_errors);
        assert!(parse_errors.is_empty(), "Parse errors: {:?}", parse_errors);
        program
    }
    fn first_stmt(src: &str) -> StmtKind { parse_ok(src).stmts.into_iter().next().unwrap().inner }
    fn first_expr(src: &str) -> ExprKind {
        match first_stmt(src) {
            StmtKind::Expr(e) => e.inner,
            StmtKind::Let { value, .. } => value.inner,
            StmtKind::Var { value, .. } => value.inner,
            other => panic!("Expected Expr/Let/Var, got {:?}", other),
        }
    }

    #[test] fn test_int_literal()   { assert_eq!(first_expr("42"), ExprKind::Int(42)); }
    #[test] fn test_float_literal() { assert_eq!(first_expr("3.14"), ExprKind::Float(3.14)); }
    #[test] fn test_bool_literal()  { assert_eq!(first_expr("true"), ExprKind::Bool(true)); }
    #[test] fn test_none_literal()  { assert_eq!(first_expr("None"), ExprKind::None); }
    #[test] fn test_string_literal()  { assert_eq!(first_expr(r#""hello""#), ExprKind::Str("hello".to_string())); }
    #[test] fn test_fstring_literal() { assert_eq!(first_expr(r#"f"hi {x}""#), ExprKind::FStr("hi {x}".to_string())); }

    #[test] fn test_let_simple() { assert!(matches!(first_stmt("let x = 42"), StmtKind::Let { name, .. } if name == "x")); }
    #[test] fn test_var_with_type() {
        match first_stmt("var name: String = \"hello\"") {
            StmtKind::Var { name, ty, .. } => {
                assert_eq!(name, "name");
                assert!(matches!(ty.unwrap().inner, TypeKind::Named(n) if n == "String"));
            }
            _ => panic!()
        }
    }
    #[test] fn test_let_generic_type() {
        match first_stmt("let x: Option[Int] = None") {
            StmtKind::Let { ty, .. } => assert!(matches!(ty.unwrap().inner, TypeKind::Generic(n, _) if n == "Option")),
            _ => panic!()
        }
    }

    #[test] fn test_addition() { assert!(matches!(first_expr("1 + 2"), ExprKind::Binary { op: BinOp::Add, .. })); }
    #[test] fn test_precedence_mul_over_add() {
        match first_expr("1 + 2 * 3") {
            ExprKind::Binary { op: BinOp::Add, right, .. } =>
                assert!(matches!(right.inner, ExprKind::Binary { op: BinOp::Mul, .. })),
            _ => panic!()
        }
    }
    #[test] fn test_power_right_associative() {
        match first_expr("2 ** 3 ** 2") {
            ExprKind::Binary { op: BinOp::Pow, right, .. } =>
                assert!(matches!(right.inner, ExprKind::Binary { op: BinOp::Pow, .. })),
            _ => panic!()
        }
    }
    #[test] fn test_unary_minus() { assert!(matches!(first_expr("-42"), ExprKind::Unary { op: UnaryOp::Neg, .. })); }
    #[test] fn test_unary_not()   { assert!(matches!(first_expr("not true"), ExprKind::Unary { op: UnaryOp::Not, .. })); }

    #[test] fn test_call_no_args()    { assert!(matches!(first_expr("foo()"), ExprKind::Call { .. })); }
    #[test] fn test_call_with_args()  { match first_expr("add(1, 2)") { ExprKind::Call { args, .. } => assert_eq!(args.len(), 2), _ => panic!() } }
    #[test] fn test_call_with_kwargs(){ match first_expr("foo(x=1, y=2)") { ExprKind::Call { kwargs, .. } => assert_eq!(kwargs.len(), 2), _ => panic!() } }
    #[test] fn test_method_call()     { assert!(matches!(first_expr("obj.method(42)"), ExprKind::Call { .. })); }
    #[test] fn test_field_access()    { assert!(matches!(first_expr("obj.field"), ExprKind::Field { field, .. } if field == "field")); }
    #[test] fn test_index_access()    { assert!(matches!(first_expr("arr[0]"), ExprKind::Index { .. })); }

    #[test] fn test_array_literal()   { match first_expr("[1, 2, 3]") { ExprKind::Array(v) => assert_eq!(v.len(), 3), _ => panic!() } }
    #[test] fn test_empty_array()     { assert!(matches!(first_expr("[]"), ExprKind::Array(v) if v.is_empty())); }
    #[test] fn test_exclusive_range() { assert!(matches!(first_expr("0..10"), ExprKind::Range { inclusive: false, .. })); }
    #[test] fn test_inclusive_range() { assert!(matches!(first_expr("0..=10"), ExprKind::Range { inclusive: true, .. })); }

    #[test] fn test_fn_no_params() { assert!(matches!(first_stmt("fn greet()\n    pass\n"), StmtKind::Fn { name, .. } if name == "greet")); }
    #[test] fn test_fn_with_params_and_return_type() {
        match first_stmt("fn add(a: Int, b: Int) -> Int\n    return a + b\n") {
            StmtKind::Fn { name, params, return_ty, .. } => {
                assert_eq!(name, "add"); assert_eq!(params.len(), 2); assert!(return_ty.is_some());
            }
            _ => panic!()
        }
    }
    #[test] fn test_async_fn() { assert!(matches!(first_stmt("async fn fetch()\n    pass\n"), StmtKind::Fn { is_async: true, .. })); }

    #[test] fn test_if_stmt()  { assert!(matches!(first_stmt("if x > 0\n    pass\n"), StmtKind::Expr(e) if matches!(e.inner, ExprKind::If { .. }))); }
    #[test] fn test_if_else()  {
        match first_stmt("if x\n    pass\nelse\n    pass\n") {
            StmtKind::Expr(e) => match e.inner { ExprKind::If { else_block, .. } => assert!(else_block.is_some()), _ => panic!() }
            _ => panic!()
        }
    }
    #[test] fn test_while_loop() { assert!(matches!(first_stmt("while x > 0\n    pass\n"), StmtKind::While { .. })); }
    #[test] fn test_for_loop()   { assert!(matches!(first_stmt("for i in 0..10\n    pass\n"), StmtKind::For { var, .. } if var == "i")); }
    #[test] fn test_return_with_value() {
        match first_stmt("fn f()\n    return 42\n") { StmtKind::Fn { body, .. } => assert!(matches!(body[0].inner, StmtKind::Return(Some(_)))), _ => panic!() }
    }
    #[test] fn test_return_without_value() {
        match first_stmt("fn f()\n    return\n") { StmtKind::Fn { body, .. } => assert!(matches!(body[0].inner, StmtKind::Return(None))), _ => panic!() }
    }

    #[test] fn test_simple_assign()   { assert!(matches!(first_stmt("x = 10"), StmtKind::Assign { op: AssignOp::Assign, .. })); }
    #[test] fn test_compound_assign() { assert!(matches!(first_stmt("x += 5"), StmtKind::Assign { op: AssignOp::AddAssign, .. })); }

    #[test] fn test_match_basic() {
        match first_stmt("match x\n    case 1 => pass\n    case _ => pass\n") {
            StmtKind::Expr(e) => match e.inner { ExprKind::Match { arms, .. } => assert_eq!(arms.len(), 2), _ => panic!() }
            _ => panic!()
        }
    }
    #[test] fn test_match_option_pattern() {
        match first_stmt("match maybe\n    case Some(v) => pass\n    case None => pass\n") {
            StmtKind::Expr(e) => match e.inner {
                ExprKind::Match { arms, .. } => {
                    assert!(matches!(&arms[0].pattern, Pattern::Constructor(n, _) if n == "Some"));
                    assert!(matches!(&arms[1].pattern, Pattern::Literal(ExprKind::None)));
                }
                _ => panic!()
            }
            _ => panic!()
        }
    }
    #[test] fn test_match_range_pattern() {
        let (_, _, errors) = parse("match score\n    case 0..=100 => pass\n");
        assert!(errors.is_empty(), "{:?}", errors);
    }

    #[test] fn test_spawn_expr() { assert!(matches!(first_expr("spawn compute(data)"), ExprKind::Spawn(_))); }
    #[test] fn test_await_expr() { assert!(matches!(first_expr("await handle"), ExprKind::Await(_))); }
    #[test] fn test_some_expr()  { assert!(matches!(first_expr("Some(42)"), ExprKind::Some(_))); }
    #[test] fn test_ok_expr()    { assert!(matches!(first_expr("Ok(value)"), ExprKind::Ok(_))); }
    #[test] fn test_err_expr()   { assert!(matches!(first_expr("Err(msg)"), ExprKind::Err(_))); }

    #[test] fn test_class_with_field() {
        match first_stmt("class Person\n    name: String\n") {
            StmtKind::Class { name, fields, .. } => { assert_eq!(name, "Person"); assert_eq!(fields.len(), 1); }
            _ => panic!()
        }
    }
    #[test] fn test_trait_definition() { assert!(matches!(first_stmt("trait Greetable\n    fn greet(self) -> String\n        pass\n"), StmtKind::Trait { name, .. } if name == "Greetable")); }
    #[test] fn test_impl_block() {
        match first_stmt("impl Greetable for Person\n    fn greet(self) -> String\n        pass\n") {
            StmtKind::Impl { trait_name, for_type, .. } => { assert_eq!(trait_name, "Greetable"); assert_eq!(for_type, Some("Person".to_string())); }
            _ => panic!()
        }
    }

    #[test] fn test_use_statement() { assert!(matches!(first_stmt("use math::sin"), StmtKind::Use(p) if p == vec!["math", "sin"])); }
    #[test] fn test_mod_statement() { assert!(matches!(first_stmt("mod math"), StmtKind::Mod(n) if n == "math")); }

    #[test] fn test_error_recovery_continues_parsing() {
        let (program, _, parse_errors) = parse("let = 42\nlet y = 10\n");
        assert!(!parse_errors.is_empty());
        assert!(!program.stmts.is_empty());
    }

    #[test] fn test_full_program() {
        let src = "let x = 42\nvar name = \"Neba\"\n\nfn add(a: Int, b: Int) -> Int\n    return a + b\n\nlet result = add(10, 20)\n\nif result > 0\n    pass\nelse\n    pass\n\nfor i in 0..5\n    pass\n\nlet maybe: Option[Int] = Some(99)\n\nmatch maybe\n    case Some(v) => pass\n    case None => pass\n";
        let (program, lex_errors, parse_errors) = parse(src);
        assert!(lex_errors.is_empty(), "Lex: {:?}", lex_errors);
        assert!(parse_errors.is_empty(), "Parse: {:?}", parse_errors);
        assert!(!program.stmts.is_empty());
    }
}
