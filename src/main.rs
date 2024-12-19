use core::fmt;
use crossterm::{
    cursor::{Hide, MoveTo},
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{
        Clear,
        ClearType::{All, Purge},
        DisableLineWrap,
    },
};
use std::{
    collections::HashMap,
    io::{stdin, stdout},
};
const WIDTH: u16 = 40;
const HEIGHT: u16 = 20;
const NUM_WALLS: u16 = 100;
const NUM_DEAD_ROBOTS: u16 = 2;
const NUM_ROBOTS: u16 = 10;
const NUM_TELEPORTS: u8 = 2;
//SPRITES
const SPACE: char = ' ';
const PLAYER: char = '@';
const ROBOT: char = 'R';
const DEAD_ROBOT: char = 'X';
const WALL: char = '░';
const LINE_CLEAR: &str = "                              \n";

fn main() {
    execute!(
        stdout(),
        Clear(All),
        Clear(Purge),
        Hide,
        DisableLineWrap,
        MoveTo(0, 0)
    )
    .unwrap();

    let mut board = Game::new();

    'game: loop {
        board.print();

        if let Some(input) = get_player_input(&mut board) {
            //move the player before robots calculate movement
            board.update_player_position(input);

            let robot_destinations = get_next_robot_positions(&mut board);

            //remove all robot origins
            for robot in board.robot_current_positions.iter() {
                board.board.insert(*robot, Sprites::Empty);
            }
            //put robots in destination
            for robot in &robot_destinations {
                board.board.insert(*robot, Sprites::Robot);
            }
            board.robot_current_positions = robot_destinations.clone();

            if robot_destinations.contains(&input) {
                board.print();
                execute!(
                    stdout(),
                    MoveTo(0, HEIGHT + 2),
                    SetForegroundColor(Color::White),
                    Print("You've been caught by a robot!")
                )
                .unwrap();
                break 'game;
            } else if board.robot_current_positions.is_empty() {
                board.print();
                execute!(
                    stdout(),
                    MoveTo(0, HEIGHT + 2),
                    SetForegroundColor(Color::White),
                    Print("All the robots have crashed and you live!")
                )
                .unwrap();
                break 'game;
            }
        } else {
            break 'game;
        }
    }
}

fn get_player_input(board: &mut Game) -> Option<(u16, u16)> {
    let valid_options = board.get_valid_destinations(board.player_current_position);
    let mut option_string = String::new();
    for option in &valid_options {
        option_string = option_string + ", " + option.to_string();
    }

    option_string = option_string.trim_start_matches(", ").to_string();
    let mut destination = None;
    'input: loop {
        execute!(
            stdout(),
            MoveTo(0, HEIGHT + 1),
            SetForegroundColor(Color::White)
        )
        .unwrap();
        println!("(T)eleports remaining: {}", board.teleport_count);
        println!("Move with: {}", option_string);
        println!("Or type QUIT to exit.");
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        input = input.trim().to_uppercase();

        let action = InputOptions::get_from_string(input);

        if (action.is_movement() && !valid_options.contains(&action))
            || action == InputOptions::Invalid
            || action == InputOptions::Teleport && board.teleport_count == 0
        {
            execute!(
                stdout(),
                MoveTo(0, HEIGHT + 1),
                Print(LINE_CLEAR.to_owned() + LINE_CLEAR + LINE_CLEAR + LINE_CLEAR)
            )
            .unwrap();
            println!("Invalid selection. Please try again.");
            continue 'input;
        } else if action == InputOptions::Teleport && board.teleport_count > 0 {
            destination = Some(board.get_random_empty_space());
            board.teleport_count -= 1;
            break 'input;
        } else if action.is_movement() {
            match action {
                InputOptions::Move(dx, dy) => {
                    destination = Some((
                        board
                            .player_current_position
                            .0
                            .checked_add_signed(dx)
                            .unwrap(),
                        board
                            .player_current_position
                            .1
                            .checked_add_signed(dy)
                            .unwrap(),
                    ));
                }
                _ => (),
            }
            break 'input;
        } else if action == InputOptions::Quit {
            destination = None;
            println!("Thanks for playing.");
            break 'input;
        }
    }
    destination
}

