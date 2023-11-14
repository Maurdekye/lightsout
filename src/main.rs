use rand::prelude::*;
use std::{collections::{BinaryHeap, HashSet}, fmt::Display, time::SystemTime};

trait Search: Clone + std::hash::Hash + Eq + PartialEq + Display {
    type Score: Ord + Display;

    fn score(&self) -> Self::Score;
    fn end(&self) -> bool;
    fn moves(&self) -> Vec<(Self, usize)>;
}

type Rowtype = u64;

#[derive(Clone)]
struct Board {
    width: usize,
    height: usize,
    rows: Vec<Rowtype>,
}

impl Board {
    fn new(width: usize, height: usize) -> Board {
        Board {
            width: width,
            height: height,
            rows: (0..height).map(|_| 0).collect(),
        }
    }

    fn get(&self, x: usize, y: usize) -> bool {
        self.rows[y] & (1 << x) != 0
    }

    fn set(&mut self, x: usize, y: usize, value: bool) {
        self.rows[y] = (self.rows[y] & !(1 << x)) | (Into::<Rowtype>::into(value) << x);
    }

    fn randomize(&mut self, seed: u64) {
        let mut rng: StdRng = SeedableRng::seed_from_u64(seed);
        for y in 0..self.height {
            self.rows[y] = rng.gen::<Rowtype>() % (1 << self.width);
        }
    }

    fn clone_toggle(&self, x: usize, y: usize) -> Board {
        let mut new_board = self.clone();
        if y > 0 {
            new_board.rows[y - 1] ^= 1 << x;
        }
        new_board.rows[y] ^= ((7 << x) >> 1) & ((1 << self.width + 1) - 1);
        if y < self.height - 1 {
            new_board.rows[y + 1] ^= 1 << x;
        }
        new_board
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.rows.iter() {
            writeln!(
                f,
                "{}",
                (0..self.width)
                    .map(|x| if row & (1 << x) == 0 {"░░"} else {"██"})
                    // .map(|x| if row & (1 << x) == 0 { ".." } else { "##" })
                    .collect::<Vec<_>>()
                    .join("")
            )?;
        }
        Ok(())
    }
}

impl std::hash::Hash for Board {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.rows.hash(state);
    }
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows
    }
}

impl Eq for Board {
    
}

impl Search for Board {
    type Score = usize;

    fn score(&self) -> usize {
        self.rows
            .iter()
            .map(|row| {
                (0..self.width)
                    .map(|x| row & (1 << x) == 0)
                    .map(|lit| Into::<usize>::into(lit))
                    .sum::<usize>()
            })
            .sum()
    }

    fn end(&self) -> bool {
        self.score() == self.width * self.height
    }

    fn moves(&self) -> Vec<(Self, usize)> {
        let init_score = self.score();
        (0..self.width)
            .map(|x| {
                (0..self.height)
                    .map(|y| (self.clone_toggle(x, y), x, y))
                    .filter(|(board, x, y)| {
                        let mut target_score = 3;
                        if *x > 0 && *x < board.width - 1 {
                            target_score += 1;
                        }
                        if *y > 0 && *y < board.height - 1 {
                            target_score += 1;
                        }
                        board.score() != init_score - target_score
                    })
                    .map(|(board, x, y)| (board, x * self.width + y))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }
}

#[derive(Debug)]
struct SearchState<T: Search> {
    history: Vec<usize>,
    latest: T,
    latest_move_index: Option<usize>,
    score: T::Score,
}

impl<T: Search> SearchState<T> {
    fn moves(&self) -> Vec<Self> {
        let mut new_history = self.history.clone();
        match self.latest_move_index {
            Some(index) => new_history.push(index),
            None => ()
        };
        self.latest
            .moves()
            .into_iter()
            .map(|(new_move, move_index)| {
                let mut new_state: SearchState<T> = new_move.into();
                new_state.history = new_history.clone();
                new_state.latest_move_index = Some(move_index);
                new_state
            })
            .collect()
    }
}

impl<T: Search> From<T> for SearchState<T> {
    fn from(value: T) -> Self {
        let score = value.score();
        SearchState {
            history: Vec::new(),
            latest: value,
            latest_move_index: None,
            score: score,
        }
    }
}

impl<T: Search> PartialEq for SearchState<T> {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl<T: Search> PartialOrd for SearchState<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl<T: Search> Ord for SearchState<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score)
    }
}

impl<T: Search> Eq for SearchState<T> {}

fn a_star<T: Search>(init_state: T, max_depth: usize) -> (Option<SearchState<T>>, usize) {
    let mut explored: HashSet<T> = HashSet::new();
    let mut fringe: BinaryHeap<SearchState<T>> = BinaryHeap::new();
    fringe.push(init_state.into());
    ((|| {
        loop {
            // if explored.len() > 100_000 {
            //     return None;
            // }
            // println!("queue: {}", fringe.len());
            match fringe.pop() {
                None => return None,
                Some(state) => {
                    // println!(
                    //     "history: {}, score: {}\n{}",
                    //     state.history.len(),
                    //     state.score,
                    //     state.latest
                    // );
                    if state.latest.end() {
                        return Some(state);
                    } else if state.history.len() < max_depth {
                        explored.insert(state.latest.clone());
                        for next_state in state.moves() {
                            if !explored.contains(&next_state.latest) {
                                fringe.push(next_state);
                            }
                        }
                    }
                }
            }
        }
    })(), explored.len())
}

fn main() {
    let mut init_board = Board::new(5, 5);
    // for y in 0..init_board.height - 3 {
    //     init_board.rows[y] = (1 << init_board.width + 1) - 1;
    // }
    // for mv in init_board.moves() {
    //     println!("{mv}");
    // }
    let seed = random();
    println!("Seed: {seed}");
    init_board.randomize(seed);
    println!("{init_board}");
    let mut board = init_board.clone();

    let start = SystemTime::now();
    let (result, n_explored) = a_star(init_board, board.width * board.height);
    match result {
        None => println!("No solution :("),
        Some(soln) => {
            let s_len = soln.history.len();
            println!("Solution:");
            for move_id in soln.history {
                board = match board.moves().into_iter().find(|(_, i)| *i == move_id) {
                    Some((new_board, _)) => new_board,
                    None => panic!("Unable to find move id {move_id}!")
                };
                println!("{board}");
            }
            println!("{}", soln.latest);
            println!("{s_len} moves");
        }
    }
    println!("Explored {n_explored} states");
    let dur = SystemTime::now().duration_since(start).unwrap();
    println!("Took {:.4}s", dur.as_secs_f64());
}
