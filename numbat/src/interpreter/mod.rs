pub(crate) mod assert_eq;

use core::fmt;

use crate::{
    dimension::DimensionRegistry,
    markup::Markup,
    prefix_transformer::Transformer,
    pretty_print::FormatOptions,
    quantity::QuantityError,
    span::Span,
    typechecker::TypeChecker,
    typed_ast::Statement,
    unit_registry::{UnitRegistry, UnitRegistryError},
};

pub use crate::markup as m;

use assert_eq::{AssertEq2Error, AssertEq3Error};
use compact_str::{CompactString, ToCompactString};
use thiserror::Error;

pub use crate::value::Value;

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub backtrace: Vec<(CompactString, Span)>,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

#[derive(Debug, Clone, Error, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum RuntimeErrorKind {
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Expected factorial argument to be a non-negative integer")]
    FactorialOfNegativeNumber,
    #[error("Expected factorial argument to be a finite integer number")]
    FactorialOfNonInteger,
    #[error("{0}")]
    UnitRegistryError(UnitRegistryError), // TODO: can this even be triggered?
    #[error("{0}")]
    QuantityError(QuantityError),
    #[error("Assertion failed")]
    AssertFailed(Span),
    #[error("{0}")]
    AssertEq2Failed(AssertEq2Error),
    #[error("{0}")]
    AssertEq3Failed(AssertEq3Error),
    #[error("Could not load exchange rates from European Central Bank.")]
    CouldNotLoadExchangeRates,
    #[error("User error: {0}")]
    UserError(String),
    #[error("Unrecognized datetime format: {0}")]
    DateParsingError(String),
    #[error("Unknown timezone: {0}")]
    UnknownTimezone(String),
    #[error("Exceeded maximum size for time durations")]
    DurationOutOfRange,
    #[error("DateTime out of range")]
    DateTimeOutOfRange,
    #[error(
        "{0}. See https://docs.rs/jiff/latest/jiff/fmt/strtime/index.html#conversion-specifications for possible format specifiers."
    )]
    DateFormattingError(String),

    #[error("Invalid format specifiers: {0}")]
    InvalidFormatSpecifiers(String),
    #[error("Incorrect type for format specifiers: {0}")]
    InvalidTypeForFormatSpecifiers(String),

    #[error("Chemical element not found: {0}")]
    ChemicalElementNotFound(String),

    #[error("Empty array")]
    EmptyArray,

    #[error("Expected a 1-dimensional array")]
    ExpectedVector,

    #[error("Incompatible array shapes")]
    IncompatibleArrayShape,

    #[error("Expected an array shape with one or two dimensions")]
    UnsupportedArrayRank,

    #[error("Array shape entries must be non-negative integers")]
    InvalidArrayShape,

    #[error("Could not write to file: {0:?}")]
    FileWrite(std::path::PathBuf),

    #[error(
        "Could not write to history file {0:?}. History will not be saved until this is fixed."
    )]
    HistoryWrite(std::path::PathBuf),
}

#[derive(Debug, PartialEq)]
#[must_use]
pub enum InterpreterResult {
    Value(Value),
    Continue,
}

impl InterpreterResult {
    pub fn to_markup(
        &self,
        evaluated_statement: Option<&Statement>,
        registry: &DimensionRegistry,
        with_type_info: bool,
        with_equal_sign: bool,
        format_options: &FormatOptions,
    ) -> Markup {
        match self {
            Self::Value(value) => {
                let leader = if with_equal_sign {
                    m::whitespace("    ") + m::operator("=") + m::space()
                } else {
                    m::empty()
                };

                let type_markup = if with_type_info {
                    evaluated_statement
                        .and_then(Statement::as_expression)
                        .and_then(|e| {
                            let type_ = e.get_type_scheme();
                            if type_.is_scalar() {
                                None
                            } else {
                                let ty = type_.to_readable_type(registry, true);
                                Some(m::dimmed("    [") + ty + m::dimmed("]"))
                            }
                        })
                        .unwrap_or_else(m::empty)
                } else {
                    m::empty()
                };

                leader + value.pretty_print_with(format_options) + type_markup + m::nl()
            }
            Self::Continue => m::empty(),
        }
    }

