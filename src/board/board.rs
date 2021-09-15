// Could use partial-min-max crate but they're all macros so no real efficiency to be gained
use min_max::*;
use rand::Rng;
use std::mem::discriminant;

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(unused)]
pub enum Cell {
    // First value is shark's energy, second is time to reproduce, third is advances
    Shark(usize, usize, usize),
    // Value is time to reproduction and second is advances
    Fish(usize, usize),
    Empty,
}

#[allow(unused)]
pub struct Board<const W: usize, const H: usize> {
    cells: [[Cell; W]; H],
    shark_energy: usize,
    fish_repo: usize,
    shark_repo: usize,
    shark_boost: usize,
    advances: usize,
    use_4_neighborhood: bool,
}

#[allow(unused)]
impl<const W: usize, const H: usize> Board<W, H> {
    pub fn new(
        fish_pct: f32,
        shark_pct: f32,
        shark_energy: usize,
        fish_repo: usize,
        shark_repo: usize,
        shark_boost: usize,
    ) -> Self {
        let mut cells = [[Cell::Empty; W]; H];
        let mut rng = rand::thread_rng();
        let safe_fish_pct = max_partial!(0.0, min_partial!(1.0, fish_pct));
        let safe_shark_pct = max_partial!(0.0, min_partial!(1.0, fish_pct + shark_pct));

        for col in 0..W {
            for row in 0..H {
                let sample = rng.gen_range(0.0..1.0);
                if sample <= safe_fish_pct {
                    cells[row][col] = Cell::Fish(0, 0);
                } else if sample <= safe_shark_pct {
                    cells[row][col] = Cell::Shark(shark_energy, 0, 0);
                }
            }
        }
        Self {
            cells,
            shark_energy,
            fish_repo,
            shark_repo,
            shark_boost,
            advances: 0,
            use_4_neighborhood: true,
        }
    }

    #[inline]
    fn set_cell(&mut self, row: usize, col: usize, cell: Cell) {
        self.cells[row][col] = cell;
    }

    #[inline]
    pub fn cell_at(&self, row: usize, col: usize) -> Cell {
        self.cells[row][col]
    }

    #[inline]
    fn is_fish(&self, row: usize, col: usize) -> bool {
        discriminant(&self.cells[row][col]) == discriminant(&Cell::Fish(0, 0))
    }

    #[inline]
    fn is_shark(&self, row: usize, col: usize) -> bool {
        discriminant(&self.cells[row][col]) == discriminant(&Cell::Shark(0, 0, 0))
    }

    fn count_fish(&self) -> i32 {
        let mut count = 0;
        for col in 0..W {
            for row in 0..H {
                if self.is_fish(row, col) {
                    count += 1;
                }
            }
        }
        count
    }

    fn count_sharks(&self) -> i32 {
        let mut count = 0;
        for col in 0..W {
            for row in 0..H {
                if self.is_shark(row, col) {
                    count += 1;
                }
            }
        }
        count
    }

    fn find_empty_neighbor(&self, row: usize, col: usize) -> Option<(usize, usize)> {
        let mut count = 0usize;
        let mut candidate_rows = [0; 8];
        let mut candidate_cols = [0; 8];

        for col_neighbor_safe in W + col - 1..=W + col + 1 {
            for row_neighbor_safe in H + row - 1..=H + row + 1 {
                let col_neighbor = col_neighbor_safe % W;
                let row_neighbor = row_neighbor_safe % H;
                if (self.use_4_neighborhood && col_neighbor != col && row_neighbor != row) {
                    // Don't use diagonals
                    continue;
                }

                if (row_neighbor, col_neighbor) == (row, col) {
                    continue;
                }
                if self.cells[row_neighbor as usize][col_neighbor as usize] != Cell::Empty {
                    continue;
                }
                candidate_rows[count] = row_neighbor;
                candidate_cols[count] = col_neighbor;
                count += 1;
            }
        }

        if (count == 0) {
            return None;
        }

        let index = rand::thread_rng().gen_range(0..count);
        Some((candidate_rows[index], candidate_cols[index]))
    }

