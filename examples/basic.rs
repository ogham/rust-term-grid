extern crate term_grid;
use term_grid::{Grid, GridOptions, Direction};

fn main() {
    let mut grid = Grid::new(GridOptions {
        direction:        Direction::LeftToRight,
        maximum_width:    40,
        separator_width:  2,
    });

    for i in 0..25 {
        grid.add(format!("{}", 2_isize.pow(i)).into())
    }

    grid.write();
}
