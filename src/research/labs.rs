use serde::{Deserialize, Serialize};

use crate::resources::{EntityType, Resource};
use super::tree::ResearchState;

// ---------------------------------------------------------------------------
// Lab Specification
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LabSpec {
    pub entity_type: EntityType,
    pub accepted_packs: Vec<Resource>,
    pub speed_multiplier: f64,
}

/// Get the lab specification for an entity type, if it is a lab building.
pub fn get_lab_spec(entity_type: EntityType) -> Option<LabSpec> {
    match entity_type {
        EntityType::ResearchLab => Some(LabSpec {
            entity_type: EntityType::ResearchLab,
            accepted_packs: vec![Resource::SciencePack1],
            speed_multiplier: 1.0,
        }),
        EntityType::AdvancedLab => Some(LabSpec {
            entity_type: EntityType::AdvancedLab,
            accepted_packs: vec![Resource::SciencePack1, Resource::SciencePack2],
            speed_multiplier: 1.5,
        }),
        EntityType::QuantumLab => Some(LabSpec {
            entity_type: EntityType::QuantumLab,
            accepted_packs: vec![
                Resource::SciencePack1,
                Resource::SciencePack2,
                Resource::SciencePack3,
            ],
            speed_multiplier: 2.0,
        }),
        EntityType::SingularityLab => Some(LabSpec {
            entity_type: EntityType::SingularityLab,
            accepted_packs: vec![
                Resource::SciencePack1,
                Resource::SciencePack2,
                Resource::SciencePack3,
                Resource::SciencePack4,
                Resource::SciencePack5,
            ],
            speed_multiplier: 3.0,
        }),
        _ => None,
    }
}

/// Returns all lab entity types in order of capability.
pub fn all_lab_types() -> Vec<EntityType> {
    vec![
        EntityType::ResearchLab,
        EntityType::AdvancedLab,
        EntityType::QuantumLab,
        EntityType::SingularityLab,
    ]
}

/// Check whether a lab can contribute to the current research.
/// A lab can contribute if it accepts all science pack types required by the tech.
pub fn can_lab_research(lab_spec: &LabSpec, required_packs: &[(Resource, u64)]) -> bool {
    required_packs
        .iter()
        .all(|(res, _)| lab_spec.accepted_packs.contains(res))
}

/// Calculate the research ticks contributed by a single lab per game tick.
/// Returns the effective ticks of research produced (accounting for speed multiplier).
/// The lab must have the required science packs available in its input inventory.
///
/// `available_packs` maps resource -> available count in the lab's input.
/// `required_packs` is from the current Technology's science_cost.
///
/// Returns (ticks_contributed, packs_consumed) where packs_consumed lists
/// (Resource, amount) pairs that should be deducted from the lab inventory.
pub fn compute_lab_contribution(
    lab_spec: &LabSpec,
    available_packs: &[(Resource, u64)],
    required_packs: &[(Resource, u64)],
) -> (u32, Vec<(Resource, u64)>) {
    if !can_lab_research(lab_spec, required_packs) {
        return (0, vec![]);
    }

    // Check that the lab has at least 1 of each required pack type
    let has_all = required_packs.iter().all(|(res, _)| {
        available_packs
            .iter()
            .any(|(r, count)| r == res && *count > 0)
    });

    if !has_all {
        return (0, vec![]);
    }

    // Consume 1 of each required pack type per tick
    let consumed: Vec<(Resource, u64)> = required_packs
        .iter()
        .map(|(res, _)| (*res, 1))
        .collect();

    let ticks = lab_spec.speed_multiplier as u32;
    (ticks.max(1), consumed)
}

/// Update research state given active labs.
///
/// `lab_inventories` is a list of (LabSpec, available_packs) for each active lab.
/// Returns the total research ticks to advance.
pub fn update_research(
    lab_inventories: &[(LabSpec, Vec<(Resource, u64)>)],
    research_state: &ResearchState,
) -> (u32, Vec<Vec<(Resource, u64)>>) {
    let current_id = match research_state.current {
        Some(id) => id,
        None => return (0, vec![]),
    };

    let tech = super::tree::get_tech(current_id);
    let mut total_ticks = 0u32;
    let mut all_consumed = Vec::new();

    for (lab_spec, available) in lab_inventories {
        let (ticks, consumed) = compute_lab_contribution(lab_spec, available, &tech.science_cost);
        total_ticks += ticks;
        all_consumed.push(consumed);
    }

    (total_ticks, all_consumed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_lab_accepts_sp1_only() {
        let spec = get_lab_spec(EntityType::ResearchLab).unwrap();
        assert_eq!(spec.accepted_packs.len(), 1);
        assert_eq!(spec.accepted_packs[0], Resource::SciencePack1);
        assert!((spec.speed_multiplier - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn advanced_lab_accepts_sp1_sp2() {
        let spec = get_lab_spec(EntityType::AdvancedLab).unwrap();
        assert_eq!(spec.accepted_packs.len(), 2);
        assert!((spec.speed_multiplier - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn quantum_lab_accepts_sp1_sp2_sp3() {
        let spec = get_lab_spec(EntityType::QuantumLab).unwrap();
        assert_eq!(spec.accepted_packs.len(), 3);
        assert!((spec.speed_multiplier - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn singularity_lab_accepts_all() {
        let spec = get_lab_spec(EntityType::SingularityLab).unwrap();
        assert_eq!(spec.accepted_packs.len(), 5);
        assert!((spec.speed_multiplier - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn non_lab_returns_none() {
        assert!(get_lab_spec(EntityType::Smelter).is_none());
        assert!(get_lab_spec(EntityType::Assembler).is_none());
    }

    #[test]
    fn lab_cannot_research_missing_pack_type() {
        let basic = get_lab_spec(EntityType::ResearchLab).unwrap();
        // Research requiring SP1 + SP2 should fail for basic lab
        let required = vec![(Resource::SciencePack1, 10), (Resource::SciencePack2, 10)];
        assert!(!can_lab_research(&basic, &required));
    }

    #[test]
    fn lab_can_research_matching_packs() {
        let advanced = get_lab_spec(EntityType::AdvancedLab).unwrap();
        let required = vec![(Resource::SciencePack1, 10), (Resource::SciencePack2, 10)];
        assert!(can_lab_research(&advanced, &required));
    }

    #[test]
    fn compute_contribution_with_available_packs() {
        let spec = get_lab_spec(EntityType::ResearchLab).unwrap();
        let available = vec![(Resource::SciencePack1, 5)];
        let required = vec![(Resource::SciencePack1, 10)];
        let (ticks, consumed) = compute_lab_contribution(&spec, &available, &required);
        assert_eq!(ticks, 1);
        assert_eq!(consumed.len(), 1);
        assert_eq!(consumed[0], (Resource::SciencePack1, 1));
    }

    #[test]
    fn compute_contribution_without_packs() {
        let spec = get_lab_spec(EntityType::ResearchLab).unwrap();
        let available = vec![(Resource::SciencePack1, 0)];
        let required = vec![(Resource::SciencePack1, 10)];
        let (ticks, consumed) = compute_lab_contribution(&spec, &available, &required);
        assert_eq!(ticks, 0);
        assert!(consumed.is_empty());
    }
}