    fn find_shark_neighbor(&self, row: usize, col: usize) -> Option<(usize, usize)> {
        let mut count = 0usize;
        let mut fish_count = 0usize;
        let mut candidate_rows = [0; 8];
        let mut candidate_cols = [0; 8];
        let mut fish_rows = [0; 8];
        let mut fish_cols = [0; 8];

        for col_neighbor_safe in W + col - 1..=W + col + 1 {
            for row_neighbor_safe in H + row - 1..=H + row + 1 {
                let col_neighbor = col_neighbor_safe % W;
                let row_neighbor = row_neighbor_safe % H;
                if (self.use_4_neighborhood && col_neighbor != col && row_neighbor != row) {
                    // Don't use diagonals
                    continue;
                }

                if (row_neighbor, col_neighbor) == (row, col) {
                    continue;
                }
                if self.is_fish(row_neighbor, col_neighbor) {
                    fish_rows[fish_count] = row_neighbor;
                    fish_cols[fish_count] = col_neighbor;
                    fish_count += 1;
                } else if fish_count == 0 && self.cells[row_neighbor][col_neighbor] == Cell::Empty {
                    candidate_rows[count] = row_neighbor;
                    candidate_cols[count] = col_neighbor;
                    count += 1;
                }
            }
        }

        if (count == 0 && fish_count == 0) {
            return None;
        }

        if (fish_count != 0) {
            let index = rand::thread_rng().gen_range(0..fish_count);
            Some((fish_rows[index], fish_cols[index]))
        } else {
            let index = rand::thread_rng().gen_range(0..count);
            Some((candidate_rows[index], candidate_cols[index]))
        }
    }

    pub fn advance(&mut self) {
        let mut rng = rand::thread_rng();

        // Count frames so we know who belongs to what frame
        self.advances += 1;

        for col in 0..W {
            for row in 0..H {
                match self.cells[row][col] {
                    Cell::Empty => continue,
                    Cell::Fish(r, a) => {
                        if a == self.advances {
                            // Looking at a fish we've already moved this turn
                            continue;
                        }
                        let check = self.find_empty_neighbor(row, col);
                        if (check != None) {
                            // Our parent fish moves
                            let (new_row, new_col) = check.unwrap();
                            self.cells[new_row][new_col] =
                                Cell::Fish((r + 1) % self.fish_repo, a + 1);

                            if r == self.fish_repo - 1 {
                                // Baby is left behind at original position
                                self.cells[row][col] = Cell::Fish(0, a + 1);
                            } else {
                                self.cells[row][col] = Cell::Empty;
                            }
                        } else {
                            self.cells[row][col] = Cell::Fish((r + 1) % self.fish_repo, a + 1);
                        }
                    }
                    Cell::Shark(e, r, a) => {
                        if a == self.advances {
                            // Looking at a shark we've already moved this turn
                            continue;
                        }
                        let check = self.find_shark_neighbor(row, col);
                        if (check != None) {
                            let (new_row, new_col) = check.unwrap();
                            let eating = self.is_fish(new_row, new_col);
                            if (!eating && e == 1) {
                                // e == 1 means it goes to zero this turn so he's dead
                                // RIP Mr. Shark!
                                self.cells[row][col] = Cell::Empty;
                                continue;
                            }
                            // Shark moves
                            let new_energy = if eating {
                                // Yum!  New lease on life!
                                e + self.shark_boost
                            } else {
                                // Moving inexorably toward death
                                e - 1
                            };
                            let new_repo = (r + 1) % self.shark_repo;
                            self.cells[new_row][new_col] = Cell::Shark(new_energy, new_repo, a + 1);
                            if (new_repo == 0) {
                                // Baby is left behind
                                let new_repo = rand::thread_rng().gen_range(0..self.shark_repo);
                                //let new_repo = 0;
                                self.cells[row][col] =
                                    Cell::Shark(self.shark_energy, new_repo, a + 1);
                            } else {
                                self.cells[row][col] = Cell::Empty;
                            }
                        } else {
                            if (e == 1) {
                                // Alas!
                                self.cells[row][col] = Cell::Empty;
                                continue;
                            }
                            let new_energy = e - 1;
                            let new_repo = (r + 1) % self.shark_repo;
                            self.cells[row][col] = Cell::Shark(new_energy, new_repo, a + 1)
                        }
                    }
                }
            }
        }
    }

