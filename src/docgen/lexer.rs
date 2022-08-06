use logos::Logos;

// doc comments on:
// lib comment (only first lines onwards)
// global constant vars

// type defintions
// type implementation members (vars and fns)
// macros

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
#[logos(subpattern digits = r#"(\d)([\d_]+)?"#)]
pub enum Token {
    #[regex(r#"(?:///).*"#, priority = 3)]
    DocComment,

    #[regex(r#"(0[b])?(?&digits)"#, priority = 2)]
    Int,
    #[regex(r#"(?&digits)(\.[\d_]+)?"#)]
    Float,

    #[regex(r#"\w*"(?:\\.|[^\\"])*"|'(?:\\.|[^\\'])*'"#)]
    String,

    #[token("let")]
    Let,
    #[token("mut")]
    Mut,

    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("obj")]
    Obj,
    #[token("trigger")]
    Trigger,

    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("in")]
    In,

    #[token("return")]
    Return,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,

    #[token("type")]
    TypeDef,
    #[token("impl")]
    Impl,

    #[token("print")]
    Print,
    #[token("split")]
    Split,
    #[token("add")]
    Add,

    #[token("is")]
    Is,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Mult,
    #[token("/")]
    Div,
    #[token("%")]
    Mod,
    #[token("^")]
    Pow,
    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("*=")]
    MuLte,
    #[token("/=")]
    DivEq,
    #[token("%=")]
    ModEq,
    #[token("^=")]
    PowEq,

    #[token(";")]
    Eol,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LSqBracket,
    #[token("]")]
    RSqBracket,
    #[token("{")]
    LBracket,
    #[token("}")]
    RBracket,
    #[token("!{")]
    TrigFnBracket,

    #[token(",")]
    Comma,

    #[token("==")]
    Eq,
    #[token("!=")]
    NotEq,
    #[token(">")]
    Gt,
    #[token(">=")]
    Gte,
    #[token("<")]
    Lt,
    #[token("<=")]
    Lte,

    #[token("=")]
    Assign,

    #[token(":")]
    Colon,
    #[token("::")]
    DoubleColon,

    #[token("=>")]
    FatArrow,
    #[token("->")]
    Arrow,

    #[token("?")]
    QMark,
    #[token("!")]
    ExclMark,

    #[regex(r"@[a-zA-Z_]\w*")]
    TypeIndicator,

    #[regex(r"[a-zA-Z_ඞ][a-zA-Z_0-9ඞ]*")]
    Ident,

    #[regex(r"[ \t\f\n\r]+|/\*[^*]*\*(([^/\*][^\*]*)?\*)*/|//[^\n]*", logos::skip)]
    #[error]
    Error,

    Eof,
}

impl From<Token> for &str {
    fn from(tok: Token) -> Self {
        match tok {
            Token::Int => "int literal",
            Token::Float => "float literal",
            Token::String => "string literal",
            Token::TypeIndicator => "type indicator",
            Token::Let => "let",
            Token::Mut => "mut",
            Token::Ident => "identifier",
            Token::Error => "invalid",
            Token::Eof => "end of file",
            Token::True => "true",
            Token::False => "false",
            Token::Obj => "obj",
            Token::Trigger => "trigger",
            Token::Plus => "+",
            Token::Minus => "-",
            Token::Mult => "*",
            Token::Div => "/",
            Token::Mod => "%",
            Token::Pow => "^",
            Token::PlusEq => "+=",
            Token::MinusEq => "-=",
            Token::MuLte => "*=",
            Token::DivEq => "/=",
            Token::ModEq => "%=",
            Token::PowEq => "^=",
            Token::Assign => "=",
            Token::LParen => "(",
            Token::RParen => ")",
            Token::LSqBracket => "[",
            Token::RSqBracket => "]",
            Token::LBracket => "{",
            Token::RBracket => "}",
            Token::TrigFnBracket => "!{",
            Token::Comma => ",",
            Token::Eol => ";",
            Token::If => "if",
            Token::Else => "else",
            Token::While => "while",
            Token::For => "for",
            Token::In => "in",
            Token::Return => "return",
            Token::Break => "break",
            Token::Continue => "continue",
            Token::Print => "print",
            Token::Add => "add",
            Token::Split => "split",
            Token::Is => "is",
            Token::Eq => "==",
            Token::NotEq => "!=",
            Token::Gt => ">",
            Token::Gte => ">=",
            Token::Lt => "<",
            Token::Lte => "<=",
            Token::Colon => ":",
            Token::DoubleColon => "::",
            Token::FatArrow => "=>",
            Token::Arrow => "->",
            Token::QMark => "?",
            Token::ExclMark => "!",
            Token::TypeDef => "type",
            Token::Impl => "impl",
            _ => unreachable!(),
        }
    }
}

// code explained in `parser/lexer.rs`
impl ToString for Token {
    fn to_string(&self) -> String {
        let t: &'static str = (*self).into();
        t.to_string()
    }
}

#[derive(Clone, Debug)]
pub struct Tokens(pub Vec<Token>);

impl From<Token> for Tokens {
    fn from(tok: Token) -> Self {
        Self(vec![tok])
    }
}

impl ToString for Tokens {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

impl std::ops::BitOr<Token> for Token {
    type Output = Tokens;

    fn bitor(self, rhs: Self) -> Self::Output {
        Tokens(Vec::from([self, rhs]))
    }
}

impl std::ops::BitOr<Token> for Tokens {
    type Output = Tokens;

    fn bitor(self, rhs: Token) -> Self::Output {
        let mut out = self.0;
        out.push(rhs);
        Tokens(out)
    }
}
