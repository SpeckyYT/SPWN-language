use inflector::Inflector;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{format_ident, quote};
use syn::ext::IdentExt;
use syn::parse::{self, Parse, Parser, Peek};
use syn::punctuated::Punctuated;
use syn::{
    braced, parenthesized, parse_macro_input, token, Attribute, Block, Expr, ExprParen, ItemConst,
    Lit, Meta, Path, Token, Type, Variant,
};

macro_rules! syn_err {
    ($l:literal $(, $a:expr)*) => {
        syn_err!(proc_macro2::Span::call_site(); $l $(, $a)*)
    };
    ($s:expr; $l:literal $(, $a:expr)*) => {
        return Err(syn::Error::new($s, format!($l $(, $a)*)))
    };
}

#[derive(Debug)]
struct SpwnAttrs {
    docs: Vec<Lit>,
    raw: Vec<TokenStream>,
}

impl Parse for SpwnAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;

        let mut docs = vec![];
        let mut raw = vec![];

        for attr in attrs {
            if attr.path == Path::parse.parse_str("doc").unwrap() {
                docs.push(match attr.parse_meta()? {
                    Meta::NameValue(nv) => nv.lit,
                    _ => syn_err!(r#"expected #[doc = "..."]"#),
                });
            } else if attr.path == Path::parse.parse_str("raw").unwrap() {
                raw.push(attr.parse_args()?);
            }
        }

        Ok(Self { docs, raw })
    }
}

#[derive(Debug)]
struct TypeConstant {
    name: Ident,
    ty: Ident,
    exprs: Punctuated<Expr, Token![,]>,
    attrs: SpwnAttrs,
}

impl Parse for TypeConstant {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs: SpwnAttrs = input.parse()?;

        input.parse::<Token![const]>()?;
        let name = input.parse()?;

        input.parse::<Token![=]>()?;

        let ty: Ident = input.parse()?;
        let content;
        parenthesized!(content in input);

        let exprs = Punctuated::parse_terminated(&content)?;

        input.parse::<Token![;]>()?;

        Ok(Self {
            name,
            ty,
            exprs,
            attrs,
        })
    }
}

#[derive(Debug)]
struct Ref {
    name: Ident,
    is_ref: bool,
}

impl Parse for Ref {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let mut is_ref = false;

        if input.peek(Token![&]) {
            input.parse::<Token![&]>()?;
            is_ref = true;
        };

        Ok(Self {
            name: input.parse()?,
            is_ref,
        })
    }
}

#[derive(Debug)]
struct MacroArgWhere {
    area: Option<Ident>,
    key: Option<Ident>,
}

impl Parse for MacroArgWhere {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        mod kw {
            syn::custom_keyword!(Area);
            syn::custom_keyword!(Key);
        }

        input.parse::<Token![where]>()?;

        fn parse_bound<T: Parse, U: Parse>(input: parse::ParseStream) -> syn::Result<(U, bool)> {
            input.parse::<T>()?;
            let content;
            parenthesized!(content in input);
            let out = content.parse::<U>()?;

            if !input.peek(Ident::peek_any) {
                return Ok((out, true));
            }

            Ok((out, false))
        }

        let mut area = None;
        let mut key = None;

        loop {
            let lk = input.lookahead1();
            if lk.peek(kw::Area) && area.is_none() {
                let (a, brk) = parse_bound::<kw::Area, _>(input)?;
                if brk {
                    break;
                } else {
                    area = Some(a);
                }
            } else if lk.peek(kw::Key) && key.is_none() {
                let (k, brk) = parse_bound::<kw::Key, _>(input)?;
                if brk {
                    break;
                } else {
                    key = Some(k);
                }
            } else {
                return Err(lk.error());
            }
        }

        Ok(Self { area, key })
    }
}

#[derive(Debug)]
enum DestructureKind {
    Unit,
    Struct,
    Tuple,
}

#[derive(Debug)]
struct Destructure {
    kind: DestructureKind,
    fields: Punctuated<Ident, Token![,]>,
}

impl Parse for Destructure {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let content;

        if input.peek(token::Paren) {
            parenthesized!(content in input);

            Ok(Self {
                kind: DestructureKind::Tuple,
                fields: Punctuated::parse_terminated(&content)?,
            })
        } else if input.peek(token::Brace) {
            braced!(content in input);

            Ok(Self {
                kind: DestructureKind::Struct,
                fields: Punctuated::parse_terminated(&content)?,
            })
        } else {
            Ok(Self {
                kind: DestructureKind::Unit,
                fields: Punctuated::new(),
            })
        }
    }
}

