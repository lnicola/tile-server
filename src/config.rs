use crate::tile_grid::TileGrid;

#[derive(Clone)]
pub struct Config {
    pub tile_grid: TileGrid,
    pub reverse_y: bool,
    pub tile_width: usize,
    pub tile_height: usize,
}
