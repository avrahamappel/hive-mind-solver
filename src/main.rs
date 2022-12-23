struct Board {
    tiles: Vec<Vec<Tile>>,
    exit: usize,
}

struct Player {
    x: usize, y: usize,
}

enum Dir {
    Up, Down, Right, Left }

enum State { Success(Vec<Dir>), Dead, Repeated, Just(Player) }

enum Tile {
    None,
    Wall,
    Teleport,
    Pit,
    Ice,
    Exit,
}

/// Move the player in the given direction and find out what happens
fn apply(d: Dir, b: Board, p: Player, visited: HashSet<Player>, history: Vec<Dir>) -> State {
    let new_p = match d {
        Dir::Up => Player { x: d.x, y: d.y - 1 },
        Dir::Down => Player { x: d.x, y: d.y + 1},
        Dir::Right => Player { x: d.x + 1, y: d.y },
        Dir::Left => Player { x: d.x - 1, y: d.y },
    };

    match b.getTile(new_p) {
        Tile::None => State::Just(new_p), // @TODO check if tile was already visited
                                   Tile::Wall => State::Just(p),
                                   Tile::Teleport => todo!(), // @TODO get position of other teleport
                                                             Tile::Pit => State::Dead,
                                                             Tile::Ice => apply(d, b, new_p), // Repeat this step (sliding along ice)
                                                                                              Tile::Exit => State::Success(history),
    }
}

/// Figure out how to get the player to the exit
fn solve(b1: Board, p1: Player)->Option<Vec<Dir>> {
    [Dir::Up, Dir::Down, Dir::Right, Dir::Left].iter().filterMap(|dir| {
        let new_p = apply(dir, b1, p1);

        match new_p {
            State::Success => Some(history),
            State::Dead => None,
            State::Repeated => None,
            State::Just(np) => solve(b1, np),
        }
    }).first()
}

fn main() {
    println!("Hello, world!");
}
