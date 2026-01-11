//! enum-group-macros-impl - Procedural macros for enum grouping
//!
//! This is the proc-macro companion crate for `enum-group-macros`.
//! You should depend on `enum-group-macros` instead of this crate directly.
//!
//! See the `enum-group-macros` crate for documentation.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{braced, parse_macro_input, Attribute, Ident, Token, Type, Visibility};

// =============================================================================
// Custom Syntax Parser
// =============================================================================

/// Parsed representation of a single variant within a group
#[derive(Debug)]
struct ParsedVariant {
  attrs: Vec<Attribute>,
  name: Ident,
  ty: Type,
}

/// Parsed representation of a group (e.g., `SupportMessage { ... }`)
#[derive(Debug)]
struct ParsedGroup {
  name: Ident,
  variants: Vec<ParsedVariant>,
}

/// Parsed input for `define_enum_group!`
#[derive(Debug)]
struct EnumGroupInput {
  attrs: Vec<Attribute>,
  vis: Visibility,
  name: Ident,
  groups: Vec<ParsedGroup>,
}

impl Parse for ParsedVariant {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let attrs = input.call(Attribute::parse_outer)?;
    let name: Ident = input.parse()?;

    // Parse (Type)
    let content;
    syn::parenthesized!(content in input);
    let ty: Type = content.parse()?;

    Ok(ParsedVariant { attrs, name, ty })
  }
}

impl Parse for ParsedGroup {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let name: Ident = input.parse()?;

    let content;
    braced!(content in input);

    let mut variants = Vec::new();
    while !content.is_empty() {
      variants.push(content.parse::<ParsedVariant>()?);
      // Optional trailing comma
      if content.peek(Token![,]) {
        content.parse::<Token![,]>()?;
      }
    }

    Ok(ParsedGroup { name, variants })
  }
}

impl Parse for EnumGroupInput {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    // Parse outer attributes (like #[derive(...)])
    let attrs = input.call(Attribute::parse_outer)?;

    // Parse visibility and enum keyword
    let vis: Visibility = input.parse()?;
    input.parse::<Token![enum]>()?;
    let name: Ident = input.parse()?;

    // Parse the groups inside braces
    let content;
    braced!(content in input);

    let mut groups = Vec::new();
    while !content.is_empty() {
      groups.push(content.parse::<ParsedGroup>()?);
      // Handle optional comma between groups
      if content.peek(Token![,]) {
        content.parse::<Token![,]>()?;
      }
    }

    Ok(EnumGroupInput { attrs, vis, name, groups })
  }
}

// =============================================================================
// Code Generator
// =============================================================================

fn generate_enum_group(input: EnumGroupInput) -> TokenStream2 {
  let EnumGroupInput { attrs, vis, name: wire_name, groups } = input;

  let group_enum_name = format_ident!("{}Group", wire_name);

  // Collect all variants for the flat wire enum
  let mut all_variants = Vec::new();
  let mut group_enum_variants = Vec::new();
  let mut into_group_arms = Vec::new();

  // Generate group enums and collect info
  let group_enums: Vec<TokenStream2> = groups
    .iter()
    .map(|group| {
      let group_name = &group.name;

      // Variants for this group enum
      let variants: Vec<TokenStream2> = group
        .variants
        .iter()
        .map(|v| {
          let v_attrs = &v.attrs;
          let v_name = &v.name;
          let v_ty = &v.ty;
          quote! {
              #(#v_attrs)*
              #v_name(#v_ty)
          }
        })
        .collect();

      // Add to all_variants for wire enum
      for v in &group.variants {
        let v_attrs = &v.attrs;
        let v_name = &v.name;
        let v_ty = &v.ty;
        all_variants.push(quote! {
            #(#v_attrs)*
            #v_name(#v_ty)
        });

        // Generate into_group arm
        into_group_arms.push(quote! {
            Self::#v_name(v) => #group_enum_name::#group_name(#group_name::#v_name(v))
        });
      }

      // Add to group enum variants
      group_enum_variants.push(quote! {
          #group_name(#group_name)
      });

      // Generate the group enum
      quote! {
          #(#attrs)*
          #vis enum #group_name {
              #(#variants),*
          }
      }
    })
    .collect();

  // Generate the flat wire enum
  let wire_enum = quote! {
      #(#attrs)*
      #vis enum #wire_name {
          #(#all_variants),*
      }
  };

  // Generate the group dispatch enum
  let group_dispatch_enum = quote! {
      #[derive(Debug, Clone)]
      #vis enum #group_enum_name {
          #(#group_enum_variants),*
      }
  };

  // Generate an inherent into_group method (doesn't require trait import)
  let inherent_impl = quote! {
      impl #wire_name {
          /// Convert this enum into its grouped representation.
          #vis fn into_group(self) -> #group_enum_name {
              match self {
                  #(#into_group_arms),*
              }
          }
      }
  };

  // Generate the EnumGroup trait impl (for users who want trait-based access)
  let trait_impl = quote! {
      impl ::enum_group_macros::EnumGroup for #wire_name {
          type Group = #group_enum_name;

          fn into_group(self) -> Self::Group {
              // Delegate to inherent method
              #wire_name::into_group(self)
          }
      }
  };

  // Combine all generated code
  quote! {
      #(#group_enums)*

      #wire_enum

      #group_dispatch_enum

      #inherent_impl

      #trait_impl
  }
}

