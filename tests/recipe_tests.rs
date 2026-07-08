use vimforge::ecs::recipes::{all_recipes, recipes_for, recipe_by_id};
use vimforge::resources::EntityType;

#[test]
fn test_all_recipes_not_empty() {
    let recipes = all_recipes();
    assert!(!recipes.is_empty(), "Should have at least one recipe");
}

#[test]
fn test_recipes_have_valid_fields() {
    for recipe in all_recipes() {
        assert!(!recipe.id.is_empty(), "Recipe ID should not be empty");
        assert!(recipe.ticks > 0, "Recipe ticks should be > 0");
        assert!(!recipe.outputs.is_empty(), "Recipe should have at least one output");
    }
}

#[test]
fn test_recipes_for_smelter() {
    let smelter_recipes = recipes_for(EntityType::Smelter);
    assert!(!smelter_recipes.is_empty(), "Smelter should have recipes");
    for recipe in &smelter_recipes {
        assert_eq!(recipe.building, EntityType::Smelter);
    }
}

#[test]
fn test_recipe_by_id_lookup() {
    let recipes = all_recipes();
    if let Some(first) = recipes.first() {
        let found = recipe_by_id(first.id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, first.id);
    }
}

#[test]
fn test_recipe_by_id_not_found() {
    let found = recipe_by_id("nonexistent_recipe_id_xyz");
    assert!(found.is_none());
}

#[test]
fn test_recipes_for_unknown_building() {
    // Conveyors don't process recipes
    let recipes = recipes_for(EntityType::BasicBelt);
    assert!(recipes.is_empty());
}

/// The tile simulation buffers at most two input items per machine
/// (Processing has two slots), so every recipe must need <= 2 items total.
#[test]
fn test_recipes_need_at_most_two_input_items() {
    for recipe in all_recipes() {
        assert!(
            recipe.flat_inputs().len() <= 2,
            "recipe {} needs {} input items; the simulation supports at most 2",
            recipe.id,
            recipe.flat_inputs().len()
        );
        assert!(
            !recipe.flat_inputs().is_empty(),
            "recipe {} has no inputs",
            recipe.id
        );
    }
}

/// The active recipe is re-derived from a machine's buffered inputs, so
/// within one building every recipe must have a UNIQUE input multiset.
#[test]
fn test_recipe_input_sets_unique_per_building() {
    let recipes = all_recipes();
    for a in &recipes {
        for b in &recipes {
            if a.id == b.id || a.building != b.building {
                continue;
            }
            let mut ai = a.flat_inputs();
            let mut bi = b.flat_inputs();
            ai.sort_by_key(|r| format!("{:?}", r));
            bi.sort_by_key(|r| format!("{:?}", r));
            assert_ne!(
                ai, bi,
                "recipes {} and {} on {:?} have identical input sets",
                a.id, b.id, a.building
            );
        }
    }
}

/// The legacy campaign chain must survive: iron ore smelts to ingot, and
/// two iron ingots assemble into a circuit board ("widget").
#[test]
fn test_legacy_campaign_recipes_present() {
    use vimforge::resources::Resource;
    let smelt = recipe_by_id("smelt_iron").expect("smelt_iron recipe");
    assert_eq!(smelt.ticks, 3, "smelter timing is a playthrough regression gate");
    assert_eq!(smelt.outputs[0].resource, Resource::IronIngot);

    let widget = recipe_by_id("assemble_widget").expect("assemble_widget recipe");
    assert_eq!(widget.ticks, 5, "assembler timing is a playthrough regression gate");
    assert_eq!(widget.outputs[0].resource, Resource::CircuitBoard);
    assert_eq!(
        widget.flat_inputs(),
        vec![Resource::IronIngot, Resource::IronIngot]
    );

    // Science pack 1 must be researchable from a cold start (ungated).
    let sp1 = recipe_by_id("science_pack_1").expect("science_pack_1 recipe");
    assert_eq!(sp1.numeric_id, 0, "science pack 1 must not be research-gated");
}

#[test]
fn test_recipe_inputs_outputs_consistent() {
    for recipe in all_recipes() {
        // Each recipe should have consistent building assignment
        let building_recipes = recipes_for(recipe.building);
        assert!(
            building_recipes.iter().any(|r| r.id == recipe.id),
            "Recipe {} should appear in recipes_for({:?})",
            recipe.id,
            recipe.building
        );
    }
}
