use vimforge::fluid::pipes::PipeGrid;
use vimforge::fluid::storage::TankState;
use vimforge::resources::Resource;

#[test]
fn test_pipe_grid_new() {
    let grid = PipeGrid::new();
    assert!(grid.pipes.is_empty());
}

#[test]
fn test_place_and_remove_pipe() {
    let mut grid = PipeGrid::new();
    grid.place_pipe(5, 5, 100);
    assert!(grid.pipes.contains_key(&(5, 5)));
    grid.remove_pipe(5, 5);
    assert!(!grid.pipes.contains_key(&(5, 5)));
}

#[test]
fn test_pipe_state_fill_fraction() {
    let mut grid = PipeGrid::new();
    grid.place_pipe(0, 0, 100);
    let pipe = grid.pipes.get(&(0, 0)).unwrap();
    assert_eq!(pipe.fill_fraction(), 0.0);
}

#[test]
fn test_tank_new() {
    let tank = TankState::new(1000, 10, 10);
    assert_eq!(tank.capacity, 1000);
    assert_eq!(tank.level, 0);
    assert!(tank.fluid_type.is_none());
}

#[test]
fn test_tank_fill_and_drain() {
    let mut tank = TankState::new(100, 50, 50);
    let filled = tank.try_fill(Resource::Water, 30);
    assert_eq!(filled, 30);
    assert_eq!(tank.level, 30);
    assert_eq!(tank.fluid_type, Some(Resource::Water));

    let drained = tank.try_drain(20);
    assert!(drained.is_some());
    let (resource, amount) = drained.unwrap();
    assert_eq!(resource, Resource::Water);
    assert_eq!(amount, 20);
    assert_eq!(tank.level, 10);
}

#[test]
fn test_tank_overfill() {
    let mut tank = TankState::new(50, 50, 50);
    let filled = tank.try_fill(Resource::Water, 100);
    assert_eq!(filled, 50); // capped at capacity
    assert_eq!(tank.level, 50);
}

#[test]
fn test_tank_drain_empty() {
    let mut tank = TankState::new(100, 10, 10);
    let drained = tank.try_drain(10);
    assert!(drained.is_none());
}

#[test]
fn test_tank_wrong_fluid() {
    let mut tank = TankState::new(100, 50, 50);
    tank.try_fill(Resource::Water, 30);
    let filled = tank.try_fill(Resource::CrudeOil, 20);
    assert_eq!(filled, 0); // can't mix fluids
}