    pub fn print_board(&self, text: &str) {
        println!("{}{}", "Board Contents: ", text);
        for row in 0..H {
            print!("Row {}: ", row);
            for col in 0..W {
                print!("{:?}, ", self.cell_at(row, col));
            }
            println!("");
        }
    }
}

#[test]
pub fn test_board_creation() {
    let board = Board::<10, 10>::new(0.5, 0.5, 5, 3, 5, 3);
    for row in 0..10 {
        for col in 0..10 {
            assert!(board.cell_at(row, col) != Cell::Empty);
        }
    }
    let board = Board::<10, 10>::new(0.0, 1.0, 5, 3, 5, 3);
    for row in 0..10 {
        for col in 0..10 {
            assert!(board.is_shark(row, col));
        }
    }

    let board = Board::<10, 10>::new(1.0, 0.0, 5, 3, 5, 3);
    for row in 0..10 {
        for col in 0..10 {
            assert!(board.is_fish(row, col));
        }
    }

    let board = Board::<10, 10>::new(0.0, 0.0, 5, 3, 5, 3);
    for row in 0..10 {
        for col in 0..10 {
            assert!(board.cell_at(row, col) == Cell::Empty);
        }
    }
}

#[test]
fn test_fish() {
    let mut board = Board::<2, 2>::new(0.0, 0.0, 5, 2, 5, 5);
    board.set_cell(0, 0, Cell::Fish(0, 0));
    assert!(board.is_fish(0, 0));
    assert_eq!(1, board.count_fish());
    board.advance();
    assert!(!board.is_fish(0, 0));
    assert_eq!(1, board.count_fish());
    board.advance();
    assert_eq!(2, board.count_fish());
    board.advance();
    board.advance();
    assert_eq!(4, board.count_fish());
    board.advance();
    board.advance();
    assert_eq!(4, board.count_fish());
    board.advance();
    board.advance();
    assert_eq!(4, board.count_fish());
}

#[test]
fn test_sharks() {
    let mut board = Board::<2, 2>::new(0.0, 0.0, 2, 2, 2, 2);
    board.set_cell(0, 0, Cell::Fish(2, 0));
    board.set_cell(0, 1, Cell::Shark(2, 2, 0));
    board.advance();
    // Shark should eat the fish
    assert_eq!(0, board.count_fish());
    assert_eq!(1, board.count_sharks());

    let mut board = Board::<2, 2>::new(0.0, 0.0, 2, 2, 3, 2);
    board.set_cell(0, 0, Cell::Fish(3, 0));
    board.set_cell(1, 0, Cell::Fish(3, 0));
    board.set_cell(0, 1, Cell::Shark(2, 0, 0));
    board.advance();
    // One fish down
    assert_eq!(1, board.count_fish());
    assert_eq!(1, board.count_sharks());
    board.advance();
    // ...and the second
    assert_eq!(0, board.count_fish());
    assert_eq!(1, board.count_sharks());
    board.advance();
    // New baby shark!
    assert_eq!(2, board.count_sharks());
    board.advance();
    assert_eq!(2, board.count_sharks());
    // Four more and daddy shark dies
    board.advance();
    // Baby shark dies
    assert_eq!(1, board.count_sharks());
    board.advance();
    board.advance();
    // Should be barely alive
    assert_eq!(2, board.count_sharks());
    board.advance();
    // That should have done him in
    assert_eq!(0, board.count_sharks());
}
