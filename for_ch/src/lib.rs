extern crate proc_macro;
use quote::*;
use syn::{
    parse::{discouraged::Speculative, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Token,
};

/// A macro to flatten for-loop and if-let
///
/// while
///
/// ```rust
/// 'label: for x in iter;
/// ...
/// ```
///
/// would expend to
///
/// ```rust
/// 'label: for x in iter {
///     ...
/// }
/// ```
///
/// and
///
/// ```rust
/// for x in iter1, for y in iter2;
/// ...
/// ```
///
/// would expend to
///
/// ```rust
/// for (x, y) in iter1.into_iter().zip(iter2) {
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
/// if expr;
/// ...
/// ```
///
/// would expend to
///
/// ```rust
/// if expr {
///     ...
/// }
/// ```
///
///
///
/// ## Example
///
/// ```rust
/// for_ch! {
///     for x in 0..10;                         // forall x in 0..10,
///     // you can add a label before `for`
///     for y in x..10, for _ in 0..5;          // forall y in x..x+5,
///     // zipping
///     if let Some(z) = foo(x, y).await?;      // exists z. Some(z) = foo(x, y).await?
///     // if let guard
///     if x - y < z;                           // satisfies x - y < z
///     // guard
///     println!("x = {}, y = {}, z = {}", x, y, z);
/// }
/// ```
///
/// would expend to
///
/// ```rust
/// for x in 0..10 {
///     for y in (x..10).zip(0..5) {
///         if let Some(z) = foo(x, y).await? {
///             if x - y < z {
///                 println!("x = {}, y = {}, z = {}", x, y, z);
///             }
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

/// for x in xs
struct ForInItem {
    _for_tok: Token![for],
    pat: syn::Pat,
    _in_tok: Token![in],
    iter: syn::Expr,
}

/// 'label: for x in xs | for y in ys | for z in zs ...;
struct ForIn {
    label: Option<syn::Label>,
    items: Punctuated<ForInItem, Token![,]>,
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

/// if expr;

struct IfGuard {
    _if_tok: Token![if],
    expr: syn::Expr,
    _semi_tok: Token![;],
}

enum ForChItem {
    Stmt(syn::Stmt),
    IfLet(IfLet),
    IfGuard(IfGuard),
    ForIn(ForIn),
}

struct ForCh {
    stmts: Vec<ForChItem>,
}

impl Parse for ForInItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _for_tok: input.parse()?,
            pat: input.parse()?,
            _in_tok: input.parse()?,
            iter: input.parse()?,
        })
    }
}

impl Parse for ForIn {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let label = if input.peek(syn::Lifetime) && input.peek2(Token![:]) {
            Some(input.parse()?)
        } else {
            None
        };

        let mut items = Punctuated::new();

        // first item
        items.push_value(input.parse()?);

        // (| for_in_item)*
        while !input.is_empty() && input.peek(Token![,]) && input.peek2(Token![for]) {
            items.push_punct(input.parse()?);
            items.push_value(input.parse()?);
        }

        Ok(Self {
            label,
            items,
            _semi_tok: input.parse()?,
        })
    }
}

impl ToTokens for ForIn {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let (pat, iter) = for_in_zippings(self.items.iter());

        self.label.to_tokens(tokens);
        quote!(for).to_tokens(tokens);
        pat.to_tokens(tokens);
        quote!(in).to_tokens(tokens);
        iter.to_tokens(tokens);
    }
}

fn for_in_zippings<'a>(
    mut items: impl Iterator<Item = &'a ForInItem>,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let (fst_pat, fst_iter) = if let Some(fst) = items.next() {
        (&fst.pat, &fst.iter)
    } else {
        return Default::default();
    };

    let (snd_pat, snd_iter) = for_in_zippings(items);
    if snd_pat.is_empty() || snd_iter.is_empty() {
        return (quote! { #fst_pat }, quote! { #fst_iter });
    }

    (
        quote! { (#fst_pat, #snd_pat) },
        quote! { (#fst_iter).into_iter().zip(#snd_iter) },
    )
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

impl Parse for IfGuard {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _if_tok: input.parse()?,
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
            if let Ok(if_guard) = fork.parse::<IfGuard>() {
                input.advance_to(&fork);
                stmts.push(ForChItem::IfGuard(if_guard));
                continue;
            }

            let fork = input.fork();
            if let Ok(if_let) = fork.parse::<IfLet>() {
                input.advance_to(&fork);
                stmts.push(ForChItem::IfLet(if_let));
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
                    quote! {
                        #for_in {
                            #rest
                        }
                    }
                }
                ForChItem::IfGuard(if_guard) => {
                    let expr = &if_guard.expr;
                    quote! {
                        if #expr {
                            #rest
                        }
                    }
                }
            }
        }
        [] => proc_macro2::TokenStream::new(),
    }
}
