//! Errors that might be produced during semantic analysis

use codespan::ByteSpan;
use codespan_reporting::{Diagnostic, Label};
use moniker::{Binder, FreeVar, Var};

use pikelet_core::nbe::NbeError;
use pikelet_core::syntax;

use crate::syntax::{concrete, raw};

/// An internal error. These are bugs!
#[derive(Debug, failure::Fail, Clone, PartialEq)]
pub enum InternalError {
    #[fail(display = "Unexpected bound variable: `{}`.", var)]
    UnexpectedBoundVar { span: ByteSpan, var: Var<String> },
    #[fail(display = "not yet implemented: {}", message)]
    Unimplemented {
        span: Option<ByteSpan>,
        message: String,
    },
    #[fail(display = "nbe: {}", _0)]
    Nbe(#[cause] NbeError),
}

impl From<NbeError> for InternalError {
    fn from(src: NbeError) -> InternalError {
        InternalError::Nbe(src)
    }
}

impl InternalError {
    pub fn to_diagnostic(&self) -> Diagnostic {
        match *self {
            InternalError::UnexpectedBoundVar { span, ref var } => {
                Diagnostic::new_bug(format!("unexpected bound variable: `{}`", var)).with_label(
                    Label::new_primary(span).with_message("bound variable encountered here"),
                )
            },
            InternalError::Unimplemented { span, ref message } => {
                let base = Diagnostic::new_bug(format!("not yet implemented: {}", message));
                match span {
                    None => base,
                    Some(span) => base.with_label(
                        Label::new_primary(span)
                            .with_message("unimplemented feature encountered here"),
                    ),
                }
            },
            InternalError::Nbe(ref nbe_error) => {
                Diagnostic::new_bug(format!("failed to normalize: {}", nbe_error))
            },
        }
    }
}

/// An error produced during type checking
#[derive(Debug, failure::Fail, Clone, PartialEq)]
pub enum TypeError {
    #[fail(
        display = "Name had more than one declaration associated with it: `{}`",
        binder
    )]
    DuplicateDeclarations {
        original_span: ByteSpan,
        duplicate_span: ByteSpan,
        binder: Binder<String>,
    },
    #[fail(display = "Declaration followed definition: `{}`", binder)]
    DeclarationFollowedDefinition {
        definition_span: ByteSpan,
        declaration_span: ByteSpan,
        binder: Binder<String>,
    },
    #[fail(
        display = "Name had more than one definition associated with it: `{}`",
        binder
    )]
    DuplicateDefinitions {
        original_span: ByteSpan,
        duplicate_span: ByteSpan,
        binder: Binder<String>,
    },
    #[fail(display = "Applied an argument to a non-function type `{}`", found)]
    ArgAppliedToNonFunction {
        fn_span: ByteSpan,
        arg_span: ByteSpan,
        found: Box<concrete::Term>,
    },
    #[fail(
        display = "Type annotation needed for the function parameter `{}`",
        name
    )]
    FunctionParamNeedsAnnotation {
        param_span: ByteSpan,
        var_span: Option<ByteSpan>,
        name: FreeVar<String>,
    },
    #[fail(display = "Type annotation needed for the binder `{}`", binder)]
    BinderNeedsAnnotation {
        span: ByteSpan,
        binder: Binder<String>,
    },
    #[fail(display = "found a `{}`, but expected a type `{}`", found, expected)]
    LiteralMismatch {
        literal_span: ByteSpan,
        found: raw::Literal,
        expected: Box<concrete::Term>,
    },
    #[fail(display = "Ambiguous integer literal")]
    AmbiguousIntLiteral { span: ByteSpan },
    #[fail(display = "Ambiguous floating point literal")]
    AmbiguousFloatLiteral { span: ByteSpan },
    #[fail(display = "Empty case expressions need type annotations.")]
    AmbiguousEmptyCase { span: ByteSpan },
    #[fail(display = "Unable to elaborate hole, expected: `{:?}`", expected)]
    UnableToElaborateHole {
        span: ByteSpan,
        expected: Option<Box<concrete::Term>>,
    },
    #[fail(
        display = "Type mismatch: found `{}` but `{}` was expected",
        found, expected
    )]
    Mismatch {
        span: ByteSpan,
        found: Box<concrete::Term>,
        expected: Box<concrete::Term>,
    },
    #[fail(display = "Found a function but expected `{}`", expected)]
    UnexpectedFunction {
        span: ByteSpan,
        expected: Box<concrete::Term>,
    },
    #[fail(display = "Found `{}` but a universe was expected", found)]
    ExpectedUniverse {
        span: ByteSpan,
        found: Box<concrete::Term>,
    },
    #[fail(display = "Not yet defined: `{}`", free_var)]
    UndefinedName {
        span: ByteSpan,
        free_var: FreeVar<String>,
    },
    #[fail(display = "Undefined import `{:?}`", name)]
    UndefinedImport { span: ByteSpan, name: String },
    #[fail(
        display = "Label mismatch: found label `{}` but `{}` was expected",
        found, expected
    )]
    LabelMismatch {
        span: ByteSpan,
        found: syntax::Label,
        expected: syntax::Label,
    },
    #[fail(
        display = "Mismatched array length: expected {} elements but found {}",
        expected_len, found_len
    )]
    ArrayLengthMismatch {
        span: ByteSpan,
        found_len: u64,
        expected_len: u64,
    },
    #[fail(display = "Ambiguous record")]
    AmbiguousArrayLiteral { span: ByteSpan },
    #[fail(
        display = "The type `{}` does not contain a field named `{}`.",
        found, expected_label
    )]
    NoFieldInType {
        label_span: ByteSpan,
        expected_label: syntax::Label,
        found: Box<concrete::Term>,
    },
    #[fail(
        display = "Mismatched record size: expected {} fields but found {}",
        expected_size, found_size
    )]
    RecordSizeMismatch {
        span: ByteSpan,
        found_size: u64,
        expected_size: u64,
    },
    #[fail(display = "Internal error - this is a bug! {}", _0)]
    Internal(#[cause] InternalError),
}

