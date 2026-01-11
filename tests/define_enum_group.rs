//! Tests for the `define_enum_group!` macro.
//!
//! This file tests parsing, code generation, trait implementation, and attribute propagation.

#![allow(dead_code)] // Generated enum variants are intentionally not fully used in tests

use enum_group_macros::{define_enum_group, EnumGroup};

// =============================================================================
// Test Helper Types
// =============================================================================

/// Simple message type for basic tests.
#[derive(Debug, Clone, PartialEq)]
struct MsgA {
  pub value: i32,
}

/// Another simple message type.
#[derive(Debug, Clone, PartialEq)]
struct MsgB {
  pub text: String,
}

/// Third message type for multi-variant tests.
#[derive(Debug, Clone, PartialEq)]
struct MsgC {
  pub flag: bool,
}

/// Fourth message type for complex tests.
#[derive(Debug, Clone, PartialEq)]
struct MsgD {
  pub data: Vec<u8>,
}

// =============================================================================
// Section A: Syntax Acceptance
// =============================================================================

/// Test: Minimal definition with one group containing one variant.
///
/// Verifies the macro accepts the simplest possible input:
/// - Single group
/// - Single variant
/// - Basic derive attributes
#[test]
fn test_minimal_definition() {
  define_enum_group! {
    #[derive(Debug, Clone)]
    enum MinimalMsg {
      OnlyGroup {
        OnlyVariant(MsgA),
      }
    }
  }

  // Verify wire enum can be instantiated
  let msg = MinimalMsg::OnlyVariant(MsgA { value: 42 });

  // Verify into_group works
  let grouped = msg.into_group();
  assert!(matches!(grouped, MinimalMsgGroup::OnlyGroup(OnlyGroup::OnlyVariant(_))));
}

/// Test: Multiple groups with multiple variants each.
///
/// Verifies the macro handles complex definitions:
/// - Multiple groups
/// - Multiple variants per group
/// - Proper variant-to-group mapping
#[test]
fn test_multiple_groups_multiple_variants() {
  define_enum_group! {
    #[derive(Debug, Clone)]
    enum ComplexMsg {
      GroupAlpha {
        VariantA(MsgA),
        VariantB(MsgB),
      },
      GroupBeta {
        VariantC(MsgC),
        VariantD(MsgD),
      }
    }
  }

  // Test variants from GroupAlpha
  let msg_a = ComplexMsg::VariantA(MsgA { value: 1 });
  assert!(matches!(msg_a.into_group(), ComplexMsgGroup::GroupAlpha(_)));

  let msg_b = ComplexMsg::VariantB(MsgB { text: "hello".to_string() });
  assert!(matches!(msg_b.into_group(), ComplexMsgGroup::GroupAlpha(_)));

  // Test variants from GroupBeta
  let msg_c = ComplexMsg::VariantC(MsgC { flag: true });
  assert!(matches!(msg_c.into_group(), ComplexMsgGroup::GroupBeta(_)));

  let msg_d = ComplexMsg::VariantD(MsgD { data: vec![1, 2, 3] });
  assert!(matches!(msg_d.into_group(), ComplexMsgGroup::GroupBeta(_)));
}

/// Test: Trailing commas after variants and groups.
///
/// Verifies the macro accepts optional trailing commas in all positions.
#[test]
fn test_trailing_commas_everywhere() {
  define_enum_group! {
    #[derive(Debug, Clone)]
    enum TrailingCommaMsg {
      Group1 {
        Var1(MsgA),
        Var2(MsgB), // trailing comma after last variant
      }, // trailing comma after group
      Group2 {
        Var3(MsgC),
      },
    }
  }

  let msg = TrailingCommaMsg::Var1(MsgA { value: 1 });
  assert!(matches!(msg.into_group(), TrailingCommaMsgGroup::Group1(_)));
}

/// Test: No trailing commas anywhere.
///
/// Verifies the macro works without any trailing commas.
#[test]
fn test_no_trailing_commas() {
  define_enum_group! {
    #[derive(Debug, Clone)]
    enum NoTrailingMsg {
      Group1 {
        Var1(MsgA)
      }
      Group2 {
        Var2(MsgB)
      }
    }
  }

  let msg = NoTrailingMsg::Var1(MsgA { value: 1 });
  assert!(matches!(msg.into_group(), NoTrailingMsgGroup::Group1(_)));
}

