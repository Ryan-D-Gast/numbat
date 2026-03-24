mod type_checking;
mod type_inference;

use crate::Statement;
use crate::parser::parse;
use crate::prefix_transformer::Transformer;
use crate::typechecker::{Result, TypeCheckError};
use crate::typed_ast::{self, DType};

use super::TypeChecker;
use super::type_scheme::TypeScheme;

const TEST_PRELUDE: &str = "
    dimension Scalar = 1
    dimension A
    dimension B
    dimension C = A * B
    unit a: A
    unit b: B
    unit c: C = a * b

    fn returns_a() -> A = a
    fn takes_a_returns_a(x: A) -> A = x
    fn takes_a_returns_b(x: A) -> B = b
    fn takes_a_and_b_returns_c(x: A, y: B) -> C = x * y

    struct SomeStruct { a: A, b: B }

    let callable = takes_a_returns_b

    fn atan2<T>(x: T, y: T) -> Scalar

    fn len<T>(x: Array<T>) -> Scalar
    fn head<T>(x: Array<T>) -> T
    fn range(start: Scalar, end: Scalar) -> Array<Scalar>
    fn fill<T>(value: T, shape: Array<Scalar>) -> Array<T>
    fn hcat<T>(xs1: Array<T>, xs2: Array<T>) -> Array<T>
    fn map<T, U>(f: Fn[(T) -> U], xs: Array<T>) -> Array<U>
    fn map2<T, U, V>(f: Fn[(T, U) -> V], other: T, xs: Array<U>) -> Array<V>
    fn filter<T>(p: Fn[(T) -> Bool], xs: Array<T>) -> Array<T>
    fn foldl<T, U>(f: Fn[(T, U) -> T], acc: T, xs: Array<U>) -> T
    type Matrix<D: Dim, Rows: Shape, Cols: Shape> = Array<D>
    type Vector<D: Dim, N: Shape> = Array<D>
    fn transpose<D>(x: Array<D>) -> Array<D>
    fn zeros(shape: Array<Scalar>) -> Array<Scalar>
    fn ones(shape: Array<Scalar>) -> Array<Scalar>
    fn eye(shape: Array<Scalar>) -> Array<Scalar>
    fn identity(shape: Array<Scalar>) -> Array<Scalar> = eye(shape)
    fn split(input: String, separator: String) -> Array<String>

    fn id<T>(x: T) -> T = x
    fn id_for_dim<T: Dim>(x: T) -> T = x
    ";

fn type_a() -> DType {
    DType::base_dimension("A")
}

fn type_b() -> DType {
    DType::base_dimension("B")
}

fn type_c() -> DType {
    DType::base_dimension("A").multiply(&DType::base_dimension("B"))
}

fn run_typecheck(input: &str) -> Result<typed_ast::Statement<'_>> {
    let statements = parse(TEST_PRELUDE, 0)
        .expect("No parse errors for inputs in this test suite")
        .into_iter()
        .chain(parse(input, 0).expect("No parse errors for inputs in this test suite"));

    let transformed_statements = Transformer::new()
        .transform(statements)
        .map_err(|err| Box::new(err.into()))?;

    TypeChecker::default()
        .check(&transformed_statements)
        .map(|mut statements_checked| statements_checked.pop().unwrap())
}

fn assert_successful_typecheck(input: &str) {
    if let Err(err) = dbg!(run_typecheck(input)) {
        panic!("Input was expected to typecheck successfully, but failed with: {err:?}")
    }
}

fn get_inferred_fn_type(input: &str) -> TypeScheme {
    let statement = run_typecheck(input).expect("Input was expected to type-check");
    match statement {
        Statement::DefineFunction { fn_type, .. } => fn_type,
        _ => {
            unreachable!();
        }
    }
}

#[track_caller]
fn get_typecheck_error(input: &str) -> TypeCheckError {
    if let Err(err) = dbg!(run_typecheck(input)) {
        *err
    } else {
        panic!("Input was expected to yield a type check error");
    }
}
