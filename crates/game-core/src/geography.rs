use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GridPoint {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RegionId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainKind {
    Plains,
    Forest,
    Hills,
    Mountain,
    River,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClimateBand {
    Boreal,
    Temperate,
    Subtropical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapCell {
    pub position: GridPoint,
    pub terrain: TerrainKind,
    pub climate: ClimateBand,
    pub elevation_meters: u16,
    pub soil_fertility: u8,
    pub water_access: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    pub id: RegionId,
    pub name: String,
    pub cells: Vec<GridPoint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldMap {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<MapCell>,
    pub regions: Vec<Region>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MapError {
    #[error("map dimensions must be greater than zero")]
    EmptyDimensions,
    #[error("map cell count does not match dimensions")]
    CellCountMismatch,
    #[error("map contains duplicate or out-of-bounds coordinates")]
    InvalidCoordinates,
    #[error("region references a cell outside the map")]
    InvalidRegionCell,
}

impl WorldMap {
    pub fn starter(seed: u64) -> Self {
        let width = 8;
        let height = 6;
        let mut random = ChaCha8Rng::seed_from_u64(seed ^ 0x5348_4552_4c4f_434b);
        let mut cells = Vec::with_capacity(usize::from(width) * usize::from(height));

        for y in 0..height {
            for x in 0..width {
                let position = GridPoint { x, y };
                let elevation = 120 + (random.next_u32() % 760) as u16;
                let fertility = 25 + (random.next_u32() % 76) as u8;
                let climate = match y {
                    0..=1 => ClimateBand::Boreal,
                    2..=3 => ClimateBand::Temperate,
                    _ => ClimateBand::Subtropical,
                };
                let terrain = if x == 3 {
                    TerrainKind::River
                } else if elevation >= 700 {
                    TerrainKind::Mountain
                } else if elevation >= 500 {
                    TerrainKind::Hills
                } else if random.next_u32() % 100 < 38 {
                    TerrainKind::Forest
                } else {
                    TerrainKind::Plains
                };
                let water_access = x.abs_diff(3) <= 1;
                cells.push(MapCell {
                    position,
                    terrain,
                    climate,
                    elevation_meters: elevation,
                    soil_fertility: fertility,
                    water_access,
                });
            }
        }

        let valley_cells = (0..height)
            .flat_map(|y| (2..=4).map(move |x| GridPoint { x, y }))
            .collect();
        let upland_cells = cells
            .iter()
            .filter(|cell| cell.position.x == 0 || cell.position.x >= 6)
            .map(|cell| cell.position)
            .collect();

        Self {
            width,
            height,
            cells,
            regions: vec![
                Region {
                    id: RegionId(1),
                    name: "Речная долина".into(),
                    cells: valley_cells,
                },
                Region {
                    id: RegionId(2),
                    name: "Восточные и западные возвышенности".into(),
                    cells: upland_cells,
                },
            ],
        }
    }

    pub fn cell_at(&self, position: GridPoint) -> Option<&MapCell> {
        if position.x >= self.width || position.y >= self.height {
            return None;
        }
        let index = usize::from(position.y) * usize::from(self.width)
            + usize::from(position.x);
        self.cells.get(index)
    }

    pub fn neighbors(&self, position: GridPoint) -> Vec<&MapCell> {
        let candidates = [
            position.x.checked_sub(1).map(|x| GridPoint { x, y: position.y }),
            position.x.checked_add(1).map(|x| GridPoint { x, y: position.y }),
            position.y.checked_sub(1).map(|y| GridPoint { x: position.x, y }),
            position.y.checked_add(1).map(|y| GridPoint { x: position.x, y }),
        ];
        candidates
            .into_iter()
            .flatten()
            .filter_map(|neighbor| self.cell_at(neighbor))
            .collect()
    }

    pub fn validate(&self) -> Result<(), MapError> {
        if self.width == 0 || self.height == 0 {
            return Err(MapError::EmptyDimensions);
        }
        let expected = usize::from(self.width) * usize::from(self.height);
        if self.cells.len() != expected {
            return Err(MapError::CellCountMismatch);
        }
        for (index, cell) in self.cells.iter().enumerate() {
            let expected_position = GridPoint {
                x: (index % usize::from(self.width)) as u16,
                y: (index / usize::from(self.width)) as u16,
            };
            if cell.position != expected_position {
                return Err(MapError::InvalidCoordinates);
            }
        }
        for region in &self.regions {
            if region.cells.iter().any(|position| self.cell_at(*position).is_none()) {
                return Err(MapError::InvalidRegionCell);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starter_map_is_deterministic_and_valid() {
        let first = WorldMap::starter(42);
        let second = WorldMap::starter(42);
        assert_eq!(first, second);
        assert_eq!(first.width, 8);
        assert_eq!(first.height, 6);
        assert_eq!(first.cells.len(), 48);
        assert_eq!(first.regions.len(), 2);
        assert!(first.validate().is_ok());
    }

    #[test]
    fn river_corridor_has_water_access_and_regions_cover_the_map() {
        let map = WorldMap::starter(7);
        for y in 0..map.height {
            let river = map.cell_at(GridPoint { x: 3, y }).expect("river cell");
            assert_eq!(river.terrain, TerrainKind::River);
            assert!(river.water_access);
        }
        let covered = map.regions.iter().map(|region| region.cells.len()).sum::<usize>();
        assert_eq!(covered, usize::from(map.width) * usize::from(map.height));
    }

    #[test]
    fn neighbors_respect_grid_edges() {
        let map = WorldMap::starter(11);
        assert_eq!(map.neighbors(GridPoint { x: 0, y: 0 }).len(), 2);
        assert_eq!(map.neighbors(GridPoint { x: 3, y: 2 }).len(), 4);
        assert!(map.cell_at(GridPoint { x: map.width, y: 0 }).is_none());
    }

    #[test]
    fn serialization_round_trip_preserves_geography() {
        let map = WorldMap::starter(99);
        let encoded = serde_json::to_string(&map).expect("serialize map");
        let decoded: WorldMap = serde_json::from_str(&encoded).expect("deserialize map");
        assert_eq!(decoded, map);
        assert!(decoded.validate().is_ok());
    }

    #[test]
    fn invalid_cell_order_is_rejected() {
        let mut map = WorldMap::starter(3);
        map.cells.swap(0, 1);
        assert_eq!(map.validate(), Err(MapError::InvalidCoordinates));
    }
}