/// Test: Empty group with no variants.
///
/// Verifies the macro accepts groups with zero variants (edge case for loop coverage).
#[test]
fn test_empty_group() {
  define_enum_group! {
    #[derive(Debug, Clone)]
    enum EmptyGroupMsg {
      EmptyGroup {
        // No variants - tests the `while !content.is_empty()` path
      },
      NonEmptyGroup {
        SomeVariant(MsgA),
      }
    }
  }

  // Can only instantiate from non-empty group
  let msg = EmptyGroupMsg::SomeVariant(MsgA { value: 1 });
  assert!(matches!(msg.into_group(), EmptyGroupMsgGroup::NonEmptyGroup(_)));
}

// =============================================================================
// Section B: Visibility Modifiers
// =============================================================================

/// Test: Public visibility (`pub enum`).
///
/// Verifies the macro propagates `pub` visibility to all generated types.
#[test]
fn test_pub_visibility() {
  mod inner {
    use super::MsgA;
    use enum_group_macros::define_enum_group;

    define_enum_group! {
      #[derive(Debug, Clone)]
      pub enum PubMsg {
        PubGroup {
          PubVariant(MsgA),
        }
      }
    }
  }

  // Access from outer scope - proves visibility is public
  let msg = inner::PubMsg::PubVariant(MsgA { value: 1 });
  let _grouped: inner::PubMsgGroup = msg.into_group();
}

/// Test: Crate-level visibility (`pub(crate) enum`).
///
/// Verifies the macro accepts restricted visibility modifiers.
#[test]
fn test_pub_crate_visibility() {
  mod inner {
    use super::MsgA;
    use enum_group_macros::define_enum_group;

    define_enum_group! {
      #[derive(Debug, Clone)]
      pub(crate) enum PubCrateMsg {
        CrateGroup {
          CrateVariant(MsgA),
        }
      }
    }
  }

  // Access from same crate
  let msg = inner::PubCrateMsg::CrateVariant(MsgA { value: 1 });
  let _grouped: inner::PubCrateMsgGroup = msg.into_group();
}

/// Test: Private visibility (no modifier).
///
/// Verifies the macro works with default (inherited) visibility.
#[test]
fn test_private_visibility() {
  define_enum_group! {
    #[derive(Debug, Clone)]
    enum PrivateMsg {
      PrivateGroup {
        PrivateVariant(MsgA),
      }
    }
  }

  let msg = PrivateMsg::PrivateVariant(MsgA { value: 1 });
  let _grouped: PrivateMsgGroup = msg.into_group();
}

// =============================================================================
// Section C: Type Variations
// =============================================================================

/// Test: Generic types in variants.
///
/// Verifies the macro correctly parses generic types like `Option<T>` and `Vec<T>`.
#[test]
fn test_generic_types() {
  define_enum_group! {
    #[derive(Debug, Clone)]
    enum GenericTypesMsg {
      Generics {
        OptionalValue(Option<i32>),
        VectorValue(Vec<u8>),
        NestedGeneric(Option<Vec<String>>),
      }
    }
  }

  let msg1 = GenericTypesMsg::OptionalValue(Some(42));
  assert!(matches!(msg1.into_group(), GenericTypesMsgGroup::Generics(_)));

  let msg2 = GenericTypesMsg::VectorValue(vec![1, 2, 3]);
  assert!(matches!(msg2.into_group(), GenericTypesMsgGroup::Generics(_)));

  let msg3 = GenericTypesMsg::NestedGeneric(Some(vec!["a".to_string()]));
  assert!(matches!(msg3.into_group(), GenericTypesMsgGroup::Generics(_)));
}

/// Test: Path types in variants.
///
/// Verifies the macro correctly parses fully qualified type paths.
#[test]
fn test_path_types() {
  define_enum_group! {
    #[derive(Debug, Clone)]
    enum PathTypesMsg {
      Paths {
        StdString(std::string::String),
        BoxedSlice(Box<[u8]>),
      }
    }
  }

  let msg1 = PathTypesMsg::StdString("hello".to_string());
  assert!(matches!(msg1.into_group(), PathTypesMsgGroup::Paths(_)));

  let msg2 = PathTypesMsg::BoxedSlice(vec![1, 2, 3].into_boxed_slice());
  assert!(matches!(msg2.into_group(), PathTypesMsgGroup::Paths(_)));
}

