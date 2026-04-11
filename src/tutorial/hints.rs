use crate::levels::config::get_level;

/// Get a specific hint for a level by index.
/// Returns None if the level or hint index doesn't exist.
pub fn get_hint(level: usize, hint_index: usize) -> Option<&'static str> {
    let config = get_level(level)?;
    config.hints.get(hint_index).copied()
}

/// Get the objective string for a level.
/// Returns a fallback message if the level doesn't exist.
pub fn get_objective(level: usize) -> &'static str {
    match get_level(level) {
        Some(config) => config.objective,
        None => "Unknown level.",
    }
}

/// Get the display name of a level.
/// Returns a fallback if the level doesn't exist.
pub fn get_level_name(level: usize) -> &'static str {
    match get_level(level) {
        Some(config) => config.name,
        None => "Unknown",
    }
}
