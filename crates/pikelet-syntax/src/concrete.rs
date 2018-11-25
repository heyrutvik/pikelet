//! The concrete syntax of the language

use codespan::{ByteIndex, ByteSpan};
use std::fmt;

use pretty::{self, ToDoc};
use {FloatFormat, IntFormat};

/// Commands entered in the REPL
#[derive(Debug, Clone)]
pub enum ReplCommand {
    /// Evaluate a term
    ///
    /// ```text
    /// <term>
    /// ```
    Eval(Box<Term>),
    /// Show the raw representation of a term
    ///
    /// ```text
    /// :raw <term>
    /// ```
    Raw(Box<Term>),
    /// Show the core representation of a term
    ///
    /// ```text
    /// :core <term>
    /// ```
    Core(Box<Term>),
    /// Print some help about using the REPL
    ///
    /// ```text
    /// :?
    /// :h
    /// :help
    /// ```
    Help,
    /// Add a declaration to the REPL environment
    ///
    /// ```text
    ///:let <name> = <term>
    /// ```
    Let(String, Box<Term>),
    ///  No command
    NoOp,
    /// Quit the REPL
    ///
    /// ```text
    /// :q
    /// :quit
    /// ```
    Quit,
    /// Print the type of the term
    ///
    /// ```text
    /// :t <term>
    /// :type <term>
    /// ```
    TypeOf(Box<Term>),
    /// Repl commands that could not be parsed correctly
    ///
    /// This is used for error recovery
    Error(ByteSpan),
}

/// A group of lambda parameters that share an annotation
pub type FunIntroParamGroup = (Vec<(ByteIndex, String)>, Option<Box<Term>>);

/// The parameters to a lambda abstraction
pub type FunIntroParams = Vec<FunIntroParamGroup>;

/// A group of parameters to a dependent function that share an annotation
pub type FunTypeParamGroup = (Vec<(ByteIndex, String)>, Term);

/// The parameters to a dependent function type
pub type FunTypeParams = Vec<FunTypeParamGroup>;

#[derive(Debug, Clone, PartialEq)]
pub struct RecordTypeField {
    pub label: (ByteIndex, String),
    pub binder: Option<(ByteIndex, String)>,
    pub ann: Term,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordField {
    Punned {
        label: (ByteIndex, String),
        shift: Option<u32>,
    },
    Explicit {
        label: (ByteIndex, String),
        params: FunIntroParams,
        return_ann: Option<Box<Term>>,
        term: Term,
    },
}

/// Top-level items within a module
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// Declares the type associated with a name, prior to its definition
    ///
    /// ```text
    /// foo : some-type
    /// ```
    Declaration {
        name: (ByteIndex, String),
        ann: Term,
    },
    /// Defines the term that should be associated with a name
    ///
    /// ```text
    /// foo = some-body
    /// foo x (y : some-type) = some-body
    /// ```
    Definition {
        name: (ByteIndex, String),
        params: FunIntroParams,
        return_ann: Option<Box<Term>>,
        body: Term,
    },
    /// Items that could not be correctly parsed
    ///
    /// This is used for error recovery
    Error(ByteSpan),
}

impl Item {
    /// Return the span of source code that this declaration originated from
    pub fn span(&self) -> ByteSpan {
        match *self {
            Item::Definition {
                name: (start, _),
                body: ref term,
                ..
            }
            | Item::Declaration {
                name: (start, _),
                ann: ref term,
            } => ByteSpan::new(start, term.span().end()),
            Item::Error(span) => span,
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_doc().group().render_fmt(pretty::FALLBACK_WIDTH, f)
    }
}

/// Literals
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// String literals
    // TODO: Preserve escapes?
    String(ByteSpan, String),
    /// Character literals
    // TODO: Preserve escapes?
    Char(ByteSpan, char),
    /// Integer literals
    // TODO: Preserve digit separators?
    Int(ByteSpan, u64, IntFormat),
    /// Floating point literals
    // TODO: Preserve digit separators?
    Float(ByteSpan, f64, FloatFormat),
}

impl Literal {
    /// Return the span of source code that the literal originated from
    pub fn span(&self) -> ByteSpan {
        match *self {
            Literal::String(span, _)
            | Literal::Char(span, _)
            | Literal::Int(span, _, _)
            | Literal::Float(span, _, _) => span,
        }
    }
}

/// Patterns
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// A term that is surrounded with parentheses
    ///
    /// ```text
    /// (p)
    /// ```
    Parens(ByteSpan, Box<Pattern>),
    /// Patterns annotated with types
    ///
    /// ```text
    /// p : t
    /// ```
    Ann(Box<Pattern>, Box<Term>),
    /// Literal patterns
    Literal(Literal),
    /// Patterns that either introduce bound variables, or match by structural
    /// equality with a constant in-scope
    ///
    /// ```text
    /// x
    /// true
    /// false
    /// ```
    Name(ByteSpan, String, Option<u32>),
    /// Terms that could not be correctly parsed
    ///
    /// This is used for error recovery
    Error(ByteSpan),
}

