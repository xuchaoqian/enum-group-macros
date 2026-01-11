//! Tests for the `match_enum_group!` macro.
//!
//! This file tests parsing, expansion, and runtime behavior of the matching macro.

#![allow(dead_code)] // Generated enum variants are intentionally not fully used in tests

use enum_group_macros::{define_enum_group, match_enum_group};

// =============================================================================
// Test Helper Types
// =============================================================================

/// Simple message type for testing.
#[derive(Debug, Clone, PartialEq)]
pub struct MsgA {
  pub value: i32,
}

/// Another message type.
#[derive(Debug, Clone, PartialEq)]
pub struct MsgB {
  pub text: String,
}

/// Third message type.
#[derive(Debug, Clone, PartialEq)]
pub struct MsgC {
  pub flag: bool,
}

// =============================================================================
// Shared Test Enum Definition
// =============================================================================

// Define a shared enum for multiple tests to avoid repetition
define_enum_group! {
  #[derive(Debug, Clone)]
  pub enum TestWireMsg {
    GroupAlpha {
      AlphaOne(MsgA),
      AlphaTwo(MsgB),
    },
    GroupBeta {
      BetaOne(MsgC),
    }
  }
}

// =============================================================================
// Section A: Basic Matching
// =============================================================================

/// Test: Match exhaustively on all groups.
///
/// Verifies the macro expands to a valid match expression that covers all groups.
#[test]
fn test_match_all_groups() {
  let msg_alpha = TestWireMsg::AlphaOne(MsgA { value: 42 });
  let msg_beta = TestWireMsg::BetaOne(MsgC { flag: true });

  // Match on alpha group
  let result_alpha = match_enum_group!(msg_alpha, TestWireMsg, {
    GroupAlpha(g) => format!("alpha: {:?}", g),
    GroupBeta(g) => format!("beta: {:?}", g),
  });
  assert!(result_alpha.starts_with("alpha:"));

  // Match on beta group
  let result_beta = match_enum_group!(msg_beta, TestWireMsg, {
    GroupAlpha(g) => format!("alpha: {:?}", g),
    GroupBeta(g) => format!("beta: {:?}", g),
  });
  assert!(result_beta.starts_with("beta:"));
}

/// Test: Match a single group at a time with nested match.
///
/// Verifies the matched group enum can be further pattern matched.
#[test]
fn test_match_single_group() {
  let msg = TestWireMsg::AlphaTwo(MsgB { text: "hello".to_string() });

  let result = match_enum_group!(msg, TestWireMsg, {
    GroupAlpha(inner) => {
      // Further match on the group enum
      match inner {
        GroupAlpha::AlphaOne(a) => format!("AlphaOne: {}", a.value),
        GroupAlpha::AlphaTwo(b) => format!("AlphaTwo: {}", b.text),
      }
    },
    GroupBeta(_) => "beta".to_string(),
  });

  assert_eq!(result, "AlphaTwo: hello");
}

// =============================================================================
// Section B: Binding Patterns
// =============================================================================

/// Test: Match with variable binding.
///
/// Verifies `GroupName(var)` correctly binds the group enum to a variable.
#[test]
fn test_binding_variable() {
  let msg = TestWireMsg::AlphaOne(MsgA { value: 100 });

  let result = match_enum_group!(msg, TestWireMsg, {
    GroupAlpha(captured_group) => {
      // Use the captured variable
      match captured_group {
        GroupAlpha::AlphaOne(a) => a.value,
        GroupAlpha::AlphaTwo(_) => -1,
      }
    },
    GroupBeta(_unused) => -999,
  });

  assert_eq!(result, 100);
}

/// Test: Match with underscore binding.
///
/// Verifies `GroupName(_)` discards the inner value correctly.
#[test]
fn test_binding_underscore() {
  let msg = TestWireMsg::BetaOne(MsgC { flag: false });

  let result = match_enum_group!(msg, TestWireMsg, {
    GroupAlpha(_) => "was alpha",
    GroupBeta(_) => "was beta",
  });

  assert_eq!(result, "was beta");
}