    /// Returns `true` if the interpreter result is [`Value`].
    ///
    /// [`Value`]: InterpreterResult::Value
    #[must_use]
    pub fn is_value(&self) -> bool {
        matches!(self, Self::Value(..))
    }

    /// Returns `true` if the interpreter result is [`Continue`].
    ///
    /// [`Continue`]: InterpreterResult::Continue
    #[must_use]
    pub fn is_continue(&self) -> bool {
        matches!(self, Self::Continue)
    }

    pub fn value_as_string(&self) -> Option<CompactString> {
        match self {
            Self::Continue => None,
            Self::Value(value) => Some(value.to_compact_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, Box<RuntimeError>>;

pub type PrintFunction = dyn FnMut(&Markup) + Send;

pub struct InterpreterSettings {
    pub print_fn: Box<PrintFunction>,
}

impl Default for InterpreterSettings {
    fn default() -> Self {
        Self {
            print_fn: Box::new(move |s: &Markup| {
                print!("{s}");
            }),
        }
    }
}

pub trait Interpreter {
    fn new() -> Self;

    fn interpret_statements(
        &mut self,
        settings: &mut InterpreterSettings,
        statements: &[Statement],
        prefix_transformer: &Transformer,
        typechecker: &TypeChecker,
    ) -> Result<InterpreterResult>;
    fn get_unit_registry(&self) -> &UnitRegistry;
}

#[cfg(test)]
mod tests {
    use compact_str::CompactString;

    use crate::prefix_parser::AcceptsPrefix;
    use crate::quantity::Quantity;
    use crate::unit::{CanonicalName, Unit};
    use crate::{bytecode_interpreter::BytecodeInterpreter, prefix_transformer::Transformer};

    use super::*;

    static TEST_PRELUDE: &str = "
        dimension Scalar = 1

        dimension Length
        dimension Time
        dimension Mass

        dimension Velocity = Length / Time
        dimension Momentum = Mass * Velocity
        dimension Frequency = 1 / Time

        @metric_prefixes
        @aliases(m: short)
        unit meter : Length

        unit alternative_length_base_unit: Length # maybe this should be disallowed

        @aliases(s: short)
        unit second : Time

        @aliases(Hz: short)
        unit hertz: Frequency = 1 / second

        fn sin(x: Scalar) -> Scalar
        fn is_infinite(x: Scalar) -> Bool
        fn atan2<D>(y: D, x: D) -> Scalar
        fn vcat<A>(xs1: Array<A>, xs2: Array<A>) -> Array<A>
        fn hcat<A>(xs1: Array<A>, xs2: Array<A>) -> Array<A>
        fn shape<A>(x: Array<A>) -> Array<Scalar>
        fn reshape<A>(x: Array<A>, new_shape: Array<Scalar>) -> Array<A>
        fn element_at<A>(i: Scalar, x: Array<A>) -> A
        fn range(start: Scalar, end: Scalar) -> Array<Scalar>
        fn fill<A>(value: A, shape: Array<Scalar>) -> Array<A>
        fn contains<T>(x: T, xs: Array<T>) -> Bool
        fn map<T, U>(f: Fn[(T) -> U], xs: Array<T>) -> Array<U>
        fn map2<T, U, V>(f: Fn[(T, U) -> V], other: T, xs: Array<U>) -> Array<V>
        fn filter<T>(p: Fn[(T) -> Bool], xs: Array<T>) -> Array<T>
        fn foldl<T, U>(f: Fn[(T, U) -> T], acc: T, xs: Array<U>) -> T
        fn sort_by_key<T, D: Dim>(key: Fn[(T) -> D], xs: Array<T>) -> Array<T>
        fn transpose<D>(x: Array<D>) -> Array<D>
        fn zeros(shape: Array<Scalar>) -> Array<Scalar>
        fn ones(shape: Array<Scalar>) -> Array<Scalar>
        fn eye(shape: Array<Scalar>) -> Array<Scalar>
        fn identity(shape: Array<Scalar>) -> Array<Scalar> = eye(shape)
        fn split(input: String, separator: String) -> Array<String>";

    #[track_caller]
    fn get_interpreter_result(input: &str) -> Result<InterpreterResult> {
        let full_code = format!("{TEST_PRELUDE}\n{input}");
        let statements = crate::parser::parse(&full_code, 0)
            .expect("No parse errors for inputs in this test suite");
        let mut transformer = Transformer::new();
        let statements_transformed = transformer
            .transform(statements)
            .expect("No name resolution errors for inputs in this test suite");
        let mut typechecker = TypeChecker::default();
        let statements_typechecked = typechecker
            .check(&statements_transformed)
            .expect("No type check errors for inputs in this test suite");
        BytecodeInterpreter::new().interpret_statements(
            &mut InterpreterSettings::default(),
            &statements_typechecked,
            &transformer,
            &typechecker,
        )
    }

    #[track_caller]
    fn assert_evaluates_to(input: &str, expected: Quantity) {
        if let InterpreterResult::Value(actual) = get_interpreter_result(input).unwrap() {
            let actual = actual.unsafe_as_quantity();
            assert_eq!(actual, expected);
        } else {
            panic!();
        }
    }

    #[track_caller]
    fn assert_evaluates_to_scalar(input: &str, expected: f64) {
        assert_evaluates_to(input, Quantity::from_scalar(expected))
    }

    #[track_caller]
    fn assert_runtime_error(input: &str, err_expected: RuntimeErrorKind) {
        if let Err(err_actual) = get_interpreter_result(input) {
            assert_eq!(err_actual.kind, err_expected);
        } else {
            panic!();
        }
    }

    #[test]
    fn simple_arithmetic() {
        assert_evaluates_to_scalar("0", 0.0);
        assert_evaluates_to_scalar("1", 1.0);
        assert_evaluates_to_scalar("1+2", 1.0 + 2.0);
        assert_evaluates_to_scalar("-1", -1.0);

        assert_evaluates_to_scalar("2+3*4", 2.0 + 3.0 * 4.0);
        assert_evaluates_to_scalar("2*3+4", 2.0 * 3.0 + 4.0);
        assert_evaluates_to_scalar("(2+3)*4", (2.0 + 3.0) * 4.0);

        assert_evaluates_to_scalar("(2/3)*4", (2.0 / 3.0) * 4.0);
        assert_evaluates_to_scalar("-2 * 3", -2.0 * 3.0);
        assert_evaluates_to_scalar("2 * -3", 2.0 * -3.0);
        assert_evaluates_to_scalar("2 - 3 - 4", 2.0 - 3.0 - 4.0);
        assert_evaluates_to_scalar("2 - -3", 2.0 - -3.0);

        assert_evaluates_to_scalar("+2 * 3", 2.0 * 3.0);
        assert_evaluates_to_scalar("2 * +3", 2.0 * 3.0);
        assert_evaluates_to_scalar("+2 - +3", 2.0 - 3.0);
    }

    #[test]
    fn comparisons() {
        assert_evaluates_to_scalar("if 2 meter > 150 cm then 1 else 0", 1.0);

        assert_runtime_error(
            "1 meter > alternative_length_base_unit",
            RuntimeErrorKind::QuantityError(QuantityError::IncompatibleUnits(
                Unit::new_base(
                    CompactString::const_new("meter"),
                    CanonicalName::new("m", AcceptsPrefix::only_short()),
                ),
                Unit::new_base(
                    CompactString::const_new("alternative_length_base_unit"),
                    CanonicalName::new("alternative_length_base_unit", AcceptsPrefix::only_long()),
                ),
            )),
        );
    }

    #[test]
    fn arithmetic_with_units() {
        use crate::unit::Unit;

        assert_evaluates_to(
            "2 meter + 3 meter",
            Quantity::from_scalar(2.0 + 3.0) * Quantity::from_unit(Unit::meter()),
        );

        assert_evaluates_to(
            "dimension Pixel
             @aliases(px: short)
             unit pixel : Pixel
             2 * pixel",
            Quantity::from_scalar(2.0)
                * Quantity::from_unit(Unit::new_base(
                    CompactString::const_new("pixel"),
                    CanonicalName::new("px", AcceptsPrefix::only_short()),
                )),
        );

        assert_evaluates_to(
            "fn speed(distance: Length, time: Time) -> Velocity = distance / time
             speed(10 * meter, 2 * second)",
            Quantity::from_scalar(5.0)
                * (Quantity::from_unit(Unit::meter()) / Quantity::from_unit(Unit::second())),
        );
    }

    #[test]
    fn power_operator() {
        assert_evaluates_to_scalar("2^3", 2.0f64.powf(3.0));
        assert_evaluates_to_scalar("-2^4", -(2.0f64.powf(4.0)));
        assert_evaluates_to_scalar("2^(-3)", 2.0f64.powf(-3.0));
    }

    #[test]
    fn multiline_input_yields_result_of_last_line() {
        assert_evaluates_to_scalar("2\n3", 3.0);
    }

    #[test]
    fn variable_definitions() {
        assert_evaluates_to_scalar("let x = 2\nlet y = 3\nx + y", 2.0 + 3.0);
    }

    #[test]
    fn function_definitions() {
        assert_evaluates_to_scalar("fn f(x: Scalar) = 2 * x + 3\nf(5)", 2.0 * 5.0 + 3.0);
    }

    #[test]
    fn foreign_functions() {
        assert_evaluates_to_scalar("sin(1)", 1.0f64.sin());
        assert_evaluates_to_scalar("atan2(2 meter, 1 meter)", 2.0f64.atan2(1.0f64));

        assert_eq!(
            get_interpreter_result("transpose([1, 2; 3, 4])").unwrap(),
            InterpreterResult::Value(Value::Array(
                crate::value::ArrayValue::from_rows(vec![
                    vec![
                        Value::Quantity(Quantity::from_scalar(1.0)),
                        Value::Quantity(Quantity::from_scalar(3.0)),
                    ],
                    vec![
                        Value::Quantity(Quantity::from_scalar(2.0)),
                        Value::Quantity(Quantity::from_scalar(4.0)),
                    ],
                ])
                .unwrap()
            ))
        );

        assert_evaluates_to_scalar("element_at(2, [3, 2, 1, 0])", 1.0);

        assert_eq!(
            get_interpreter_result("vcat([1, 2; 3, 4], [5, 6])").unwrap(),
            InterpreterResult::Value(Value::Array(
                crate::value::ArrayValue::from_rows(vec![
                    vec![
                        Value::Quantity(Quantity::from_scalar(1.0)),
                        Value::Quantity(Quantity::from_scalar(2.0)),
                    ],
                    vec![
                        Value::Quantity(Quantity::from_scalar(3.0)),
                        Value::Quantity(Quantity::from_scalar(4.0)),
                    ],
                    vec![
                        Value::Quantity(Quantity::from_scalar(5.0)),
                        Value::Quantity(Quantity::from_scalar(6.0)),
                    ],
                ])
                .unwrap()
            ))
        );

        assert_eq!(
            get_interpreter_result("shape([1, 2; 3, 4])").unwrap(),
            InterpreterResult::Value(Value::Array(crate::value::ArrayValue::from_values(vec![
                Value::Quantity(Quantity::from_scalar(2.0)),
                Value::Quantity(Quantity::from_scalar(2.0)),
            ])))
        );

        assert_eq!(
            get_interpreter_result("reshape([1, 2, 3, 4], [2, 2])").unwrap(),
            InterpreterResult::Value(Value::Array(
                crate::value::ArrayValue::from_rows(vec![
                    vec![
                        Value::Quantity(Quantity::from_scalar(1.0)),
                        Value::Quantity(Quantity::from_scalar(2.0)),
                    ],
                    vec![
                        Value::Quantity(Quantity::from_scalar(3.0)),
                        Value::Quantity(Quantity::from_scalar(4.0)),
                    ],
                ])
                .unwrap()
            ))
        );

        assert_eq!(
            get_interpreter_result("hcat([1, 2; 3, 4], [5; 6])").unwrap(),
            InterpreterResult::Value(Value::Array(
                crate::value::ArrayValue::from_rows(vec![
                    vec![
                        Value::Quantity(Quantity::from_scalar(1.0)),
                        Value::Quantity(Quantity::from_scalar(2.0)),
                        Value::Quantity(Quantity::from_scalar(5.0)),
                    ],
                    vec![
                        Value::Quantity(Quantity::from_scalar(3.0)),
                        Value::Quantity(Quantity::from_scalar(4.0)),
                        Value::Quantity(Quantity::from_scalar(6.0)),
                    ],
                ])
                .unwrap()
            ))
        );

        assert_eq!(
            get_interpreter_result("zeros([2, 3])").unwrap(),
            InterpreterResult::Value(Value::Array(
                crate::value::ArrayValue::from_rows(vec![
                    vec![
                        Value::Quantity(Quantity::from_scalar(0.0)),
                        Value::Quantity(Quantity::from_scalar(0.0)),
                        Value::Quantity(Quantity::from_scalar(0.0)),
                    ],
                    vec![
                        Value::Quantity(Quantity::from_scalar(0.0)),
                        Value::Quantity(Quantity::from_scalar(0.0)),
                        Value::Quantity(Quantity::from_scalar(0.0)),
                    ],
                ])
                .unwrap()
            ))
        );

        assert_eq!(
            get_interpreter_result("ones([2, 2])").unwrap(),
            InterpreterResult::Value(Value::Array(
                crate::value::ArrayValue::from_rows(vec![
                    vec![
                        Value::Quantity(Quantity::from_scalar(1.0)),
                        Value::Quantity(Quantity::from_scalar(1.0)),
                    ],
                    vec![
                        Value::Quantity(Quantity::from_scalar(1.0)),
                        Value::Quantity(Quantity::from_scalar(1.0)),
                    ],
                ])
                .unwrap()
            ))
        );

        assert_eq!(
            get_interpreter_result("eye([3, 3])").unwrap(),
            InterpreterResult::Value(Value::Array(
                crate::value::ArrayValue::from_rows(vec![
                    vec![
                        Value::Quantity(Quantity::from_scalar(1.0)),
                        Value::Quantity(Quantity::from_scalar(0.0)),
                        Value::Quantity(Quantity::from_scalar(0.0)),
                    ],
                    vec![
                        Value::Quantity(Quantity::from_scalar(0.0)),
                        Value::Quantity(Quantity::from_scalar(1.0)),
                        Value::Quantity(Quantity::from_scalar(0.0)),
                    ],
                    vec![
                        Value::Quantity(Quantity::from_scalar(0.0)),
                        Value::Quantity(Quantity::from_scalar(0.0)),
                        Value::Quantity(Quantity::from_scalar(1.0)),
                    ],
                ])
                .unwrap()
            ))
        );

        assert_eq!(
            get_interpreter_result("identity([3, 3])").unwrap(),
            InterpreterResult::Value(Value::Array(
                crate::value::ArrayValue::from_rows(vec![
                    vec![
                        Value::Quantity(Quantity::from_scalar(1.0)),
                        Value::Quantity(Quantity::from_scalar(0.0)),
                        Value::Quantity(Quantity::from_scalar(0.0)),
                    ],
                    vec![
                        Value::Quantity(Quantity::from_scalar(0.0)),
                        Value::Quantity(Quantity::from_scalar(1.0)),
                        Value::Quantity(Quantity::from_scalar(0.0)),
                    ],
                    vec![
                        Value::Quantity(Quantity::from_scalar(0.0)),
                        Value::Quantity(Quantity::from_scalar(0.0)),
                        Value::Quantity(Quantity::from_scalar(1.0)),
                    ],
                ])
                .unwrap()
            ))
        );

        assert_eq!(
            get_interpreter_result("fill(\"x\", [2, 2])").unwrap(),
            InterpreterResult::Value(Value::Array(
                crate::value::ArrayValue::from_rows(vec![
                    vec![
                        Value::String(CompactString::from("x")),
                        Value::String(CompactString::from("x")),
                    ],
                    vec![
                        Value::String(CompactString::from("x")),
                        Value::String(CompactString::from("x")),
                    ],
                ])
                .unwrap()
            ))
        );

        assert_eq!(
            get_interpreter_result("map(sin, [0, 1])").unwrap(),
            InterpreterResult::Value(Value::Array(crate::value::ArrayValue::from_values(vec![
                Value::Quantity(Quantity::from_scalar(0.0f64.sin())),
                Value::Quantity(Quantity::from_scalar(1.0f64.sin())),
            ])))
        );

        assert_eq!(
            get_interpreter_result("map2(contains, 2, [[0], [2], [1, 2], []])").unwrap(),
            InterpreterResult::Value(Value::Array(crate::value::ArrayValue::from_values(vec![
                Value::Boolean(false),
                Value::Boolean(true),
                Value::Boolean(true),
                Value::Boolean(false),
            ])))
        );

        assert_eq!(
            get_interpreter_result("filter(is_infinite, [0, inf, 2, -inf])").unwrap(),
            InterpreterResult::Value(Value::Array(crate::value::ArrayValue::from_values(vec![
                Value::Quantity(Quantity::from_scalar(f64::INFINITY)),
                Value::Quantity(Quantity::from_scalar(f64::NEG_INFINITY)),
            ])))
        );

        assert_eq!(
            get_interpreter_result("range(2, 4)").unwrap(),
            InterpreterResult::Value(Value::Array(crate::value::ArrayValue::from_values(vec![
                Value::Quantity(Quantity::from_scalar(2.0)),
                Value::Quantity(Quantity::from_scalar(3.0)),
                Value::Quantity(Quantity::from_scalar(4.0)),
            ])))
        );

        assert_eq!(
            get_interpreter_result("split(\"one two three\", \" \")").unwrap(),
            InterpreterResult::Value(Value::Array(crate::value::ArrayValue::from_values(vec![
                Value::String(CompactString::from("one")),
                Value::String(CompactString::from("two")),
                Value::String(CompactString::from("three")),
            ])))
        );

        assert_eq!(
            get_interpreter_result("fn negate(x: Scalar) = -x\nsort_by_key(negate, [3, 1, 2])")
                .unwrap(),
            InterpreterResult::Value(Value::Array(crate::value::ArrayValue::from_values(vec![
                Value::Quantity(Quantity::from_scalar(3.0)),
                Value::Quantity(Quantity::from_scalar(2.0)),
                Value::Quantity(Quantity::from_scalar(1.0)),
            ])))
        );

        assert_evaluates_to_scalar(
            "fn add(x: Scalar, y: Scalar) = x + y\nfoldl(add, 0, [1, 2, 3, 4])",
            10.0,
        );
    }

    #[test]
    fn division_by_zero_raises_runtime_error() {
        assert_runtime_error("1/0", RuntimeErrorKind::DivisionByZero);
    }
}