impl Pattern {
    /// Return the span of source code that this pattern originated from
    pub fn span(&self) -> ByteSpan {
        match *self {
            Pattern::Parens(span, _) | Pattern::Name(span, _, _) | Pattern::Error(span) => span,
            Pattern::Ann(ref pattern, ref ty) => pattern.span().to(ty.span()),
            Pattern::Literal(ref literal) => literal.span(),
        }
    }
}

/// Terms
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    /// A term that is surrounded with parentheses
    ///
    /// ```text
    /// (e)
    /// ```
    Parens(ByteSpan, Box<Term>),
    /// A term annotated with a type
    ///
    /// ```text
    /// e : t
    /// ```
    Ann(Box<Term>, Box<Term>),
    /// Type of types
    ///
    /// ```text
    /// Type
    /// ```
    Universe(ByteSpan, Option<u32>),
    /// Literals
    Literal(Literal),
    /// Array literals
    ArrayIntro(ByteSpan, Vec<Term>),
    /// Holes
    ///
    /// ```text
    /// _
    /// ```
    Hole(ByteSpan),
    /// Names
    ///
    /// ```text
    /// x
    /// x^1
    /// ```
    Name(ByteSpan, String, Option<u32>),
    /// An imported definition
    ///
    /// ```text
    /// import "prelude"
    /// ```
    Import(ByteSpan, ByteSpan, String),
    /// Dependent function type
    ///
    /// ```text
    /// (x : t1) -> t2
    /// (x y : t1) -> t2
    /// ```
    FunType(ByteIndex, FunTypeParams, Box<Term>),
    /// Non-Dependent function type
    ///
    /// ```text
    /// t1 -> t2
    /// ```
    FunArrow(Box<Term>, Box<Term>),
    /// Function introduction
    ///
    /// ```text
    /// \x => t
    /// \x y => t
    /// \x : t1 => t2
    /// \(x : t1) y (z : t2) => t3
    /// \(x y : t1) => t3
    /// ```
    FunIntro(ByteIndex, FunIntroParams, Box<Term>),
    /// Function application
    ///
    /// ```text
    /// e1 e2
    /// ```
    FunApp(Box<Term>, Vec<Term>),
    /// Let binding
    ///
    /// ```text
    /// let x : S32
    ///     x = 1
    /// in
    ///     x
    /// ```
    Let(ByteIndex, Vec<Item>, Box<Term>),
    /// Where expressions
    ///
    /// ```text
    /// id "hello"
    /// where {
    ///     id : (A : Type) -> A -> A;
    ///     id A x = x;
    /// }
    /// ```
    Where(Box<Term>, Vec<Item>, ByteIndex),
    /// If expression
    ///
    /// ```text
    /// if t1 then t2 else t3
    /// ```
    If(ByteIndex, Box<Term>, Box<Term>, Box<Term>),
    /// Case expression
    ///
    /// ```text
    /// case t1 { pat => t2; .. }
    /// ```
    Case(ByteSpan, Box<Term>, Vec<(Pattern, Term)>),
    /// Record type
    ///
    /// ```text
    /// Record { x : t1, .. }
    /// ```
    RecordType(ByteSpan, Vec<RecordTypeField>),
    /// Record introduction
    ///
    /// ```text
    /// record { x = t1, .. }
    /// record { id (a : Type) (x : a) : a = x, .. }
    /// ```
    RecordIntro(ByteSpan, Vec<RecordField>),
    /// Record field projection
    ///
    /// ```text
    /// e.l
    /// e.l^1
    /// ```
    RecordProj(ByteSpan, Box<Term>, ByteIndex, String, Option<u32>),
    /// Terms that could not be correctly parsed
    ///
    /// This is used for error recovery
    Error(ByteSpan),
}

impl Term {
    /// Return the span of source code that this term originated from
    pub fn span(&self) -> ByteSpan {
        match *self {
            Term::Parens(span, ..)
            | Term::Universe(span, ..)
            | Term::Hole(span)
            | Term::Name(span, ..)
            | Term::Import(span, ..)
            | Term::Case(span, ..)
            | Term::RecordType(span, ..)
            | Term::RecordIntro(span, ..)
            | Term::RecordProj(span, ..)
            | Term::ArrayIntro(span, ..)
            | Term::Error(span) => span,
            Term::Literal(ref literal) => literal.span(),
            Term::FunType(start, _, ref body)
            | Term::FunIntro(start, _, ref body)
            | Term::Let(start, _, ref body)
            | Term::If(start, _, _, ref body) => ByteSpan::new(start, body.span().end()),
            Term::Where(ref expr, _, end) => ByteSpan::new(expr.span().start(), end),
            Term::Ann(ref term, ref ty) => term.span().to(ty.span()),
            Term::FunArrow(ref ann, ref body) => ann.span().to(body.span()),
            Term::FunApp(ref head, ref arg) => head.span().to(arg.last().unwrap().span()),
        }
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_doc().group().render_fmt(pretty::FALLBACK_WIDTH, f)
    }
}