fn parse_ident_or_self(input: parse::ParseStream) -> syn::Result<Ident> {
    if input.peek(Token![self]) {
        input.parse::<Token![self]>()?;
        Ok(Ident::new("self", input.span()))
    } else {
        input.parse()
    }
}

#[derive(Debug)]
enum ArgType {
    Spread(Ident),
    Destructure {
        binder: Ident,
        name: Ident,
        fields: Destructure,
    },
    Ref {
        binder: Ident,
        tys: Punctuated<Ref, Token![|]>,
    },
    Any(Ident),
}

impl Parse for ArgType {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        return if input.peek2(Token![:]) {
            let binder = input.parse()?;
            input.parse::<Token![:]>()?;
            Ok(Self::Ref {
                binder,
                tys: Punctuated::parse_separated_nonempty(input)?,
            })
        } else if input.peek2(Token![where]) {
            let binder = input.parse()?;
            Ok(Self::Any(binder))
        } else if input.peek(Token![...]) {
            input.parse::<Token![...]>()?;
            Ok(Self::Spread(input.parse()?))
        } else {
            let name = input.parse()?;
            let fields = input.parse()?;
            input.parse::<Token![as]>()?;
            let binder = parse_ident_or_self(input)?;
            Ok(Self::Destructure {
                binder,
                name,
                fields,
            })
        };
    }
}

#[derive(Debug)]
struct MacroArg {
    ty: ArgType,
    cwhere: Option<MacroArgWhere>,
    default: Option<Lit>,
}

impl Parse for MacroArg {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let ty = input.parse()?;

        let default = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        let cwhere = if input.peek(Token![where]) {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            ty,
            default,
            cwhere,
        })
    }
}

#[derive(Debug)]
struct TypeMacro {
    name: Ident,
    args: Punctuated<MacroArg, Token![,]>,
    ret_ty: Option<Ident>,
    block: Block,
    attrs: SpwnAttrs,
}

impl Parse for TypeMacro {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs: SpwnAttrs = input.parse()?;
        input.parse::<Token![fn]>()?;

        let name = input.parse()?;

        let content;
        parenthesized!(content in input);

        let args = Punctuated::parse_terminated(&content)?;

        let ret_ty = if input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            name,
            args,
            ret_ty,
            block: input.parse()?,
            attrs,
        })
    }
}

#[derive(Debug)]
struct TypeImpl {
    name: Ident,
    constants: Vec<TypeConstant>,
    macros: Vec<TypeMacro>,
    attrs: SpwnAttrs,
}

impl Parse for TypeImpl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs: SpwnAttrs = input.parse()?;

        input.parse::<Token![impl]>()?;
        input.parse::<Token![@]>()?;

        let name = input.parse()?;

        let content;
        braced!(content in input);

        let mut constants = vec![];
        let mut macros = vec![];

        loop {
            if content.is_empty() {
                break;
            }

            match TypeConstant::parse(&content) {
                Ok(c) => {
                    constants.push(c);
                    continue;
                }
                Err(e) => {
                    match TypeMacro::parse(&content) {
                        Ok(m) => {
                            macros.push(m);
                            continue;
                        }
                        Err(e) => return Err(e),
                    }

                    return Err(e);
                }
            };
        }

        Ok(Self {
            attrs,
            name,
            constants,
            macros,
        })
    }
}

