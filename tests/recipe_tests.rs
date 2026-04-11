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
