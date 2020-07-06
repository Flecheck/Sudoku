mod solver;
mod sudoku;

extern crate bit_set;
extern crate priority_queue;
#[macro_use]
extern crate lazy_static;
extern crate itertools;

use std::io;
use std::io::BufRead;
use std::io::Write;




use crate::solver::Solver;
use crate::sudoku::Sudoku;

use itertools::Itertools;

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let lines = stdin.lock().lines();

    lines.for_each(|line| {
        let mut sudoku: [[usize; 9]; 9] = [[0; 9]; 9];
        let a = line.unwrap();
        let l = a.chars().chunks(9);
        for (x, l) in l.into_iter().enumerate() {
            for (y, c) in l.enumerate() {
                sudoku[x][y] = c.to_digit(10).unwrap_or(0) as usize;
            }
        }
        let sudoku = Sudoku { cases: sudoku };
        let mut solver = Solver::new(&sudoku);
        solver.solve();
        let res = solver.into_sudoku();
        writeln!(stdout, "{}", res);
    });
}
