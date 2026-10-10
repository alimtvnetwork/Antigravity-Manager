use super::*;
use crate::proxy::common::variant_mapping::GEMINI_FAMILIES;
use std::collections::{HashMap, HashSet};

#[test]
fn canonical_families_expose_the_complete_public_dto() {
    let families = get_canonical_families();

    assert_eq!(families.len(), GEMINI_FAMILIES.len());
    assert_eq!(
        families,
        vec![
            CanonicalFamilyDto {
                canonical_id: "gemini-3.7-flash".to_string(),
                display_name: "Gemini 3.7 Flash".to_string(),
                match_ids: vec![
                    "gemini-3.7-flash".to_string(),
                    "gemini-3.7-flash-high".to_string(),
                    "gemini-3.7-flash-medium".to_string(),
                    "gemini-3.7-flash-low".to_string(),
                    "gemini-3.7-flash-tiered".to_string(),
                    "gemini-3.6-flash-high".to_string(),
                    "gemini-3.6-flash-medium".to_string(),
                    "gemini-3.6-flash-low".to_string(),
                    "gemini-3.6-flash".to_string(),
                    "gemini-3.6-flash-tiered".to_string(),
                ],
            },
            CanonicalFamilyDto {
                canonical_id: "gemini-3.5-flash".to_string(),
                display_name: "Gemini 3.5 Flash".to_string(),
                match_ids: vec![
                    "gemini-3.5-flash".to_string(),
                    "gemini-3.5-flash-high".to_string(),
                    "gemini-3.5-flash-medium".to_string(),
                    "gemini-3.5-flash-low".to_string(),
                    "gemini-3-flash".to_string(),
                    "gemini-3.5-flash-extra-low".to_string(),
                    "gemini-3-flash-agent".to_string(),
                ],
            },
            CanonicalFamilyDto {
                canonical_id: "gemini-3.1-pro".to_string(),
                display_name: "Gemini 3.1 Pro".to_string(),
                match_ids: vec![
                    "gemini-3.1-pro".to_string(),
                    "gemini-3.1-pro-high".to_string(),
                    "gemini-pro".to_string(),
                    "gemini-3.1-pro-low".to_string(),
                    "gemini-pro-agent".to_string(),
                ],
            },
        ]
    );

    let serialized = serde_json::to_string(&families).unwrap();
    assert!(!serialized.contains("thinking_budget"));
    assert!(!serialized.contains("max_output_tokens"));
}

#[test]
fn canonical_families_match_ids_are_unique_globally_after_normalization() {
    let mut owners = HashMap::new();

    for family in get_canonical_families() {
        let mut family_match_ids = HashSet::new();
        for match_id in &family.match_ids {
            let normalized = match_id.to_lowercase();
            assert!(
                family_match_ids.insert(normalized.clone()),
                "{} contains duplicate match ID {}",
                family.canonical_id,
                match_id
            );
            assert!(
                owners
                    .insert(normalized.clone(), family.canonical_id.clone())
                    .is_none(),
                "{} belongs to multiple canonical families",
                normalized
            );
        }
    }
}
