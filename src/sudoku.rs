use std::fmt;

#[derive(Debug, Clone)]
pub struct Sudoku {
    pub cases: [[usize; 9]; 9],
}

impl fmt::Display for Sudoku {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut i = self.cases.iter();
        if let Some(x) = i.next() {
            for c in x {
                write!(f, "{}", c)?;
            }

            for line in i {
                write!(f, "")?;
                for c in line {
                    write!(f, "{}", c)?;
                }
            }
        }
        Ok(())
    }
}