impl TypeError {
    /// Convert the error into a diagnostic message
    pub fn to_diagnostic(&self) -> Diagnostic {
        match *self {
            TypeError::Internal(ref err) => err.to_diagnostic(),
            TypeError::DuplicateDeclarations {
                original_span,
                duplicate_span,
                ref binder,
            } => Diagnostic::new_error(format!(
                "name had more than one declaration associated with it `{}`",
                binder,
            ))
            .with_label(
                Label::new_primary(duplicate_span).with_message("the duplicated declaration"),
            )
            .with_label(
                Label::new_secondary(original_span).with_message("the original declaration"),
            ),
            TypeError::DeclarationFollowedDefinition {
                definition_span,
                declaration_span,
                binder: _,
            } => Diagnostic::new_error(format!("declarations cannot follow definitions"))
                .with_label(Label::new_primary(declaration_span).with_message("the declaration"))
                .with_label(
                    Label::new_secondary(definition_span).with_message("the original definition"),
                ),
            TypeError::DuplicateDefinitions {
                original_span,
                duplicate_span,
                ref binder,
            } => Diagnostic::new_error(format!(
                "name had more than one definition associated with it `{}`",
                binder,
            ))
            .with_label(
                Label::new_primary(duplicate_span).with_message("the duplicated definition"),
            )
            .with_label(
                Label::new_secondary(original_span).with_message("the original definition"),
            ),
            TypeError::ArgAppliedToNonFunction {
                fn_span,
                arg_span,
                ref found,
            } => Diagnostic::new_error(format!(
                "applied an argument to a term that was not a function - found type `{}`",
                found,
            ))
            .with_label(Label::new_primary(fn_span).with_message("the term"))
            .with_label(Label::new_secondary(arg_span).with_message("the applied argument")),
            TypeError::FunctionParamNeedsAnnotation {
                param_span,
                var_span: _, // TODO
                ref name,
            } => Diagnostic::new_error(format!(
                "type annotation needed for the function parameter `{}`",
                name
            ))
            .with_label(
                Label::new_primary(param_span)
                    .with_message("the parameter that requires an annotation"),
            ),
            TypeError::BinderNeedsAnnotation { span, ref binder } => Diagnostic::new_error(
                format!("type annotation needed for the binder `{}`", binder),
            )
            .with_label(
                Label::new_primary(span).with_message("the binder that requires an annotation"),
            ),
            TypeError::LiteralMismatch {
                literal_span,
                ref found,
                ref expected,
            } => {
                let found_text = match *found {
                    raw::Literal::String(_, _) => "string",
                    raw::Literal::Char(_, _) => "character",
                    raw::Literal::Int(_, _, _) => "numeric",
                    raw::Literal::Float(_, _, _) => "floating point",
                };

                Diagnostic::new_error(format!(
                    "found a {} literal, but expected a type `{}`",
                    found_text, expected,
                ))
                .with_label(Label::new_primary(literal_span).with_message("the literal"))
            },
            TypeError::AmbiguousIntLiteral { span } => Diagnostic::new_error(
                "ambiguous integer literal",
            )
            .with_label(Label::new_primary(span).with_message("type annotation needed here")),
            TypeError::AmbiguousFloatLiteral { span } => Diagnostic::new_error(
                "ambiguous floating point literal",
            )
            .with_label(Label::new_primary(span).with_message("type annotation needed here")),
            TypeError::AmbiguousEmptyCase { span } => Diagnostic::new_error(
                "empty case expressions need type annotations",
            )
            .with_label(Label::new_primary(span).with_message("type annotation needed here")),
            TypeError::UnableToElaborateHole {
                span,
                expected: None,
                ..
            } => Diagnostic::new_error("unable to elaborate hole")
                .with_label(Label::new_primary(span).with_message("the hole")),
            TypeError::UnableToElaborateHole {
                span,
                expected: Some(ref expected),
                ..
            } => Diagnostic::new_error(format!(
                "unable to elaborate hole - expected: `{}`",
                expected,
            ))
            .with_label(Label::new_primary(span).with_message("the hole")),
            TypeError::UnexpectedFunction {
                span, ref expected, ..
            } => Diagnostic::new_error(format!(
                "found a function but expected a term of type `{}`",
                expected,
            ))
            .with_label(Label::new_primary(span).with_message("the function")),
            TypeError::Mismatch {
                span,
                ref found,
                ref expected,
            } => Diagnostic::new_error(format!(
                "found a term of type `{}`, but expected a term of type `{}`",
                found, expected,
            ))
            .with_label(Label::new_primary(span).with_message("the term")),
            TypeError::ExpectedUniverse { ref found, span } => {
                Diagnostic::new_error(format!("expected type, found a value of type `{}`", found))
                    .with_label(Label::new_primary(span).with_message("the value"))
            },
            TypeError::UndefinedName { ref free_var, span } => {
                Diagnostic::new_bug(format!("cannot find `{}` in scope", free_var))
                    .with_label(Label::new_primary(span).with_message("not found in this scope"))
            },
            TypeError::UndefinedImport { span, ref name } => {
                Diagnostic::new_error(format!("cannot find import for `{:?}`", name))
                    .with_label(Label::new_primary(span).with_message("import not found"))
            },
            TypeError::LabelMismatch {
                span,
                ref expected,
                ref found,
            } => Diagnostic::new_error(format!(
                "expected field called `{}`, but found a field called `{}`",
                expected, found,
            ))
            .with_label(Label::new_primary(span)),
            TypeError::ArrayLengthMismatch {
                span,
                found_len,
                expected_len,
            } => Diagnostic::new_error(format!(
                "mismatched array length: expected {} elements but found {}",
                expected_len, found_len
            ))
            .with_label(
                Label::new_primary(span).with_message(format!("array with {} elements", found_len)),
            ),
            TypeError::AmbiguousArrayLiteral { span } => Diagnostic::new_error(
                "ambiguous array literal",
            )
            .with_label(Label::new_primary(span).with_message("type annotations needed here")),
            TypeError::NoFieldInType {
                label_span,
                ref expected_label,
                ref found,
            } => Diagnostic::new_error(format!(
                "the type `{}` does not contain a field called `{}`",
                found, expected_label
            ))
            .with_label(Label::new_primary(label_span).with_message("the field lookup")),
            TypeError::RecordSizeMismatch {
                span,
                found_size,
                expected_size,
            } => Diagnostic::new_error(format!(
                "mismatched record size: expected {} fields but found {}",
                expected_size, found_size
            ))
            .with_label(
                Label::new_primary(span).with_message(format!("record with {} fields", found_size)),
            ),
        }
    }
}

impl From<InternalError> for TypeError {
    fn from(src: InternalError) -> TypeError {
        TypeError::Internal(src)
    }
}

impl From<NbeError> for TypeError {
    fn from(src: NbeError) -> TypeError {
        TypeError::from(InternalError::from(src))
    }
}