// =============================================================================
// Section D: Attribute Propagation
// =============================================================================

/// Test: Derive attributes propagate to all generated enums.
///
/// Verifies that `#[derive(...)]` attributes are applied to:
/// - Group enums
/// - Wire enum
/// - Dispatch enum (always gets Debug, Clone)
#[test]
fn test_derive_attributes() {
  define_enum_group! {
    #[derive(Debug, Clone, PartialEq)]
    enum DeriveMsg {
      Group1 {
        Var1(MsgA),
      }
    }
  }

  // Test Debug
  let msg = DeriveMsg::Var1(MsgA { value: 42 });
  let debug_str = format!("{:?}", msg);
  assert!(debug_str.contains("Var1"));

  // Test Clone
  let cloned = msg.clone();
  assert_eq!(msg, cloned);

  // Test PartialEq
  let msg2 = DeriveMsg::Var1(MsgA { value: 42 });
  assert_eq!(msg, msg2);
}

/// Test: Serde tag/content attributes.
///
/// Verifies that `#[serde(...)]` attributes are propagated correctly.
#[test]
fn test_serde_tag_content() {
  use serde::{Deserialize, Serialize};

  // Need serde derives on test types
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  struct SerdeMsg {
    value: i32,
  }

  define_enum_group! {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", content = "payload")]
    enum TaggedMsg {
      Tagged {
        Data(SerdeMsg),
      }
    }
  }

  let msg = TaggedMsg::Data(SerdeMsg { value: 42 });
  let json = serde_json::to_string(&msg).expect("serialize failed");

  // Verify tag/content format
  assert!(json.contains("\"type\""));
  assert!(json.contains("\"payload\""));
  assert!(json.contains("\"Data\""));
}

/// Test: Variant-level attributes.
///
/// Verifies that attributes on individual variants are preserved.
#[test]
fn test_variant_level_attributes() {
  use serde::{Deserialize, Serialize};

  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  struct RenamedPayload {
    data: String,
  }

  define_enum_group! {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", content = "payload")]
    enum VariantAttrMsg {
      Attrs {
        #[serde(rename = "custom_name")]
        OriginalName(RenamedPayload),
      }
    }
  }

  let msg = VariantAttrMsg::OriginalName(RenamedPayload { data: "test".to_string() });
  let json = serde_json::to_string(&msg).expect("serialize failed");

  // Verify the variant was renamed in serialization
  assert!(json.contains("\"custom_name\""));
  assert!(!json.contains("\"OriginalName\""));
}

// =============================================================================
// Section E: Generated Code Structure
// =============================================================================

/// Test: Group enums are generated and accessible.
///
/// Verifies each group becomes its own enum type.
#[test]
fn test_group_enums_exist() {
  define_enum_group! {
    #[derive(Debug, Clone)]
    enum StructureMsg {
      Alpha {
        A1(MsgA),
        A2(MsgB),
      },
      Beta {
        B1(MsgC),
      }
    }
  }

  // Directly instantiate group enums (not through wire enum)
  let alpha: Alpha = Alpha::A1(MsgA { value: 1 });
  let beta: Beta = Beta::B1(MsgC { flag: true });

  // Verify they are the expected types
  assert!(matches!(alpha, Alpha::A1(_)));
  assert!(matches!(beta, Beta::B1(_)));
}

/// Test: Wire enum contains all variants from all groups.
///
/// Verifies the wire enum is flattened correctly.
#[test]
fn test_wire_enum_flattened() {
  define_enum_group! {
    #[derive(Debug, Clone)]
    enum FlattenedMsg {
      Group1 {
        FromG1A(MsgA),
        FromG1B(MsgB),
      },
      Group2 {
        FromG2C(MsgC),
      }
    }
  }

  // All variants are accessible directly on wire enum
  let _: FlattenedMsg = FlattenedMsg::FromG1A(MsgA { value: 1 });
  let _: FlattenedMsg = FlattenedMsg::FromG1B(MsgB { text: "x".to_string() });
  let _: FlattenedMsg = FlattenedMsg::FromG2C(MsgC { flag: false });
}