#[proc_macro]
pub fn def_type(input: TokenStream1) -> TokenStream1 {
    let ty_impl = parse_macro_input!(input as TypeImpl);

    let name = ty_impl.name;

    let builtin_ident = format_ident!("{}", &name.to_string().to_pascal_case());
    let core_gen_ident = format_ident!("gen_{}_core", &name);

    let impl_doc = ty_impl.attrs.docs;
    let impl_raw = ty_impl.attrs.raw;

    let consts_core_gen = ty_impl.constants.iter().map(|c| {
        let raw = &c.attrs.raw;
        let docs = &c.attrs.docs;
        let name = &c.name;
        let ty = format_ident!("{}", &c.ty.to_string().to_snake_case());
        let val = "aa"; //&c.value;
        quote! {
            indoc::formatdoc!("\t{const_raw}
                \t#[doc(u{const_doc:?})]
                \t{const_name}: @{const_type} = {const_val},",
                const_raw = stringify!(#(#raw),*),
                const_doc = <[String]>::join(&[#(#docs .to_string()),*], "\n"),
                const_name = stringify!(#name),
                const_type = stringify!(#ty),
                const_val = stringify!(#val),
            )
        }
    });

    let mut macros_core_gen = vec![];
    let mut macros_codegen = vec![];

    for m in &ty_impl.macros {
        let args = &m.args;

        let arg_name = format!("{}", m.name.to_string().to_snake_case());

        for a in args {
            let mod_arg = match &a.ty {
                ArgType::Spread(_) => todo!(),
                ArgType::Destructure {
                    binder,
                    name,
                    fields,
                } => todo!(),
                ArgType::Ref { binder, tys } => {
                    let non_ref_tys = tys.pairs().map(|p| {
                        let v = p.value();
                        if !v.is_ref {
                            let name = &v.name;
                            let punc = if matches!(p.punct(), Some(_)) {
                                quote! {|}
                            } else {
                                quote! {}
                            };
                            quote! {
                                #name #punc
                            }
                        } else {
                            quote! {}
                        }
                    });

                    let ref_tys = tys.pairs().map(|p| {
                        let v = p.value();
                        if v.is_ref {
                            let name = &v.name;
                            let getter =
                                format_ident!("{}Getter", name.to_string().to_camel_case());
                            quote! {
                                #name(#getter)
                            }
                        } else {
                            quote! {}
                        }
                    });

                    let sty = if tys.len() > 1 {
                        quote! { enum }
                    } else {
                        quote! { struct }
                    };

                    quote! {
                        crate::vm::value::gen_wrapper! {
                            #sty: #(#non_ref_tys)*; #(#ref_tys)*
                        }
                    }
                }
                ArgType::Any(_) => todo!(),
            };

            macros_codegen.push(quote! {
                mod #arg_name {
                    #mod_arg
                }
            })
        }

        let raw = &m.attrs.raw;
        let docs = &m.attrs.docs;

        let name = &m.name;

        let args = m
            .args
            .iter()
            .map(|a| match &a.ty {
                ArgType::Spread(t) => format!("...{t}"),
                ArgType::Destructure { binder, name, .. } => {
                    format!("{binder}: @{}", name.to_string().to_snake_case())
                }
                ArgType::Ref { binder, tys } => format!(
                    "{binder}: {}",
                    tys.iter()
                        .map(|ty| format!("@{}", ty.name.to_string().to_snake_case()))
                        .collect::<Vec<_>>()
                        .join(" | ")
                ),
                ArgType::Any(t) => t.to_string(),
            })
            .collect::<Vec<_>>()
            .join(", ");

        let ret_ty = if let Some(ty) = &m.ret_ty {
            format!(" -> @{} ", ty.to_string().to_snake_case())
        } else {
            " ".into()
        };

        macros_core_gen.push(quote! {
            indoc::formatdoc!("\t{macro_raw}
                \t#[doc(u{macro_doc:?})]
                \t{macro_name}: ({macro_args}){macro_ret}{{
                    \t/* compiler built-in */
                \t}},",
                macro_raw = stringify!(#(#raw),*),
                macro_doc = <[String]>::join(&[#(#docs .to_string()),*], "\n"),
                macro_name = stringify!(#name),
                macro_args = #args,
                macro_ret = #ret_ty,
            )
        });
    }

    quote! {
        impl crate::vm::value::type_aliases::#builtin_ident {
            pub fn get_override_fn(self, name: &'static str) -> Option<crate::vm::value::BuiltinFn> {
               //#(#macros_codegen)*
            }
            pub fn get_override_const(self, name: &'static str) -> Option<crate::compiling::bytecode::Constant> {
                None
            }
        }

        #[cfg(test)]
        mod #core_gen_ident {
            #[test]
            pub fn #core_gen_ident() {
                let path = std::path::PathBuf::from(format!("{}{}.spwn", crate::CORE_PATH, stringify!(#name)));
                let out = indoc::formatdoc!(r#"
                        /* 
                         * This file is automatically generated when SPWN is compiled! 
                         * Do not modify or your changes will be overwritten!  
                        */
                        {impl_raw}
                        #[doc(u{impl_doc:?})]
                        impl @{typ} {{{consts}
                            {macros}
                        }}
                    "#,
                    typ = stringify!(#name),
                    impl_raw = stringify!(#(#impl_raw),*),
                    impl_doc = <[String]>::join(&[#(#impl_doc .to_string()),*], "\n"),
                    consts = <[String]>::join(&[#(#consts_core_gen),*], ""),
                    macros = <[String]>::join(&[#(#macros_core_gen),*], ""),
                );

                std::fs::write(path, &out).unwrap();
            }
        }
    }
    .into()
}
