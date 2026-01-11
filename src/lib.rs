//! # enum-group-macros
//!
//! Define grouped enums with ergonomic pattern matching.
//!
//! This crate allows you to define a "wire" enum with variants organized into logical groups,
//! and provides macros for matching on those groups without boilerplate.
//!
//! ## Example
//!
//! ```ignore
//! use enum_group_macros::{define_enum_group, match_enum_group, EnumGroup};
//! use serde::{Deserialize, Serialize};
//!
//! // Define message types
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! pub struct MsgA { pub value: i32 }
//!
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! pub struct MsgB { pub text: String }
//!
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! pub struct MsgC { pub flag: bool }
//!
//! // Define grouped enum
//! define_enum_group! {
//!     #[derive(Debug, Clone, Serialize, Deserialize)]
//!     #[serde(tag = "type", content = "payload")]
//!     pub enum WireMsg {
//!         Protocol {
//!             A(MsgA),
//!             B(MsgB),
//!         },
//!         Business {
//!             C(MsgC),
//!         }
//!     }
//! }
//!
//! // This generates:
//! // - enum Protocol { A(MsgA), B(MsgB) }
//! // - enum Business { C(MsgC) }
//! // - enum WireMsg { A(MsgA), B(MsgB), C(MsgC) }
//! // - enum WireMsgGroup { Protocol(Protocol), Business(Business) }
//! // - impl EnumGroup for WireMsg
//!
//! fn handle_message(msg: WireMsg) {
//!     match_enum_group!(msg, WireMsg, {
//!         Protocol(p) => {
//!             println!("Protocol message: {:?}", p);
//!         },
//!         Business(b) => {
//!             println!("Business message: {:?}", b);
//!         },
//!     })
//! }
//! ```
//!
//! ## Features
//!
//! - **Zero runtime overhead**: All grouping is compile-time
//! - **Async-friendly**: Works seamlessly with `async`/`await`
//! - **Serde compatible**: Attributes like `#[serde(...)]` are propagated
//! - **IDE support**: Full autocomplete and type checking
//!
//! ## How It Works
//!
//! The `define_enum_group!` macro generates:
//!
//! 1. **Group enums**: One enum per group (e.g., `Protocol`, `Business`)
//! 2. **Wire enum**: A flat enum with all variants for serialization
//! 3. **Group dispatch enum**: An enum wrapping group enums (e.g., `WireMsgGroup`)
//! 4. **EnumGroup impl**: Conversion from wire enum to grouped representation
//!
//! The `match_enum_group!` macro expands to a match on the grouped representation,
//! using the `EnumGroup` trait to access the `Group` type without explicit imports.

// Re-export the procedural macros
pub use enum_group_macros_impl::{define_enum_group, match_enum_group};

/// Trait for enums with grouped variants.
///
/// This trait is automatically implemented by `define_enum_group!` and provides
/// a way to convert a flat wire enum into its grouped representation.
///
/// You typically don't need to interact with this trait directly - it's used
/// internally by `match_enum_group!` to access the `Group` type.
///
/// # Example
///
/// ```ignore
/// use enum_group_macros::EnumGroup;
///
/// // After define_enum_group! generates the impl:
/// let msg: BrokerToCosignerMessage = /* ... */;
///
/// // Access the Group type via the trait
/// let grouped: <BrokerToCosignerMessage as EnumGroup>::Group = msg.into_group();
///
/// // Or more simply, use match_enum_group! which handles this for you
/// ```
pub trait EnumGroup {
  /// The grouped representation of this enum.
  ///
  /// For a wire enum `WireMsg`, this is typically `WireMsgGroup`.
  type Group;

  /// Convert this enum into its grouped representation.
  ///
  /// This method matches on each variant and wraps it in the appropriate
  /// group enum, then wraps that in the `Group` enum.
  fn into_group(self) -> Self::Group;
}
