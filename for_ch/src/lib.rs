extern crate proc_macro;
use quote::*;
use syn::{
    parse::{discouraged::Speculative, Parse, ParseStream},
    parse_macro_input, Token,
};

/// A macro to flatten for-loop and if-let
///
/// while 
/// 
/// ```rust
/// for x in iter;
/// ...
/// ```
///
/// would expend to
///
/// ```rust
/// for x in iter {
///     ...
/// }
/// ```
/// 
/// and
/// 
/// ```rust
/// if let Some(x) = foo();
/// ...
/// ```
///
/// would expend to
///
/// ```rust
/// if let Some(x) = foo() {
///     ...
/// }
/// ```
///
/// and
///
/// ```rust
/// while let Some(x) = foo();
/// ...
/// ```
///
/// would expend to
///
/// ```rust
/// while let Some(x) = foo() {
///     ...
/// }
/// ```
///
/// ## Example
/// 
/// ```rust
/// for_ch! {
///     for x in 0..10; 
///     for y in x..10; // you can add a label before `for`
///     if let Some(z) = foo(x, y).await?;
///     if x - y < z { continue; } // guard
///     println!("x = {}, y = {}, z = {}", x, y, z);
/// }
/// ```
/// 
/// would expend to
/// 
/// ```rust
/// for x in 0..10 {
///     for y in x..10 {
///         if let Some(z) = foo(x, y).await? {
///             if x - y < z { continue; }
///             println!("x = {}, y = {}, z = {}", x, y, z);
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn for_ch(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ForCh);
    if input.stmts.is_empty() {
        return proc_macro::TokenStream::new();
    }

    let body = for_body(&input.stmts);
    let output = quote! {
        loop {
            #body
            break;
        }
    };

    proc_macro::TokenStream::from(output)
}

/// for x in xs;
struct ForIn {
    label: Option<syn::Label>,
    _for_tok: Token![for],
    pat: syn::Pat,
    _in_tok: Token![in],
    iter: syn::Expr,
    _semi_tok: Token![;],
}

/// if let Some(x) = option;

struct IfLet {
    _if_tok: Token![if],
    _let_tok: Token![let],
    pat: syn::Pat,
    _eq_tok: Token![=],
    expr: syn::Expr,
    _semi_tok: Token![;],
}

/// while let Some(x) = option;
struct WhileLet {
    label: Option<syn::Label>,
    _while_tok: Token![while],
    _let_tok: Token![let],
    pat: syn::Pat,
    _eq_tok: Token![=],
    expr: syn::Expr,
    _semi_tok: Token![;],
}

enum ForChItem {
    Stmt(syn::Stmt),
    IfLet(IfLet),
    WhileLet(WhileLet),
    ForIn(ForIn),
}

struct ForCh {
    stmts: Vec<ForChItem>,
}

impl Parse for ForIn {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let label = if input.peek(syn::Lifetime) && input.peek2(Token![:]) {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            label,
            _for_tok: input.parse()?,
            pat: input.parse()?,
            _in_tok: input.parse()?,
            iter: input.parse()?,
            _semi_tok: input.parse()?,
        })
    }
}

impl Parse for IfLet {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _if_tok: input.parse()?,
            _let_tok: input.parse()?,
            pat: input.parse()?,
            _eq_tok: input.parse()?,
            expr: input.parse()?,
            _semi_tok: input.parse()?,
        })
    }
}

impl Parse for WhileLet {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let label = if input.peek(syn::Lifetime) && input.peek2(Token![:]) {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self {
            label,
            _while_tok: input.parse()?,
            _let_tok: input.parse()?,
            pat: input.parse()?,
            _eq_tok: input.parse()?,
            expr: input.parse()?,
            _semi_tok: input.parse()?,
        })
    }
}


impl Parse for ForCh {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut stmts = vec![];
        while !input.is_empty() {
            let fork = input.fork();
            if let Ok(if_let) = fork.parse::<IfLet>() {
                input.advance_to(&fork);
                stmts.push(ForChItem::IfLet(if_let));
                continue;
            }

            let fork = input.fork();
            if let Ok(while_let) = fork.parse::<WhileLet>() {
                input.advance_to(&fork);
                stmts.push(ForChItem::WhileLet(while_let));
                continue;
            }

            let fork = input.fork();
            if let Ok(for_in) = fork.parse::<ForIn>() {
                input.advance_to(&fork);
                stmts.push(ForChItem::ForIn(for_in));
                continue;
            }

            stmts.push(ForChItem::Stmt(input.parse()?));
        }

        Ok(Self { stmts })
    }
}

fn for_body(stmts: &[ForChItem]) -> proc_macro2::TokenStream {
    match stmts {
        [item, rest @ ..] => {
            let rest = for_body(rest);
            match item {
                ForChItem::Stmt(s) => quote! { #s #rest },
                ForChItem::IfLet(if_let) => {
                    let pat = &if_let.pat;
                    let expr = &if_let.expr;
                    quote! {
                        if let #pat = #expr {
                            #rest
                        }
                    }
                }
                ForChItem::ForIn(for_in) => {
                    let label = &for_in.label;
                    let pat = &for_in.pat;
                    let iter = &for_in.iter;
                    quote! {
                        #label for #pat in #iter {
                            #rest
                        }
                    }
                }
                ForChItem::WhileLet(while_let) => {
                    let label = &while_let.label;
                    let pat = &while_let.pat;
                    let expr = &while_let.expr;
                    quote! {
                        #label while let #pat = #expr {
                            #rest
                        }
                    }
                },
            }
        }
        [] => proc_macro2::TokenStream::new(),
    }
}
