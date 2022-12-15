use pathfinding::prelude::*;

use super::*;

#[derive(Default, PartialEq, Eq)]
pub enum PathTile {
    #[default]
    NonWalkable,

    Walkable,
}

const STRAIGHT_COST: u32 = 10;
const DIAGONAL_COST: u32 = 14;

impl Terrain {
    pub fn generate_path_tiles(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                // Check if this node is walkable or not
                let tiles_above = [
                    self.layers[FRONT].get(x as isize, y as isize + 1),
                    self.layers[FRONT].get(x as isize, y as isize + 2),
                    self.layers[FRONT].get(x as isize, y as isize + 3),
                ];

                if self.layers[FRONT][(x, y)] != Tile::EMPTY
                    && (tiles_above[0].is_none() || tiles_above[0] == Some(&Tile::EMPTY))
                    && (tiles_above[1].is_none() || tiles_above[1] == Some(&Tile::EMPTY))
                    && (tiles_above[2].is_none() || tiles_above[2] == Some(&Tile::EMPTY))
                {
                    self.nodes[(x, y)] = PathTile::Walkable;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PathNode {
    pub x: u32,
    pub y: u32,
}

impl PathNode {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    // Euclid distance between 2 PathNodes
    pub fn distance(&self, other: &Self) -> u32 {
        f32::sqrt(
            ((self.x as isize - other.x as isize).pow(2)
                + (self.y as isize - other.y as isize).pow(2)) as f32,
        ) as u32
    }
}

impl Terrain {
    pub fn path_neighbours(&self, root: &PathNode) -> Vec<(PathNode, u32)> {
        let mut neighbours = Vec::new();

        for dx in [-1, 0, 1] {
            for dy in [-1, 0, 1] {
                // Skip the root node
                if dx == 0 && dy == 0 {
                    continue;
                }

                let (x, y) = (root.x as isize + dx, root.y as isize + dy);

                // Only count walkable neighbours
                if let Some(node) = self.nodes.get(x, y) && *node == PathTile::Walkable {
                    // Determine how expensive the move will be
                    let cost = if dy == 0 {
                        STRAIGHT_COST
                    } else {
                        DIAGONAL_COST
                    };

                    neighbours.push((PathNode::new(x as u32, y as u32), cost));
                }
            }
        }

        neighbours
    }

    pub fn find_path(&self, start: &PathNode, goal: &PathNode) -> Option<Vec<PathNode>> {
        let result = astar(
            start,
            |p| self.path_neighbours(p),
            |p| p.distance(goal),
            |p| *p == *goal,
        );

        result.map(|path| path.0)
    }
}
