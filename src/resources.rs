use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resource {
    Ore,
    Ingot,
    Widget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    OreDeposit,
    Smelter,
    Assembler,
    Conveyor,
    Splitter,
    Merger,
    OutputBin,
    Wall,
}

impl EntityType {
    pub fn from_insert_char(c: char) -> Option<EntityType> {
        match c {
            's' => Some(EntityType::Smelter),
            'a' => Some(EntityType::Assembler),
            'c' => Some(EntityType::Conveyor),
            'p' => Some(EntityType::Splitter),
            'e' => Some(EntityType::Merger),
            'w' => Some(EntityType::Wall),
            _ => None,
        }
    }

    pub fn from_find_char(c: char) -> Option<EntityType> {
        match c {
            's' => Some(EntityType::Smelter),
            'a' => Some(EntityType::Assembler),
            'c' => Some(EntityType::Conveyor),
            'p' => Some(EntityType::Splitter),
            'm' => Some(EntityType::Merger),
            'o' => Some(EntityType::OreDeposit),
            'b' => Some(EntityType::OutputBin),
            'w' => Some(EntityType::Wall),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            EntityType::OreDeposit => "ore deposit",
            EntityType::Smelter => "smelter",
            EntityType::Assembler => "assembler",
            EntityType::Conveyor => "conveyor",
            EntityType::Splitter => "splitter",
            EntityType::Merger => "merger",
            EntityType::OutputBin => "output bin",
            EntityType::Wall => "wall",
        }
    }

    pub fn from_search_prefix(prefix: &str) -> Option<EntityType> {
        let lower = prefix.to_lowercase();
        if "ore deposit".starts_with(&lower) || "ore".starts_with(&lower) {
            Some(EntityType::OreDeposit)
        } else if "smelter".starts_with(&lower) || "sme".starts_with(&lower) {
            Some(EntityType::Smelter)
        } else if "assembler".starts_with(&lower) || "ass".starts_with(&lower) {
            Some(EntityType::Assembler)
        } else if "conveyor".starts_with(&lower) || "con".starts_with(&lower) {
            Some(EntityType::Conveyor)
        } else if "splitter".starts_with(&lower) || "spl".starts_with(&lower) {
            Some(EntityType::Splitter)
        } else if "merger".starts_with(&lower) || "mer".starts_with(&lower) {
            Some(EntityType::Merger)
        } else if "output bin".starts_with(&lower) || "bin".starts_with(&lower) {
            Some(EntityType::OutputBin)
        } else if "wall".starts_with(&lower) {
            Some(EntityType::Wall)
        } else {
            None
        }
    }

    pub fn is_player_placeable(&self) -> bool {
        matches!(
            self,
            EntityType::Smelter
                | EntityType::Assembler
                | EntityType::Conveyor
                | EntityType::Splitter
                | EntityType::Merger
                | EntityType::Wall
        )
    }

    pub fn has_facing(&self) -> bool {
        matches!(
            self,
            EntityType::Smelter
                | EntityType::Assembler
                | EntityType::Conveyor
                | EntityType::Splitter
                | EntityType::Merger
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Facing {
    Up,
    Right,
    Down,
    Left,
}

impl Facing {
    pub fn opposite(&self) -> Facing {
        match self {
            Facing::Up => Facing::Down,
            Facing::Down => Facing::Up,
            Facing::Left => Facing::Right,
            Facing::Right => Facing::Left,
        }
    }

    pub fn rotate_cw(&self) -> Facing {
        match self {
            Facing::Up => Facing::Right,
            Facing::Right => Facing::Down,
            Facing::Down => Facing::Left,
            Facing::Left => Facing::Up,
        }
    }

    pub fn rotate_ccw(&self) -> Facing {
        match self {
            Facing::Up => Facing::Left,
            Facing::Left => Facing::Down,
            Facing::Down => Facing::Right,
            Facing::Right => Facing::Up,
        }
    }

    pub fn offset(&self) -> (isize, isize) {
        match self {
            Facing::Up => (0, -1),
            Facing::Down => (0, 1),
            Facing::Left => (-1, 0),
            Facing::Right => (1, 0),
        }
    }

    pub fn perpendicular(&self) -> (Facing, Facing) {
        match self {
            Facing::Up | Facing::Down => (Facing::Left, Facing::Right),
            Facing::Left | Facing::Right => (Facing::Up, Facing::Down),
        }
    }

    pub fn arrow_glyph(&self) -> char {
        match self {
            Facing::Up => '↑',
            Facing::Down => '↓',
            Facing::Left => '←',
            Facing::Right => '→',
        }
    }

    pub fn all() -> [Facing; 4] {
        [Facing::Up, Facing::Right, Facing::Down, Facing::Left]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn to_facing(&self) -> Facing {
        match self {
            Direction::Up => Facing::Up,
            Direction::Down => Facing::Down,
            Direction::Left => Facing::Left,
            Direction::Right => Facing::Right,
        }
    }
}

impl From<Facing> for Direction {
    fn from(f: Facing) -> Self {
        match f {
            Facing::Up => Direction::Up,
            Facing::Down => Direction::Down,
            Facing::Left => Direction::Left,
            Facing::Right => Direction::Right,
        }
    }
}

pub fn get_input_sides(entity_type: EntityType, facing: Facing) -> Vec<Facing> {
    match entity_type {
        EntityType::OreDeposit => vec![],
        EntityType::Smelter => vec![facing.opposite()],
        EntityType::Assembler => {
            let (a, b) = facing.perpendicular();
            vec![a, b]
        }
        EntityType::Conveyor => {
            let opp = facing.opposite();
            let (a, b) = facing.perpendicular();
            vec![opp, a, b]
        }
        EntityType::Splitter => vec![facing.opposite()],
        EntityType::Merger => {
            let (a, b) = facing.perpendicular();
            vec![a, b]
        }
        EntityType::OutputBin => Facing::all().to_vec(),
        EntityType::Wall => vec![],
    }
}

pub fn get_output_sides(entity_type: EntityType, facing: Facing) -> Vec<Facing> {
    match entity_type {
        EntityType::OreDeposit => Facing::all().to_vec(),
        EntityType::Smelter => vec![facing],
        EntityType::Assembler => vec![facing],
        EntityType::Conveyor => vec![facing],
        EntityType::Splitter => {
            let (a, b) = facing.perpendicular();
            vec![a, b]
        }
        EntityType::Merger => vec![facing],
        EntityType::OutputBin => vec![],
        EntityType::Wall => vec![],
    }
}
