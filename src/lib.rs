use std::cmp::max;
use std::convert;
use std::iter::repeat;

extern crate unicode_width;
use unicode_width::UnicodeWidthStr;


pub struct Cell {
    pub contents: String,
    pub width: Width,
}

impl convert::From<String> for Cell {
    fn from(string: String) -> Self {
        Cell {
            width: UnicodeWidthStr::width(&*string),
            contents: string,
        }
    }
}

pub enum Direction {
    LeftToRight,
    TopToBottom,
}


pub type Width = usize;

pub struct GridOptions {
    pub direction:       Direction,
    pub separator_width: Width,
    pub maximum_width:   Width,
}

impl GridOptions {
    fn fit_into_grid(&self, cells: &[Cell]) -> Option<Dimensions> {
        // TODO: this function could almost certainly be optimised...
        // surely not *all* of the numbers of lines are worth searching through!

        let cell_count = cells.len();

        // Instead of numbers of columns, try to find the fewest number of *lines*
        // that the output will fit in.
        for num_lines in 1 .. cell_count {

            // The number of columns is the number of files divided by the number
            // of lines, *rounded up*.
            let mut num_columns = cell_count / num_lines;
            if cell_count % num_lines != 0 {
                num_columns += 1;
            }

            // Early abort: if there are so many columns that the width of the
            // *column separators* is bigger than the width of the screen, then
            // don't even try to tabulate it.
            // This is actually a necessary check, because the width is stored as
            // a usize, and making it go negative makes it huge instead, but it
            // also serves as a speed-up.
            let total_separator_width = (num_columns - 1) * self.separator_width;
            if self.maximum_width < total_separator_width {
                continue;
            }

            // Remove the separator width from the available space.
            let adjusted_width = self.maximum_width - total_separator_width;

            // Find the width of each column by adding the lengths of the file
            // names in that column up.
            let mut column_widths: Vec<Width> = repeat(0).take(num_columns).collect();
            for (index, cell) in cells.iter().enumerate() {
                let index = match self.direction {
                    Direction::LeftToRight  => index % num_columns,
                    Direction::TopToBottom  => index / num_lines,
                };
                column_widths[index] = max(column_widths[index], cell.width);
            }

            // If they all fit in the terminal, combined, then success!
            if sum(column_widths.iter().map(|&x| x)) < adjusted_width {
                return Some(Dimensions {
                    num_lines: num_lines,
                    widths: column_widths,
                });
            }
        }

        // If you get here you have really long file names.
        return None;
    }
}

pub struct Grid {
    options: GridOptions,
    cells: Vec<Cell>,
}

struct Dimensions {
    num_lines: Width,
    widths: Vec<Width>,
}

impl Grid {
    pub fn new(options: GridOptions) -> Grid {
        Grid {
            options: options,
            cells: Vec::new(),
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        self.cells.reserve(additional)
    }

    pub fn add(&mut self, cell: Cell) {
        self.cells.push(cell)
    }

    pub fn write(&self) {
        if let Some(dimensions) = self.options.fit_into_grid(&self.cells[..]) {
            for y in 0 .. dimensions.num_lines {
                for x in 0 .. dimensions.widths.len() {
                    let num = match self.options.direction {
                        Direction::LeftToRight   => y * dimensions.widths.len() + x,
                        Direction::TopToBottom   => y + dimensions.num_lines * x,
                    };

                    // Show whitespace in the place of trailing files
                    if num >= self.cells.len() {
                        continue;
                    }

                    let ref cell = self.cells[num];
                    if x == dimensions.widths.len() - 1 {
                        // The final column doesn't need to have trailing spaces
                        print!("{}", cell.contents);
                    }
                    else {
                        assert!(dimensions.widths[x] >= cell.width);
                        let extra_spaces = dimensions.widths[x] - cell.width + self.options.separator_width;
                        print!("{}", pad_string(&cell.contents, extra_spaces));
                    }
                }
                print!("\n");
            }
        }
        else {
            for cell in self.cells.iter() {
                println!("{}", cell.contents);
            }
        }
    }
}

/// Pad a string with the given number of spaces.
fn spaces(length: usize) -> String {
    repeat(" ").take(length).collect()
}

/// Pad a string with the given alignment and number of spaces.
///
/// This doesn't take the width the string *should* be, rather the number
/// of spaces to add: this is because the strings are usually full of
/// invisible control characters, so getting the displayed width of the
/// string is not as simple as just getting its length.
fn pad_string(string: &str, padding: usize) -> String {
    format!("{}{}", string, spaces(padding))
}

// TODO: This function can be replaced with `Iterator#sum` once the
// `iter_arith` feature flag cools down.
fn sum<I: Iterator<Item=Width>>(iterator: I) -> Width {
    iterator.fold(0, |s, e| s + e)
}