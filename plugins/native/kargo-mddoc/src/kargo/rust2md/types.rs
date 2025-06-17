use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Id = String;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Crate {
    pub root: Id,
    pub crate_version: Option<String>,
    pub includes: Option<HashMap<String, String>>,
    pub index: HashMap<Id, Item>,
    pub format_version: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Item {
    pub name: Option<String>,
    pub docs: Option<String>,
    pub attrs: Vec<String>,
    pub deprecation: Option<Deprecation>,
    pub visibility: Visibility,
    pub inner: ItemEnum,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Deprecation {
    pub since: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum Visibility {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "crate")]
    Crate,
    #[serde(rename = "restricted")]
    Restricted { parent: String, path: String },
    #[serde(rename = "default")]
    Default,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum ItemEnum {
    #[serde(rename = "module")]
    Module(Module),
    #[serde(rename = "primitive")]
    Primitive(Primitive),
    #[serde(rename = "struct")]
    Struct(Struct),
    #[serde(rename = "enum")]
    Enum(Enum),
    #[serde(rename = "union")]
    Union(Union),
    #[serde(rename = "trait")]
    Trait(Trait),
    #[serde(rename = "trait_alias")]
    TraitAlias(TraitAlias),
    #[serde(rename = "impl")]
    Impl(Impl),
    #[serde(rename = "type_alias")]
    TypeAlias(TypeAlias),
    #[serde(rename = "function")]
    Function(Function),
    #[serde(rename = "extern_block")]
    ExternBlock(ExternBlock),
    #[serde(rename = "extern_type")]
    ExternType,
    #[serde(rename = "extern_crate")]
    ExternCrate {
        name: String,
        rename: Option<String>,
    },
    #[serde(rename = "use")]
    Use(Use),
    #[serde(rename = "static")]
    Static(Static),
    #[serde(rename = "constant")]
    Constant { type_: Type, const_: Constant },
    #[serde(rename = "macro")]
    Macro(String),
    #[serde(rename = "proc_macro")]
    ProcMacro(ProcMacro),
    #[serde(rename = "variant")]
    Variant(Variant),
    #[serde(rename = "struct_field")]
    StructField(Type),
    #[serde(rename = "associated_const")]
    AssocConst { type_: Type, value: Option<String> },
    #[serde(rename = "associated_type")]
    AssocType {
        generics: Generics,
        bounds: Vec<GenericBound>,
        type_: Option<Type>,
    },
}

// Module
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Module {
    pub is_crate: bool,
    pub items: Vec<Id>,
    pub is_stripped: bool,
}

// Primitives
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Primitive {
    pub name: String,
}

// Struct
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Struct {
    pub kind: StructKind,
    pub generics: Generics,
    pub impls: Vec<Id>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum StructKind {
    #[serde(rename = "unit")]
    Unit,
    #[serde(rename = "tuple")]
    Tuple(Vec<Option<Id>>),
    #[serde(rename = "plain")]
    Plain {
        fields: Vec<Id>,
        has_stripped_fields: bool,
    },
}

// Enum
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Enum {
    pub variants: Vec<Id>,
    pub generics: Generics,
    pub impls: Vec<Id>,
    pub has_stripped_variants: bool,
}

// Union
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Union {
    pub fields: Vec<Id>,
    pub generics: Generics,
    pub impls: Vec<Id>,
    pub has_stripped_fields: bool,
}

// Traits
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Trait {
    pub is_auto: bool,
    pub is_unsafe: bool,
    pub has_object_within_bounds: bool,
    pub items: Vec<Id>,
    pub bounds: Vec<GenericBound>,
    pub generics: Generics,
    pub implementations: Vec<Id>,
    pub is_dyn_compatible: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraitAlias {
    pub generics: Generics,
    pub params: Vec<GenericBound>,
}

// Implementations
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Impl {
    pub is_unsafe: bool,
    pub generics: Generics,
    pub provided_trait_methods: Vec<String>,
    pub trait_: Option<TraitRef>,
    pub for_: Type,
    pub items: Vec<Id>,
    pub is_synthetic: bool,
    pub blanket_impl: Option<Type>,
    pub is_negative: bool,
}

// Type aliases
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TypeAlias {
    pub type_: Type,
    pub generics: Generics,
}

// Functions
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Function {
    pub decl: Option<String>,
    pub header: FunctionHeader,
    pub has_body: bool,
    pub generics: Generics,
    pub sig: Signature,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FunctionHeader {
    pub const_: bool,
    pub unsafe_: bool,
    pub async_: bool,
    pub abi: Abi,
}

// For backward compatibility
impl FunctionHeader {
    pub fn is_const(&self) -> bool {
        self.const_
    }
    pub fn is_unsafe(&self) -> bool {
        self.unsafe_
    }
    pub fn is_async(&self) -> bool {
        self.async_
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", content = "content")]
pub enum Abi {
    #[serde(rename = "Rust")]
    Rust,
    #[serde(rename = "C")]
    C { unwind: bool },
    #[serde(rename = "Cdecl")]
    Cdecl { unwind: bool },
    #[serde(rename = "Stdcall")]
    Stdcall { unwind: bool },
    #[serde(rename = "Fastcall")]
    Fastcall { unwind: bool },
    #[serde(rename = "Aapcs")]
    Aapcs { unwind: bool },
    #[serde(rename = "Win64")]
    Win64 { unwind: bool },
    #[serde(rename = "SysV64")]
    SysV64 { unwind: bool },
    #[serde(rename = "System")]
    System { unwind: bool },
    #[serde(rename = "Other")]
    Other(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Signature {
    pub inputs: Vec<(String, Type)>,
    pub output: Option<Box<Type>>,
    pub is_c_variadic: bool,
}

// Extern blocks
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExternBlock {
    pub abi: Abi,
    pub items: Vec<Id>,
}

// Uses
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Use {
    pub source: String,
    pub is_glob: bool,
}

// Statics
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Static {
    pub type_: Type,
    pub expr: String,
    pub is_mutable: bool,
    pub is_unsafe: bool,
}

// Constants
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Constant {
    pub expr: String,
}

// Procedural macros
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProcMacro {
    pub kind: MacroKind,
    pub helpers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MacroKind {
    #[serde(rename = "bang")]
    Bang,
    #[serde(rename = "attr")]
    Attr,
    #[serde(rename = "derive")]
    Derive,
}

// Enum variants
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Variant {
    pub kind: VariantKind,
    pub discriminant: Option<Discriminant>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum VariantKind {
    #[serde(rename = "plain")]
    Plain,
    #[serde(rename = "tuple")]
    Tuple(Vec<Option<Id>>),
    #[serde(rename = "struct")]
    Struct {
        fields: Vec<Id>,
        has_stripped_fields: bool,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Discriminant {
    pub expr: String,
    pub value: String,
}

// Types
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum Type {
    #[serde(rename = "resolved_path")]
    ResolvedPath(Box<ResolvedPath>),
    #[serde(rename = "generic")]
    Generic(String),
    #[serde(rename = "primitive")]
    Primitive(String),
    #[serde(rename = "function_pointer")]
    FunctionPointer(Box<FnPointer>),
    #[serde(rename = "tuple")]
    Tuple(Vec<Type>),
    #[serde(rename = "slice")]
    Slice(Box<Type>),
    #[serde(rename = "array")]
    Array { type_: Box<Type>, len: String },
    #[serde(rename = "pat")]
    Pat {
        type_: Box<Type>,
        #[serde(rename = "pat_unstable_do_not_use")]
        __pat_unstable_do_not_use: String,
    },
    #[serde(rename = "dyn_trait")]
    DynTrait(DynTrait),
    #[serde(rename = "infer")]
    Infer,
    #[serde(rename = "raw_pointer")]
    RawPointer { is_mutable: bool, type_: Box<Type> },
    #[serde(rename = "borrowed_ref")]
    BorrowedRef {
        lifetime: Option<String>,
        is_mutable: bool,
        type_: Box<Type>,
    },
    #[serde(rename = "qualified_path")]
    QualifiedPath {
        name: String,
        args: GenericArgs,
        self_type: Box<Type>,
        trait_: Option<ResolvedPath>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResolvedPath {
    pub path: String,
    pub args: Option<Box<GenericArgs>>,
    pub id: Option<Id>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FnPointer {
    pub header: FunctionHeader,
    pub sig: Box<Signature>,
    pub generic_params: Vec<GenericParamDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynTrait {
    pub traits: Vec<PolyTrait>,
    pub lifetime: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PolyTrait {
    pub trait_: TraitRef,
    pub generic_params: Vec<GenericParamDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraitRef {
    pub path: String,
    pub args: Option<GenericArgs>,
}

// Generics
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Generics {
    pub params: Vec<GenericParamDef>,
    pub where_predicates: Vec<WherePredicate>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GenericParamDef {
    pub name: String,
    pub kind: GenericParamDefKind,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum GenericParamDefKind {
    #[serde(rename = "lifetime")]
    Lifetime { outlives: Vec<String> },
    #[serde(rename = "type")]
    Type {
        bounds: Vec<GenericBound>,
        default: Option<Type>,
        is_synthetic: bool,
    },
    #[serde(rename = "const")]
    Const {
        type_: Type,
        default: Option<String>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum GenericBound {
    #[serde(rename = "trait_bound")]
    TraitBound {
        trait_: TraitRef,
        generic_params: Vec<GenericParamDef>,
        modifier: TraitBoundModifier,
    },
    #[serde(rename = "outlives")]
    Outlives(String),
    #[serde(rename = "use")]
    Use(Vec<PreciseCapturingArg>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TraitBoundModifier {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "maybe")]
    Maybe,
    #[serde(rename = "maybe_const")]
    MaybeConst,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum PreciseCapturingArg {
    #[serde(rename = "lifetime")]
    Lifetime(String),
    #[serde(rename = "param")]
    Param(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum WherePredicate {
    #[serde(rename = "bound_predicate")]
    BoundPredicate {
        type_: Type,
        bounds: Vec<GenericBound>,
        generic_params: Vec<GenericParamDef>,
    },
    #[serde(rename = "lifetime_predicate")]
    LifetimePredicate {
        lifetime: String,
        outlives: Vec<String>,
    },
    #[serde(rename = "eq_predicate")]
    EqPredicate { lhs: Type, rhs: Term },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum Term {
    #[serde(rename = "type")]
    Type(Type),
    #[serde(rename = "constant")]
    Constant(Constant),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum GenericArgs {
    #[serde(rename = "angle_bracketed")]
    AngleBracketed {
        args: Vec<GenericArg>,
        constraints: Vec<AssocItemConstraint>,
    },
    #[serde(rename = "parenthesized")]
    Parenthesized {
        inputs: Vec<Type>,
        output: Option<Box<Type>>,
    },
    #[serde(rename = "return_type_notation")]
    ReturnTypeNotation,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum GenericArg {
    #[serde(rename = "lifetime")]
    Lifetime(String),
    #[serde(rename = "type")]
    Type(Type),
    #[serde(rename = "const")]
    Const(Constant),
    #[serde(rename = "infer")]
    Infer,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssocItemConstraint {
    pub name: String,
    pub args: GenericArgs,
    pub binding: AssocItemConstraintKind,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum AssocItemConstraintKind {
    #[serde(rename = "equality")]
    Equality(Term),
    #[serde(rename = "constraint")]
    Constraint(Vec<GenericBound>),
}
