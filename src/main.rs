use std::collections::HashSet;
use std::io::Read;

#[derive(PartialEq, Debug)]
enum Error {
    InputEmpty,
    NoExit,
    NoSolution,
    NoPlayer,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
struct Board {
    tiles: Vec<Vec<Tile>>,
    exit: usize,
}

impl Board {
    /// Parse a board definition from the given string
    fn parse(input: &str) -> Result<Self> {
        let mut lines = input.lines();

        let exit = lines
            .next()
            .ok_or(Error::InputEmpty)?
            .char_indices()
            .find_map(|(i, c)| c.eq(&'x').then_some(i))
            .ok_or(Error::NoExit)?;

        let tiles = lines
            .map(|l| {
                l.chars()
                    .map(|c| match c {
                        '.' | 'R' => Tile::None,
                        'T' => Tile::Teleport,
                        'P' => Tile::Pit,
                        'I' => Tile::Ice,
                        'W' => Tile::Wall,
                        _ => unimplemented!(),
                    })
                    .collect()
            })
            .collect();

        Ok(Self { tiles, exit })
    }

    /// Get the tile at the player's position
    fn get_tile(&self, Player { x, y }: Player) -> Tile {
        if y == -1 {
            if x >= 0 && usize::try_from(x).unwrap().eq(&self.exit) {
                Tile::Exit
            } else {
                Tile::Wall
            }
        } else if y == self.tiles.len() as isize || x == -1 || x == self.tiles[0].len() as isize {
            Tile::Wall
        } else {
            *self
                .tiles
                .get(y as usize)
                .and_then(|r| r.get(x as usize))
                .expect("x and y should be positive")
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Player {
    x: isize,
    y: isize,
}

impl Player {
    /// Find the initial player position in the given board string
    fn parse(input: &str) -> Result<Self> {
        input
            .lines()
            .skip(1)
            .enumerate()
            .find_map(|(y, r)| {
                r.char_indices().find_map(|(x, t)| {
                    matches!(t, 'R').then(|| Self {
                        x: x as isize,
                        y: y as isize,
                    })
                })
            })
            .ok_or(Error::NoPlayer)
    }

    /// Hop one space in a direction
    fn hop(&self, d: Dir) -> Player {
        match d {
            Dir::Up => Player {
                x: self.x,
                y: self.y - 1,
            },
            Dir::Down => Player {
                x: self.x,
                y: self.y + 1,
            },
            Dir::Right => Player {
                x: self.x + 1,
                y: self.y,
            },
            Dir::Left => Player {
                x: self.x - 1,
                y: self.y,
            },
        }
    }

    /// Slide on ice
    fn slide(self, d: Dir, b: &Board) -> PlayerState {
        PlayerState::from(d, self, self.hop(d), b)
    }

    /// Use a teleport
    fn teleport(self, b: &Board) -> Self {
        if !matches!(b.get_tile(self), Tile::Teleport) {
            panic!("Tried to get teleport target of non-teleport tile");
        }

        let (x, y) = b
            .tiles
            .iter()
            .enumerate()
            .find_map(|(y, row)| {
                row.iter().enumerate().find_map(|(x, tile)| {
                    (matches!(tile, Tile::Teleport)
                        && !(x == self.x as usize && y == self.y as usize))
                        .then_some((x as isize, y as isize))
                })
            })
            .expect("No second teleport tile found");

        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Dir {
    Up,
    Down,
    Right,
    Left,
}

enum PlayerState {
    Success,
    Dead,
    Just(Player),
}

#[derive(Clone, Copy)]
enum TurnState {
    Success,
    Fail,
    Ok,
}

impl PlayerState {
    fn from(dir: Dir, from: Player, to: Player, board: &Board) -> Self {
        let tile = board.get_tile(to);

        match tile {
            Tile::None => PlayerState::Just(to),
            Tile::Wall => PlayerState::Just(from),
            Tile::Teleport => PlayerState::Just(to.teleport(board)),
            Tile::Ice => to.slide(dir, board),
            Tile::Pit => PlayerState::Dead,
            Tile::Exit => PlayerState::Success,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Tile {
    None,
    Wall,
    Teleport,
    Pit,
    Ice,
    Exit,
}

/// Move the player in the given direction and find out what happens
fn apply(d: Dir, b: &Board, p: Player) -> PlayerState {
    let new_p = p.hop(d);

    PlayerState::from(d, p, new_p, b)
}

#[derive(Clone)]
struct Turn<'a> {
    b1: &'a Board,
    p1: Player,
    b2: &'a Board,
    p2: Player,
    visited: HashSet<(Player, Player)>,
    history: Vec<Dir>,
    state: TurnState,
}

impl<'a, 'b> Turn<'b>
where
    'a: 'b,
{
    fn new(b1: &'a Board, p1: Player, b2: &'a Board, p2: Player) -> Self {
        Self {
            b1,
            p1,
            b2,
            p2,
            visited: HashSet::from([(p1, p2)]),
            history: Vec::new(),
            state: TurnState::Ok,
        }
    }

    fn apply(mut self, dir: Dir) -> Self {
        let state1 = apply(dir, self.b1, self.p1);
        let state2 = apply(dir, self.b2, self.p2);

        self.history.push(dir);

        self.state = match (state1, state2) {
            (PlayerState::Success, PlayerState::Success) => TurnState::Success,
            (PlayerState::Just(p1), PlayerState::Just(p2)) => {
                let vis_entry = (p1, p2);

                if self.visited.contains(&vis_entry) {
                    TurnState::Fail
                } else {
                    self.visited.insert(vis_entry);
                    self.p1 = p1;
                    self.p2 = p2;
                    TurnState::Ok
                }
            }
            _ => TurnState::Fail,
        };

        self
    }
}

/// Figure out how to get the player to the exit
fn solve(turns: Vec<Turn>) -> Option<Vec<Dir>> {
    if turns.is_empty() {
        None
    } else if let Some(turn) = turns.iter().find(|t| matches!(t.state, TurnState::Success)) {
        println!("We've made it!");
        Some(turn.history.clone())
    } else {
        println!("Evaluating {} paths", turns.len());

        solve(
            turns
                .into_iter()
                .filter(|t| matches!(t.state, TurnState::Ok))
                .flat_map(|turn| {
                    [Dir::Up, Dir::Down, Dir::Right, Dir::Left]
                        .into_iter()
                        .map(move |dir| turn.clone().apply(dir))
                })
                .collect(),
        )
    }
}

/// Figure out the shortest path to get the player to the exit
fn solve_puzzle(input: &str) -> Result<Vec<Dir>> {
    let (input1, input2) = input
        .split_once("\n\n")
        .expect("Couldn't find second board");

    let b1 = Board::parse(input1)?;
    let p1 = Player::parse(input1)?;
    let b2 = Board::parse(input2)?;
    let p2 = Player::parse(input2)?;

    println!(
        "A starting at ({}, {}), B starting at ({}, {})",
        p1.x, p1.y, p2.x, p2.y
    );

    solve(vec![Turn::new(&b1, p1, &b2, p2)]).ok_or(Error::NoSolution)
}

#[cfg(test)]
mod tests {
    use super::Dir::*;

    #[test]
    fn simple() {
        let input = "
 x
...
...
.R.

 x
...
...
..R
"
        .trim_matches('\n');
        assert_eq!(
            Ok(vec![Up, Up, Right, Left, Up]),
            super::solve_puzzle(input)
        )
    }

    #[test]
    fn ice() {
        let input = "
 x
...
.IW
..R

  x
...
.II
..R
"
        .trim_matches('\n');

        assert_eq!(
            Ok(vec![Left, Left, Up, Right, Up, Up]),
            super::solve_puzzle(input)
        );
    }

    #[test]
    fn teleport_and_pit() {
        let input = "
  x
...
.I.
.R.

  x
...
TPT
.R.
"
        .trim_matches('\n');

        assert_eq!(
            Ok(vec![Left, Up, Right, Up, Up]),
            super::solve_puzzle(input)
        );
    }
}

fn main() {
    let mut input = String::new();

    std::io::stdin()
        .read_to_string(&mut input)
        .expect("couldn't read stdin");

    match solve_puzzle(&input) {
        Ok(directions) => {
            println!("SOLUTION:");
            for dir in directions {
                println!("{:?}", dir);
            }
        }
        Err(err) => {
            println!("Couldn't solve puzzle: {:?}", err);
        }
    }
}
