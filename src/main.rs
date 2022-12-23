use std::collections::HashSet;

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
        } else if x == -1 {
            Tile::Exit
        } else {
            *self
                .tiles
                .get(y as usize)
                .and_then(|r| r.get(x as usize))
                .expect("x and y should be positive")
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Player {
    x: isize,
    y: isize,
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
enum Tile {
    None,
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
    let new_p = match d {
        Dir::Up => Player { x: p.x, y: p.y - 1 },
        Dir::Down => Player { x: p.x, y: p.y + 1 },
        Dir::Right => Player { x: p.x + 1, y: p.y },
        Dir::Left => Player { x: p.x - 1, y: p.y },
    };

    if visited.contains(&new_p) {
        return (State::Repeated, visited, history);
    }

    visited.insert(new_p);
    history.push(d);

    let tile = b.get_tile(new_p);

    if let Tile::Ice = tile {
        apply(d, b, new_p, visited, history) // Repeat this step (sliding along ice)
    } else {
        let state = match tile {
            Tile::None => State::Just(new_p),
            Tile::Wall => State::Just(p),
            Tile::Teleport => todo!(), // @TODO get position of other teleport
            Tile::Ice => unreachable!(),
            Tile::Pit => State::Dead,
            Tile::Exit => State::Success,
        };

        (state, visited, history)
    }
}

/// Figure out how to get the player to the exit
fn solve(b1: &Board, p1: Player, visited: HashSet<Player>, history: Vec<Dir>) -> Option<Vec<Dir>> {
    [Dir::Up, Dir::Down, Dir::Right, Dir::Left]
        .iter()
        .filter_map(|dir| {
            let (new_p, new_vis, new_hist) = apply(*dir, b1, p1, visited.clone(), history.clone());

            match new_p {
                State::Success => Some(new_hist),
                State::Dead => None,
                State::Repeated => None,
                State::Just(np) => solve(b1, np, new_vis, new_hist),
            }
        })
        .next()
}

fn main() {
    println!("Hello, world!");
}