/// Test: Dispatch enum follows {Name}Group naming convention.
///
/// Verifies the generated dispatch enum has the correct name.
#[test]
fn test_dispatch_enum_naming() {
  define_enum_group! {
    #[derive(Debug, Clone)]
    enum NamingTest {
      SomeGroup {
        Variant(MsgA),
      }
    }
  }

  // The dispatch enum should be named NamingTestGroup
  let msg = NamingTest::Variant(MsgA { value: 1 });
  let grouped: NamingTestGroup = msg.into_group();
  assert!(matches!(grouped, NamingTestGroup::SomeGroup(_)));
}

/// Test: Inherent `into_group()` method works.
///
/// Verifies the method is callable without importing the trait.
#[test]
fn test_into_group_method() {
  define_enum_group! {
    #[derive(Debug, Clone)]
    enum InherentMethodMsg {
      Group1 {
        Var1(MsgA),
      },
      Group2 {
        Var2(MsgB),
      }
    }
  }

  let msg1 = InherentMethodMsg::Var1(MsgA { value: 1 });
  let msg2 = InherentMethodMsg::Var2(MsgB { text: "hi".to_string() });

  // Call inherent method (no trait import needed beyond what's in scope)
  let group1 = msg1.into_group();
  let group2 = msg2.into_group();

  // Verify correct grouping
  assert!(matches!(group1, InherentMethodMsgGroup::Group1(Group1::Var1(_))));
  assert!(matches!(group2, InherentMethodMsgGroup::Group2(Group2::Var2(_))));
}

/// Test: EnumGroup trait is implemented.
///
/// Verifies the trait implementation allows generic usage.
#[test]
fn test_enum_group_trait_impl() {
  define_enum_group! {
    #[derive(Debug, Clone)]
    enum TraitImplMsg {
      OnlyGroup {
        OnlyVar(MsgA),
      }
    }
  }

  // Use the trait bound in a generic function
  fn use_trait<T: EnumGroup>(val: T) -> T::Group {
    val.into_group()
  }

  let msg = TraitImplMsg::OnlyVar(MsgA { value: 42 });
  let grouped = use_trait(msg);
  assert!(matches!(grouped, TraitImplMsgGroup::OnlyGroup(_)));

  // Also verify associated type
  let _: <TraitImplMsg as EnumGroup>::Group = TraitImplMsg::OnlyVar(MsgA { value: 1 }).into_group();
}

// =============================================================================
// Section F: Serde Integration
// =============================================================================

/// Test: Wire enum serializes correctly.
///
/// Verifies serialization produces expected JSON structure.
#[test]
fn test_serde_serialize() {
  use serde::{Deserialize, Serialize};

  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  struct SerdePayload {
    id: u32,
  }

  define_enum_group! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type", content = "payload")]
    enum SerializeMsg {
      Category {
        Item(SerdePayload),
      }
    }
  }

  let msg = SerializeMsg::Item(SerdePayload { id: 123 });
  let json = serde_json::to_string(&msg).expect("serialize failed");

  assert!(json.contains("\"type\":\"Item\""));
  assert!(json.contains("\"payload\""));
  assert!(json.contains("\"id\":123"));
}

/// Test: Wire enum deserializes correctly.
///
/// Verifies deserialization from JSON works.
#[test]
fn test_serde_deserialize() {
  use serde::{Deserialize, Serialize};

  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  struct DeserPayload {
    name: String,
  }

  define_enum_group! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type", content = "payload")]
    enum DeserializeMsg {
      Items {
        Thing(DeserPayload),
      }
    }
  }

  let json = r#"{"type":"Thing","payload":{"name":"test"}}"#;
  let msg: DeserializeMsg = serde_json::from_str(json).expect("deserialize failed");

  assert!(matches!(msg, DeserializeMsg::Thing(DeserPayload { name }) if name == "test"));
}

/// Test: Serialize then deserialize produces equal value.
///
/// Verifies roundtrip serialization works correctly.
#[test]
fn test_serde_roundtrip() {
  use serde::{Deserialize, Serialize};

  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  struct RoundtripPayload {
    value: i64,
    items: Vec<String>,
  }

  define_enum_group! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type", content = "payload")]
    enum RoundtripMsg {
      Data {
        Complex(RoundtripPayload),
      }
    }
  }

  let original = RoundtripMsg::Complex(RoundtripPayload {
    value: 999,
    items: vec!["a".to_string(), "b".to_string()],
  });

  let json = serde_json::to_string(&original).expect("serialize failed");
  let restored: RoundtripMsg = serde_json::from_str(&json).expect("deserialize failed");

  assert_eq!(original, restored);
}
