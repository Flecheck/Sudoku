use bit_set::BitSet;
use keyed_priority_queue::{KeyedPriorityQueue,Entry};
use std::collections::HashMap;
use crate::sudoku::Sudoku;
use array_macro::array;

#[derive(Clone, Debug)]
enum Case {
    Val(usize),
    Possibilities(BitSet),
}

#[derive(Clone, Debug)]
struct Instance {
    sudoku: [[Case; 9]; 9],
    queue: KeyedPriorityQueue<(usize, usize), usize>,
    smart: usize,
    back: usize,
}

#[derive(Clone, Debug)]
pub struct Solver {
    instance: Instance,
    stack: Vec<Instance>,
}

impl Solver {
    pub fn new(sudoku: &Sudoku) -> Solver {
        let mut possibilities = array![array![Case::Possibilities(BitSet::from_bytes(&[0b0111_1111, 0b1100_0000]));9];9];

        let mut queue = KeyedPriorityQueue::new();

        for i in 0..9 {
            for j in 0..9 {
                let c = sudoku.cases[i][j];
                if c != 0 {
                    possibilities[i][j] = Case::Val(c);
                }
            }
        }

        for i in 0..9 {
            for j in 0..9 {
                match possibilities[i][j] {
                    Case::Val(_) => {}
                    Case::Possibilities(ref mut b) => {
                        let pos: BitSet = CONSTRAINTS[i][j]
                            .iter()
                            .filter_map(|&(i, j)| match sudoku.cases[i][j] {
                                0 => None,
                                x => Some(x),
                            }).collect();
                        queue.push((i, j), pos.len());
                        b.difference_with(&pos);
                    }
                }
            }
        }

        Solver {
            instance: Instance {
                sudoku: possibilities,
                queue: queue,
                smart: 0,
                back: 0,
            },
            stack: Vec::new(),
        }
    }

    fn update(&mut self, i: usize, j: usize, val: usize) {
        CONSTRAINTS[i][j]
            .iter()
            .for_each(|&(i, j)| match self.instance.sudoku[i][j] {
                Case::Val(_) => {}
                Case::Possibilities(ref mut b) => {
                    if b.remove(val) {
                        match self.instance.queue.entry((i,j)) {
                            Entry::Occupied(entry) => {
                                let priority = entry.get_priority() + 1;
                                entry.set_priority(priority)
                            },
                            Entry::Vacant(_) => unreachable!(),
                        };
                    };
                }
            });
    }

    pub fn solve(&mut self) {
        while let Some((&(i, j), &p)) = self.instance.queue.peek() {
            if p == 8 {
                self.instance.back = self.instance.smart;
                let c = match self.instance.sudoku[i][j] {
                    Case::Possibilities(ref mut b) => {
                        b.iter().next().expect("Queue and sudoku not in sync")
                    }
                    Case::Val(_) => panic!("Val in queue"),
                };
                self.instance.queue.pop();
                self.instance.sudoku[i][j] = Case::Val(c);
                self.update(i, j, c);
            } else if p == 9 {
                self.instance = self.stack.pop().expect("Sudoku impossible")
            } else if self.instance.smart - self.instance.back == 27 {
                let c = match self.instance.sudoku[i][j] {
                    Case::Possibilities(ref mut b) => {
                        let c = b.iter().next().expect("Queue and sudoku not in sync");
                        b.remove(c);
                        match self.instance.queue.entry((i,j)) {
                            Entry::Occupied(entry) => {
                                let priority = entry.get_priority() + 1;
                                entry.set_priority(priority)
                            },
                            Entry::Vacant(_) => unreachable!(),
                        };                       c
                    }
                    Case::Val(_) => panic!("Val in queue"),
                };

                self.stack.push(self.instance.clone());
                self.instance.queue.pop();
                self.instance.sudoku[i][j] = Case::Val(c);
                self.update(i, j, c);
            } else {
                self.smart()
            }
        }
    }

