extern crate term_grid;
use term_grid::{Grid, GridOptions, Direction, Filling};

fn main() {
    let mut grid = Grid::new(GridOptions {
        direction:  Direction::TopToBottom,
        filling:    Filling::Spaces(2),
    });

    for i in 0..40 {
        grid.add(format!("{}", 2_isize.pow(i)).into())
    }

    if let Some(grid_display) = grid.fit_into_width(40) {
        println!("{}", grid_display);
    }
    else {
        println!("Couldn't fit grid into 40 columns!");
    }
}