fn get_next_robot_positions(board: &mut Game) -> Vec<(u16, u16)> {
    let mut robot_destinations: Vec<(u16, u16)> = vec![];

    for (x, y) in board.robot_current_positions.iter() {
        //initial move calculation
        let mut new_x = match x.cmp(&board.player_current_position.0) {
            std::cmp::Ordering::Greater => x.checked_add_signed(-1).unwrap(),
            std::cmp::Ordering::Less => x + 1,
            std::cmp::Ordering::Equal => x + 0,
        };
        let mut new_y = match y.cmp(&board.player_current_position.1) {
            std::cmp::Ordering::Greater => y.checked_add_signed(-1).unwrap(),
            std::cmp::Ordering::Less => y + 1,
            std::cmp::Ordering::Equal => y + 0,
        };
        //attempt to adjust if it would move into a wall
        if let Some(destination_char) = board.board.get(&(new_x, new_y)) {
            if *destination_char == Sprites::Wall {
                if *destination_char == Sprites::Empty {
                    new_y = *y;
                } else if *destination_char == Sprites::Empty {
                    new_x = *x;
                } else {
                    new_x = *x;
                    new_y = *y;
                }
            }
        }
        //Clear dead bots from last cycle --move to separate function
        let mut dead_bots = vec![];
        board
            .board
            .iter()
            .filter(|(_, value)| **value == Sprites::DeadRobot)
            .for_each(|dead_bot| dead_bots.push(*dead_bot.0));

        for position in dead_bots {
            board.board.insert(position, Sprites::Empty);
        }

        if *board.board.get(&(*x, *y)).unwrap() == Sprites::DeadRobot
            || *board.board.get(&(new_x, new_y)).unwrap() == Sprites::DeadRobot
        {
            continue;
        }
        //move and possibly crash
        if robot_destinations.contains(&(new_x, new_y)) {
            board.board.insert((new_x, new_y), Sprites::DeadRobot);
            robot_destinations = robot_destinations
                .into_iter()
                .filter(|(x, y)| !(*x == new_x && *y == new_y))
                .collect::<Vec<(u16, u16)>>();
        } else {
            robot_destinations.push((new_x, new_y));
        }
    }
    robot_destinations
}