// =============================================================================
// Procedural Macro Entry Point
// =============================================================================

/// Defines a flat wire enum and multiple specialized categorical enums.
///
/// This macro generates:
/// 1. A set of categorical enums, each containing a subset of variants.
/// 2. A single flat "wire" enum containing all variants from all groups.
/// 3. A `Group` enum for dispatch between groups.
/// 4. An `EnumGroup` trait implementation for converting wire → group.
///
/// # Example
///
/// ```ignore
/// use enum_group_macros::define_enum_group;
/// use serde::{Deserialize, Serialize};
///
/// define_enum_group! {
///     #[derive(Debug, Clone, Serialize, Deserialize)]
///     #[serde(tag = "type", content = "payload")]
///     pub enum WireMsg {
///         Protocol {
///             A(MsgA),
///             B(MsgB),
///         },
///         Business {
///             C(MsgC),
///         }
///     }
/// }
/// ```
///
/// This generates:
/// - `enum Protocol { A(MsgA), B(MsgB) }` - categorical enum
/// - `enum Business { C(MsgC) }` - categorical enum
/// - `enum WireMsg { A(MsgA), B(MsgB), C(MsgC) }` - flat wire enum
/// - `enum WireMsgGroup { Protocol(Protocol), Business(Business) }` - dispatch enum
/// - `impl EnumGroup for WireMsg` - conversion trait
#[proc_macro]
pub fn define_enum_group(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as EnumGroupInput);
  generate_enum_group(input).into()
}

// =============================================================================
// match_enum_group! Macro
// =============================================================================

/// Matches on a grouped enum using ergonomic syntax.
///
/// This macro allows you to match on the group level without manually calling
/// `into_group()` or importing the `Group` enum.
///
/// # Example
///
/// ```ignore
/// use enum_group_macros::match_enum_group;
///
/// match_enum_group!(msg, BrokerToCosignerMessage, {
///     SupportMessage(s) => {
///         // s is SupportMessage enum
///         match s {
///             SupportMessage::ReportResponse(r) => { /* ... */ }
///             SupportMessage::HeartbeatResponse(r) => { /* ... */ }
///         }
///     },
///     BusinessMessage(b) => handle_business(b),
/// })
/// ```
#[proc_macro]
pub fn match_enum_group(input: TokenStream) -> TokenStream {
  let input2: TokenStream2 = input.into();

  let result = parse_match_enum_group(input2);

  match result {
    Ok(tokens) => tokens.into(),
    Err(e) => e.to_compile_error().into(),
  }
}

/// Parsed match arm for match_enum_group!
struct MatchArm {
  group_name: Ident,
  binding: proc_macro2::TokenStream,
  body: TokenStream2,
}

fn parse_match_enum_group(input: TokenStream2) -> syn::Result<TokenStream2> {
  use syn::parse::Parser;

  let parser = |input: ParseStream| -> syn::Result<(syn::Expr, Ident, Vec<MatchArm>)> {
    // Parse value expression
    let val: syn::Expr = input.parse()?;
    input.parse::<Token![,]>()?;

    // Parse wire enum type (just the identifier)
    let wire: Ident = input.parse()?;
    input.parse::<Token![,]>()?;

    // Parse arms block
    let content;
    braced!(content in input);

    let mut arms = Vec::new();
    while !content.is_empty() {
      // Parse: GroupName(binding) => body
      let group_name: Ident = content.parse()?;

      let paren_content;
      syn::parenthesized!(paren_content in content);
      // Parse the binding pattern (can be complex like `s` or `_`)
      let binding: proc_macro2::TokenStream = paren_content.parse()?;

      content.parse::<Token![=>]>()?;

      // Parse the body (could be a block or expression)
      let body: syn::Expr = content.parse()?;

      arms.push(MatchArm { group_name, binding, body: quote! { #body } });

      // Optional trailing comma
      if content.peek(Token![,]) {
        content.parse::<Token![,]>()?;
      }
    }

    Ok((val, wire, arms))
  };

  let (val, wire, arms) = parser.parse2(input)?;

  // Generate match arms using the local type alias
  let match_arms: Vec<TokenStream2> = arms
    .iter()
    .map(|arm| {
      let group_name = &arm.group_name;
      let binding = &arm.binding;
      let body = &arm.body;

      quote! {
          __EnumGroup__::#group_name(#binding) => #body
      }
    })
    .collect();

  // Generate expansion with local type alias
  // This avoids requiring users to import the Group type
  Ok(quote! {
      {
          #[allow(non_camel_case_types)]
          type __EnumGroup__ = <#wire as ::enum_group_macros::EnumGroup>::Group;

          match <#wire as ::enum_group_macros::EnumGroup>::into_group(#val) {
              #(#match_arms),*
          }
      }
  })
}