// =============================================================================
// Section C: Match Arm Bodies
// =============================================================================

/// Test: Match arm with expression body (no braces).
///
/// Verifies `=> expr` syntax works without block braces.
#[test]
fn test_body_expression() {
  let msg = TestWireMsg::AlphaOne(MsgA { value: 5 });

  // Expression body (no braces)
  let result = match_enum_group!(msg, TestWireMsg, {
    GroupAlpha(_) => 42,
    GroupBeta(_) => 0,
  });

  assert_eq!(result, 42);
}

/// Test: Match arm with block body.
///
/// Verifies `=> { ... }` syntax works with multi-statement blocks.
#[test]
fn test_body_block() {
  let msg = TestWireMsg::AlphaTwo(MsgB { text: "world".to_string() });

  // Block body with multiple statements
  let result = match_enum_group!(msg, TestWireMsg, {
    GroupAlpha(inner) => {
      let prefix = "Hello, ";
      match inner {
        GroupAlpha::AlphaOne(_) => format!("{}number", prefix),
        GroupAlpha::AlphaTwo(b) => format!("{}{}", prefix, b.text),
      }
    },
    GroupBeta(_) => {
      let msg = "beta group";
      msg.to_string()
    },
  });

  assert_eq!(result, "Hello, world");
}

/// Test: Optional trailing comma in match arms.
///
/// Verifies the macro accepts trailing commas after match arms.
#[test]
fn test_trailing_comma_in_arms() {
  let msg = TestWireMsg::BetaOne(MsgC { flag: true });

  // With trailing comma after last arm
  let result = match_enum_group!(msg, TestWireMsg, {
    GroupAlpha(_) => false,
    GroupBeta(inner) => match inner {
      GroupBeta::BetaOne(c) => c.flag,
    }, // trailing comma
  });

  assert!(result);
}

// =============================================================================
// Section D: Expression Semantics
// =============================================================================

/// Test: match_enum_group! is an expression that returns a value.
///
/// Verifies the macro can be used in expression context.
#[test]
fn test_match_returns_value() {
  let msg = TestWireMsg::AlphaOne(MsgA { value: 7 });

  // Use directly in let binding
  let value: i32 = match_enum_group!(msg, TestWireMsg, {
    GroupAlpha(inner) => match inner {
      GroupAlpha::AlphaOne(a) => a.value * 2,
      GroupAlpha::AlphaTwo(_) => 0,
    },
    GroupBeta(_) => -1,
  });

  assert_eq!(value, 14);

  // Use in arithmetic expression
  let msg2 = TestWireMsg::AlphaOne(MsgA { value: 3 });
  let computed = 10
    + match_enum_group!(msg2, TestWireMsg, {
      GroupAlpha(inner) => match inner {
        GroupAlpha::AlphaOne(a) => a.value,
        GroupAlpha::AlphaTwo(_) => 0,
      },
      GroupBeta(_) => 0,
    });

  assert_eq!(computed, 13);
}

/// Test: match_enum_group! works inside function body.
///
/// Verifies the macro integrates with normal Rust control flow.
#[test]
fn test_match_in_function() {
  fn process_message(msg: TestWireMsg) -> String {
    match_enum_group!(msg, TestWireMsg, {
      GroupAlpha(inner) => {
        match inner {
          GroupAlpha::AlphaOne(a) => format!("Processed alpha one: {}", a.value),
          GroupAlpha::AlphaTwo(b) => format!("Processed alpha two: {}", b.text),
        }
      },
      GroupBeta(inner) => {
        match inner {
          GroupBeta::BetaOne(c) => format!("Processed beta: {}", c.flag),
        }
      },
    })
  }

  assert_eq!(process_message(TestWireMsg::AlphaOne(MsgA { value: 42 })), "Processed alpha one: 42");

  assert_eq!(
    process_message(TestWireMsg::AlphaTwo(MsgB { text: "test".to_string() })),
    "Processed alpha two: test"
  );

  assert_eq!(process_message(TestWireMsg::BetaOne(MsgC { flag: false })), "Processed beta: false");
}