#[derive(PartialEq)]
enum InputOptions {
    Quit,
    Move(i16, i16),
    Teleport,
    Invalid,
}
impl InputOptions {
    fn get_from_string(str: String) -> InputOptions {
        match str.as_str() {
            "QUIT" => InputOptions::Quit,
            "W" => InputOptions::Move(0, -1),
            "X" => InputOptions::Move(0, 1),
            "A" => InputOptions::Move(-1, 0),
            "D" => InputOptions::Move(1, 0),
            "Q" => InputOptions::Move(-1, -1),
            "Z" => InputOptions::Move(-1, 1),
            "C" => InputOptions::Move(1, 1),
            "E" => InputOptions::Move(1, -1),
            "S" => InputOptions::Move(0, 0),
            "T" => InputOptions::Teleport,
            _ => InputOptions::Invalid,
        }
    }
    fn is_movement(&self) -> bool {
        match self {
            InputOptions::Move(_, _) => true,
            _ => false,
        }
    }
    fn to_string(&self) -> &str {
        match self {
            InputOptions::Quit => "QUIT",
            InputOptions::Move(0, -1) => "W",
            InputOptions::Move(0, 1) => "X",
            InputOptions::Move(-1, 0) => "A",
            InputOptions::Move(1, 0) => "D",
            InputOptions::Move(-1, -1) => "Q",
            InputOptions::Move(-1, 1) => "Z",
            InputOptions::Move(1, 1) => "C",
            InputOptions::Move(1, -1) => "E",
            InputOptions::Move(0, 0) => "S",
            InputOptions::Teleport => "T",
            _ => "",
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Sprites {
    Wall,
    Player,
    Robot,
    DeadRobot,
    Empty,
}

impl fmt::Display for Sprites {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sprites::Player => write!(f, "{}", PLAYER),
            Sprites::DeadRobot => write!(f, "{}", DEAD_ROBOT),
            Sprites::Robot => write!(f, "{}", ROBOT),
            Sprites::Wall => write!(f, "{}", WALL),
            Sprites::Empty => write!(f, "{}", SPACE),
        }
    }
}

#[derive(Clone)]
struct Game {
    board: HashMap<(u16, u16), Sprites>,
    player_current_position: (u16, u16),
    teleport_count: u8,
    robot_current_positions: Vec<(u16, u16)>,
}
impl Game {
    fn new() -> Self {
        let mut board = Game {
            board: HashMap::new(),
            player_current_position: (0, 0), //TODO diff value
            teleport_count: NUM_TELEPORTS,
            robot_current_positions: vec![],
        };

        for x in 0..=WIDTH {
            for y in 0..=HEIGHT {
                if x == 0 || y == 0 || x == WIDTH || y == HEIGHT {
                    board.board.insert((x, y), Sprites::Wall);
                } else {
                    board.board.insert((x, y), Sprites::Empty);
                }
            }
        }
        for _ in 0..NUM_WALLS {
            board
                .board
                .insert(board.get_random_empty_space(), Sprites::Wall);
        }
        for _ in 0..NUM_DEAD_ROBOTS {
            board
                .board
                .insert(board.get_random_empty_space(), Sprites::DeadRobot);
        }
        for _ in 0..NUM_ROBOTS {
            let robot = board.get_random_empty_space();
            board.board.insert(robot, Sprites::Robot);
            board.robot_current_positions.push(robot);
        }

        board.player_current_position = board.get_random_empty_space();
        board
            .board
            .insert(board.player_current_position, Sprites::Player);

        board
    }

    fn get_random_empty_space(&self) -> (u16, u16) {
        //iter visits in arbitrary order, last gets one. this is random enough.
        let (space, _) = self
            .board
            .iter()
            .filter(|(_, sprite)| **sprite == Sprites::Empty)
            .last()
            .unwrap();
        *space
    }

    fn print(&self) {
        execute!(stdout(), Clear(All), Clear(Purge)).unwrap();
        for ((x, y), sprite) in &self.board {
            let color = match sprite {
                Sprites::Player => Color::Green,
                Sprites::Robot => Color::Red,
                Sprites::DeadRobot => Color::DarkRed,
                Sprites::Wall => Color::Grey,
                Sprites::Empty => Color::Black,
            };
            execute!(
                stdout(),
                MoveTo(*x, *y),
                SetForegroundColor(color),
                Print(sprite)
            )
            .unwrap();
        }
    }

    fn get_valid_destinations(&self, origin: (u16, u16)) -> Vec<InputOptions> {
        let neighbors = vec![
            (-1, -1),
            (-1, 1),
            (-1, 0),
            (1, 0),
            (1, 1),
            (1, -1),
            (0, 1),
            (0, -1),
            (0, 0),
        ];
        let mut valid_options = vec![];

        for (dx, dy) in neighbors {
            let destination = (
                origin.0.checked_add_signed(dx).unwrap(),
                origin.1.checked_add_signed(dy).unwrap(),
            );
            match self.board.get(&destination).unwrap() {
                Sprites::Wall | Sprites::Robot => {
                    continue;
                }
                _ => {
                    valid_options.push(InputOptions::Move(dx, dy));
                }
            }
        }
        valid_options
    }

    fn update_player_position(&mut self, destination: (u16, u16)) {
        self.board
            .insert(self.player_current_position, Sprites::Empty);

        self.board.insert(destination, Sprites::Player);

        self.player_current_position = destination;
    }
}
