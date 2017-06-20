use std::cmp::Eq;
use super::lexer::KeywordType;
use super::{imax, fmax};

/// A module, or a crate, as well as a rust source file.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ModInner<'a> {
    pub attrs:  Vec<Attr<'a>>,
    pub items:  Vec<Item<'a>>,
}

/// An Item, which is the component of a crate/module.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Item<'a> {
    pub is_pub: bool,
    pub attrs:  Vec<Attr<'a>>,
    pub detail: ItemKind<'a>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ItemKind<'a> {
    // https://doc.rust-lang.org/reference/items.html#items
    ExternCrate (&'a str),
    UseAll      (Path<'a>),
    UseSome     { path: Path<'a>, names: Vec<UseName<'a>> },
    Mod         { name: &'a str, items: Option<Vec<Item<'a>>> },
    Func        { sig: FuncSig<'a>, body: FuncBody<'a> },
    Extern      { abi: Option<&'a str>, decls: Vec<ExternFuncDecl<'a>> },
    Type        { alias: &'a str, templ: Template<'a>, origin: Ty<'a> },
    StructUnit  { name: &'a str, templ: Template<'a> },
    StructTuple { name: &'a str, templ: Template<'a>, elems: Vec<StructTupleElem<'a>> },
    StructNormal{ name: &'a str, templ: Template<'a>, fields: Vec<StructField<'a>> },
    Enum        { name: &'a str, templ: Template<'a>, vars: Vec<EnumVar<'a>> },
    Const       { name: &'a str, ty: Ty<'a>, val: Expr<'a> },
    Static      { name: &'a str, ty: Ty<'a>, val: Expr<'a> },
    Trait       { name: &'a str, templ: Template<'a>, items: Vec<TraitItem<'a>> },
    ImplType    { templ: Template<'a>, ty_for: Ty<'a>, items: Vec<ImplItem<'a>> },
    ImplTrait   { templ: Template<'a>, tr_name: TraitName<'a>, ty_for: Ty<'a>, items: Vec<ImplItem<'a>> },
}

/// The item or variable referred in a `use` declaration.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UseName<'a> {
    pub name:  &'a str,
    pub alias: Option<&'a str>,
}

/// A function declare used in `extern` block.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExternFuncDecl<'a> {
    pub sig:    FuncSig<'a>, // TODO: variadic function
    pub is_pub: bool,
    pub attrs:  Vec<Attr<'a>>,
}

/// An element of a tuple-like struct or enum variant.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StructTupleElem<'a> {
    pub is_pub: bool,
    pub attrs:  Vec<Attr<'a>>,
    pub ty:     Ty<'a>,
}

/// a field of a normal struct or enum variant.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StructField<'a> {
    pub name:   &'a str,
    pub is_pub: bool,
    pub attrs:  Vec<Attr<'a>>,
    pub ty:     Ty<'a>,
}

/// An variant of an `enum`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EnumVar<'a> {
    Unit  { name: &'a str, attrs: Vec<Attr<'a>> },
    Tuple { name: &'a str, attrs: Vec<Attr<'a>>, elems: Vec<StructTupleElem<'a>> },
    Struct{ name: &'a str, attrs: Vec<Attr<'a>>, fields: Vec<StructField<'a>> },
}

/// An item inside `trait`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TraitItem<'a> {
    Type{ name: &'a str, attrs: Vec<Attr<'a>>,
          trait_bounds: Vec<TraitName<'a>>, default: Option<Ty<'a>> },
    Func{
        sig:     FuncSig<'a>,
        attrs:   Vec<Attr<'a>>,
        default: Option<FuncBody<'a>>,
    },
}

/// An item inside `impl`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ImplItem<'a> {
    Type{ name: &'a str, attrs: Vec<Attr<'a>>, val: Ty<'a> },
    Func{
        sig:   FuncSig<'a>,
        attrs: Vec<Attr<'a>>,
        body:  FuncBody<'a>,
    },
}

/// A path, like `::std::Option`, `MyEnum::A`, etc.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Path<'a> {
    pub is_absolute: bool,
    pub comps:       Vec<PathComp<'a>>,
}

/// A path component, maybe with template hint (if any).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PathComp<'a> {
    pub body: &'a str,
    pub hint: Option<Vec<Ty<'a>>>,
}

/// Template types and trait bounds.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Template<'a> {
    pub lifetimes:    Vec<&'a str>,
    pub tys:          Vec<&'a str>,
    pub trait_bounds: Vec<TraitBound<'a>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TraitBound<'a>(pub Ty<'a>, pub Vec<TraitName<'a>>);

/// The signature of a function, including templates, trait bounds,
/// argument names and the function type.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FuncSig<'a> {
    pub name:     &'a str,
    pub templ:    Template<'a>,
    pub self_arg: SelfArg,
    pub args:     Vec<FuncArg<'a>>,
}

/// An argument of function.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FuncArg<'a> { // TODO: pattern matching for arguments
    pub name: &'a str,
    pub ty:   Ty<'a>,
}

/// The argument `self`.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SelfArg {
    /// No argument `self`. For static function or non-member-function.
    Static,
    /// fn([mut] self, ...)
    Consume{ is_mut: bool },
    /// fn(&[mut] self, ...)
    Ref{ is_mut: bool },
}

/// A type.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Ty<'a> {
    /// The placeholder `_`.
    Hole,
    /// The type `!`.
    Diverging,
    /// A generic type applied with type paramaters, like `Vec<i32>`.
    /// No paramaters indicates a normal type, like `i32`.
    Apply(TyApply<'a>),
    /// A tuple type, like `(i32, i64)`.
    Tuple(Vec<Ty<'a>>),
    /// A function pointer, like `fn(i32, u8) -> usize`.
    Func{ args: Vec<Ty<'a>>, ret: Box<Ty<'a>> },
    /// Reference.
    Ref{ is_mut: bool, lifetime: Option<&'a str>, inner: Box<Ty<'a>> },
    /// Pointers.
    Ptr{ is_mut: bool, inner: Box<Ty<'a>> },
}

/// A simple type, specialized type or trait name.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TyApply<'a> {
    pub name:      Path<'a>,
    pub lifetimes: Vec<&'a str>,
    pub params:    Vec<Ty<'a>>,
}
pub type TraitName<'a> = TyApply<'a>;

/// An attribute.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Attr<'a> {
    /// A single attribute name, like `test`, `macro_use`.
    Flag(&'a str),
    /// A key-value pair, like `crate_type = "lib"`, `recursion_limit="64"`.
    Value{ key: &'a str, value: Literal<'a> },
    /// An attribute with a list of sub-attribute arguments,
    /// like `cfg(target_os="linux")`.
    Sub(&'a str, Vec<Attr<'a>>),
}

pub type FuncBody<'a> = Vec<Token<'a>>;
pub type Expr<'a> = Vec<Token<'a>>;

/// A token or the root of a token tree.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token<'a> {
    /// A token tree delimited with `()`, `[]` or `{}`.
    Delimited(Delimiter, Vec<Token<'a>>),
    /// An inner document.
    InnerDoc(&'a str),
    /// An outer document.
    OuterDoc(&'a str),
    /// An keyword.
    Keyword(KeywordType),
    /// An identifier or `_`.
    Ident(&'a str),
    /// A lifetime.
    Lifetime(&'a str),
    /// A symbol(can't be a delimiter), always the longest. For example, `>>` will be always
    /// parsed into a single `Symbol` rather than 2 `>`s, even though it's a part of template.
    Symbol(&'static str),
    /// A literal.
    Literal(Literal<'a>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Delimiter {
    /// `()`
    Paren,
    /// `[]`
    Bracket,
    /// `{}`
    Brace,
}


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OperatorType {
    // https://doc.rust-lang.org/grammar.html#unary-operator-expressions
    Neg, Deref, Not,

    // https://doc.rust-lang.org/grammar.html#binary-operator-expressions
    Plus, Sub, Mul, Div, Mod,
    And, Or, Xor, Shl, Shr,
    LogAnd, LogOr,
    Equ, Ne, Lt, Gt, Le, Ge,
    As,
    Assign,
    AddAssign, SubAssign, MulAssign, DivAssign, ModAssign,
    AndAssign, OrAssign, XorAssign, ShlAssign, ShrAssign,
}

/// A literal.
#[derive(Debug, PartialEq, Clone)]
pub enum Literal<'a> {
    /// A char or byte char.
    CharLike { is_byte: bool, ch: char },
    /// A string, raw string, byte string or raw byte string.
    StrLike  { is_bytestr: bool, s: String },
    /// An interer type. If it has no type suffix, `ty` is None.
    IntLike  { ty: Option<Ty<'a>>, val: imax },
    /// An floating point type. If it has no type suffix, `ty` is None.
    FloatLike{ ty: Option<Ty<'a>>, val: fmax },
}

impl<'a> Eq for Literal<'a> {} // The float value is never NaN.

impl<'a> Ty<'a> {
    pub fn from_path(path: Path<'a>) -> Self {
        Ty::Apply(TyApply{ name: path, lifetimes: vec![], params: vec![] })
    }

    pub fn from_name(name: &'a str) -> Self {
        Ty::from_path(Path{
            is_absolute: false,
            comps: vec![PathComp{ body: name, hint: None }],
        })
    }

    pub fn unit() -> Self {
        Ty::Tuple(vec![])
    }
}