    pub fn into_sudoku(self) -> Sudoku {
        let mut sudoku: [[usize; 9]; 9] = [[0; 9]; 9];
        for (xc, line) in sudoku.iter_mut().enumerate() {
            for (yc, case) in line.iter_mut().enumerate() {
                *case = match self.instance.sudoku[xc][yc] {
                    Case::Val(c) => c,
                    Case::Possibilities(_) => 0,
                };
            }
        }
        Sudoku { cases: sudoku }
    }

    fn smart(&mut self) {
        let mut modif = Vec::new();
        let part = &PARTS[self.instance.smart % 27];
        {
            let map = part
                .iter()
                .map(|&(i, j)| ((i, j), &self.instance.sudoku[i][j]))
                .filter_map(|(i, x)| match x {
                    Case::Possibilities(x) => Some((i, x)),
                    _ => None,
                }).fold(HashMap::new(), |mut acc, (i, x)| {
                    for u in x {
                        acc.entry(u).or_insert_with(Vec::new).push(i);
                    }
                    acc
                });

            let pairs = map.into_iter().fold(HashMap::new(), |mut acc, (i, x)| {
                acc.entry(x).or_insert_with(Vec::new).push(i);
                acc
            });

            let mut todo: Vec<_> = pairs.into_iter().collect();
            let mut changed = true;
            while !todo.is_empty() && changed {
                changed = false;
                let (mut done, mut to_go): (Vec<_>, Vec<_>) =
                    todo.into_iter().partition(|(is, xs)| is.len() == xs.len());

                let d: Vec<usize> = done.iter().flat_map(|(_, xs)| xs.iter()).cloned().collect();

                for (_, xs) in to_go.iter_mut() {
                    *xs = xs
                        .iter()
                        .filter(|x| {
                            if !d.contains(x) {
                                changed = true;
                                false
                            } else {
                                true
                            }
                        }).cloned()
                        .collect();
                }

                todo = to_go;
                modif.append(&mut done);
            }
        }

        for (is, xs) in modif {
            let priority = 9 - xs.len();
            let bitset: BitSet = xs.into_iter().collect();
            for (i, j) in is {
                match &mut self.instance.sudoku[i][j] {
                    Case::Val(_) => panic!("Trying to modify fixed case"),
                    Case::Possibilities(b) => {
                        *b = bitset.clone();
                    }
                }
                self.instance.queue.set_priority(&(i, j), priority).unwrap();
            }
        }

        self.instance.smart += 1;
        if self.instance.back >= 27 {
            self.instance.back -= 27;
            self.instance.smart -= 27;
        }
    }
}

lazy_static! {
    static ref PARTS: Vec<Vec<(usize, usize)>> = parts();
}

fn parts() -> Vec<Vec<(usize, usize)>> {
    let lines = (0..9).map(|i| (0..9).map(|j| (i, j)).collect());
    let column = (0..9).map(|j| (0..9).map(|i| (i, j)).collect());
    let blocks = (0..3).flat_map(|i| {
        (0..3).map(move |j| {
            (0..3)
                .flat_map(|i2| (0..3).map(move |j2| (3 * i + i2, 3 * j + j2)))
                .collect()
        })
    });
    lines.chain(column).chain(blocks).collect()
}

lazy_static! {
    static ref CONSTRAINTS: [[Vec<(usize, usize)>; 9]; 9] = constraints();
}

fn constraints() -> [[Vec<(usize, usize)>; 9]; 9] {

    let mut constraints = array![array![Vec::new();9];9];

    for xc in 0..9 {
        for yc in 0..9 {
            let ref mut c = constraints[xc][yc];
            for x in 0..9 {
                if x != xc {
                    c.push((x, yc));
                }
            }

            for y in 0..9 {
                if y != yc {
                    c.push((xc, y));
                }
            }

            let gridx = (xc / 3) * 3;
            let gridy = (yc / 3) * 3;

            for x in gridx..gridx + 3 {
                for y in gridy..gridy + 3 {
                    if x != xc && y != yc {
                        c.push((x, y));
                    }
                }
            }
        }
    }
    constraints
}
