extern crate term_grid;
use term_grid::{Grid, GridOptions, Direction};

fn main() {
    let mut grid = Grid::new(GridOptions {
        direction:        Direction::TopToBottom,
        separator_width:  2,
    });

    for i in 0..40 {
        grid.add(format!("{}", 2_isize.pow(i)).into())
    }

    println!("{}", grid.fit_into_width(40).unwrap());
}
