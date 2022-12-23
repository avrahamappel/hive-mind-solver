use std::collections::HashSet;

#[derive(Debug)]
struct Board {
    tiles: Vec<Vec<Tile>>,
    exit: usize,
}

impl Board {
    fn get_tile(&self, Player { x, y }: Player) -> Tile {
        if y == -1 {
            if x.is_positive() && usize::try_from(x).unwrap().eq(&self.exit) {
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

    fn parse(input: &str) -> (Self, Player) {
        let mut lines = input.lines();

        let exit = lines
            .next()
            .expect("Input was empty")
            .char_indices()
            .find_map(|(i, c)| c.eq(&'x').then_some(i))
            .expect("Couldn't find exit position");

        let tiles = lines
            .map(|l| {
                l.chars()
                    .map(|c| match c {
                        'R' => Tile::Player,
                        '.' => Tile::None,
                        'T' => Tile::Teleport,
                        'P' => Tile::Pit,
                        'I' => Tile::Ice,
                        'W' => Tile::Wall,
                        _ => unimplemented!(),
                    })
                    .collect()
            })
            .collect();

        let board = Self { tiles, exit };

        let player = board
            .tiles
            .iter()
            .enumerate()
            .find_map(|(y, r)| {
                r.iter().enumerate().find_map(|(x, t)| {
                    matches!(t, Tile::Player).then(|| Player {
                        x: x as isize,
                        y: y as isize,
                    })
                })
            })
            .expect("Player position not specified");

        (board, player)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Player {
    x: isize,
    y: isize,
}

impl Player {
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
    fn slide(self, d: Dir, b: &Board) -> State {
        State::from(d, self, self.hop(d), b)
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
                    (x != self.x.try_into().expect("x should be positive here")
                        && y != self.y.try_into().expect("y should be positive here")
                        && matches!(tile, Tile::Teleport))
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

enum State {
    Success,
    Dead,
    Repeated,
    Just(Player),
}

impl State {
    fn from(dir: Dir, from: Player, to: Player, board: &Board) -> Self {
        let tile = board.get_tile(to);
        dbg!(tile);

        match tile {
            Tile::None | Tile::Player => State::Just(to),
            Tile::Wall => State::Just(from),
            Tile::Teleport => State::Just(to.teleport(board)),
            Tile::Ice => to.slide(dir, board),
            Tile::Pit => State::Dead,
            Tile::Exit => State::Success,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Tile {
    None,
    Player,
    Wall,
    Teleport,
    Pit,
    Ice,
    Exit,
}

/// Move the player in the given direction and find out what happens
fn apply(
    d: Dir,
    b: &Board,
    p: Player,
    mut visited: HashSet<Player>,
    mut history: Vec<Dir>,
) -> (State, HashSet<Player>, Vec<Dir>) {
    println!();
    println!("------------------------");
    dbg!(d);
    let new_p = p.hop(d);

    dbg!(new_p);

    if visited.contains(&new_p) {
        return (State::Repeated, visited, history);
    }

    visited.insert(new_p);
    history.push(d);

    dbg!(&visited, &history);

    (State::from(d, p, new_p, b), visited, history)
}

/// Figure out how to get the player to the exit
fn solve(b1: &Board, p1: Player, visited: HashSet<Player>, history: Vec<Dir>) -> Option<Vec<Dir>> {
    [Dir::Up, Dir::Down, Dir::Right, Dir::Left]
        .iter()
        .find_map(|dir| {
            let (new_p, new_vis, new_hist) = apply(*dir, b1, p1, visited.clone(), history.clone());

            match new_p {
                State::Success => Some(new_hist),
                State::Dead | State::Repeated => None,
                State::Just(np) => solve(b1, np, new_vis, new_hist),
            }
        })
}

fn solve_puzzle(input: &str) -> Option<Vec<Dir>> {
    let (board, player) = Board::parse(input);

    dbg!(&board, player);

    solve(&board, player, HashSet::from([player]), Vec::new())
}

#[cfg(test)]
mod tests {
    use super::Dir::*;

    #[test]
    fn simple() {
        let input = "
 x
.....
.....
.....
...R.
.....
"
        .trim_matches('\n');
        assert_eq!(
            Some(vec![
                Up, Up, Up, Up, Right, Up, Down, Down, Down, Down, Down, Right, Left, Down, Left,
                Up, Up, Up, Up, Up, Left, Up
            ]),
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
"
        .trim_matches('\n');

        assert_eq!(
            Some(vec![Up, Down, Right, Left, Up, Up]),
            super::solve_puzzle(input)
        );
    }
}

fn main() {
    let mut input = String::new();

    std::io::stdin()
        .read_line(&mut input)
        .expect("couldn't read stdin");

    if let Some(directions) = solve_puzzle(&input) {
        for dir in directions {
            println!("{:?}", dir);
        }
    } else {
        println!("Couldn't solve puzzle");
    }
}
