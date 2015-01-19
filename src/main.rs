extern crate sdl2;

use sdl2::{
    controller,
    event,
    //joystick,
    timer
};

use std::io::{
    BufferedReader,
    File
};

use std::ffi::CString;
use std::ptr::PtrExt;

use std::num::Float;

const MAX_PLAYERS: usize = 4;



struct ControllerState {
    left_x: i16,
    left_y: i16
}

impl Copy for ControllerState {
}

impl ControllerState {
    fn new() -> ControllerState {
        let state: ControllerState = ControllerState {
            left_x: 0,
            left_y: 0
        };
        return state;
    }
}

trait Entity {
    fn tick(&mut self, u32, [ControllerState; MAX_PLAYERS]);
}

struct Paddle {
    s: [f32; 2],
    v: [f32; 2],
    a: [f32; 2],
    radius: f32,
    friction: f32,
    player: u8
}

impl Paddle {
    fn new(player_number: u8) -> Paddle {
        let paddle: Paddle = Paddle {
            s:  match player_number {
                0 => [85.0, 127.0],
                1 => [171.0, 127.0],
                2 => [127.0, 85.0],
                3 => [127.0, 171.0],
                _ => [0.0, 0.0]
            },
            v: [0.0, 0.0],
            a: [0.0, 0.0],
            radius: 32.0,
            friction: 1.0,
            player: player_number,
        };
        return paddle;
    }
}

impl Entity for Paddle {
    fn tick(&mut self, dt: u32, controller_states: [ControllerState; MAX_PLAYERS]) {
        let t = dt as f32;
        self.a[0] = controller_states[self.player as usize].left_x as f32 / 32767.0;
        self.a[1] = controller_states[self.player as usize].left_y as f32 / 32767.0;
        self.v[0] = self.v[0] + self.a[0]*t;
        self.v[1] = self.v[1] + self.a[1]*t;
        self.s[0] = self.s[0] + self.v[0]*t + 0.5*self.a[0]*t.powi(2);
        self.s[1] = self.s[1] + self.v[1]*t + 0.5*self.a[1]*(t.powi(2));
        println!("[F[Ks: [{},{}]", self.s[0], self.s[1]);
        for v in self.v.iter_mut() {
            if *v > self.friction {
                *v -= self.friction;
            } else if *v < -self.friction {
                *v += self.friction;
            } else {
                *v = 0.0;
            }
        }
    }
}

struct Puck {
    s: [f32; 2],
    v: [f32; 2],
    radius: f32,
    height: f32,
    bounced: bool,
    friction: f32
}

impl Puck {
    fn new<'a>() -> Puck {
        let puck = Puck {
            s: [0.5*(config.window_width as f32), 0.5*(config.window_height as f32)],
            v: [0.0, 0.0],
            radius: 16.0,
            height: config.window_height as f32 + 2.0*16.0,
            bounced: false,
            friction: 0.0
        };
        return puck;
    }
}

impl Entity for Puck {
    fn tick(&mut self, dt: u32, controller_states: [ControllerState; MAX_PLAYERS]) {
        let (mut s, mut v) = (self.s, self.v);
        let t = dt as f32;
        s[0] = s[0] + v[0]*t;
        s[1] = s[1] + v[1]*t;
        for vi in v.iter_mut() {
            if *vi > 0.0 {
                *vi -= self.friction;
            } else if *vi < 0.0 {
                *vi += self.friction;
            }
        }
    }
}

static config: Config = Config {
    window_height: 600,
    window_width: 600
};

struct Config {
    window_height: u32,
    window_width: u32
}

fn main() {
    init();

    let mut controller_states: [ControllerState; MAX_PLAYERS] = [ControllerState::new(); MAX_PLAYERS];
    let mut entities: Vec<Box<Entity + 'static>> = Vec::new();

    let mut controllers: Vec<(i32, *const controller::ll::SDL_GameController)> = Vec::with_capacity(MAX_PLAYERS);
    //idx, *controller
    let mut open_controllers: Vec<(i32, *const controller::ll::SDL_GameController)> = Vec::with_capacity(MAX_PLAYERS);

    let mut t1 = timer::get_ticks();
    'game: loop {
        //controller_states = [ControllerState::new(); MAX_PLAYERS];
        'event: loop {
            let event = event::poll_event();
            match event {
                event::Event::None => break 'event,

                event::Event::Quit(_) => {quit(); return;},

                event::Event::JoyDeviceAdded(_, idx) => {
                    unsafe{
                        if controller::ll::SDL_IsGameController(idx as i32) == 1 {
                            let controller = controller::ll::SDL_GameControllerOpen(idx as i32);
                            if ! PtrExt::is_null(controller) {
                                //Would be a little nicer to insert into "None"s, but only a problem if
                                //you're spamming controller additions.
                                open_controllers.push((idx, controller));
                                println!("Controller Added: {}. Open controllers: {}", idx, open_controllers.len());
                            } else {
                                println!("Failed to add controller {}. Error: {}", idx, sdl2::get_error());
                            }
                        } else {
                            println!("Unsupported controller detected!");
                        }
                    }
                }

                event::Event::ControllerButtonDown(_, idx, controller::ControllerButton::Start) => {
                    if controllers.len() <= MAX_PLAYERS {
                        let mut player_exists = false;
                        for controller in controllers.iter() {
                            let &(controller_idx, _) = controller;
                            if controller_idx == idx {
                                player_exists = true;
                            }
                        }
                        if !player_exists {
                            for open_controller in open_controllers.iter() {
                                let &(open_idx, _) = open_controller;
                                if idx == open_idx {
                                    controllers.push(*open_controller);
                                    let len = controllers.len();
                                    let paddle = Paddle::new(len as u8 - 1);
                                    entities.push(Box::new(paddle));
                                    //Game can start with 2 or more players
                                    if len == 2 {
                                        //add puck
                                        let mut puck = Puck::new();
                                        entities.push(Box::new(puck));
                                        println!("Starting Game");
                                    }
                                    break;
                                }
                            }
                        }
                    }
                },

                event::Event::ControllerAxisMotion(_, idx, axis, value) => {
                    for i in range(0, controllers.len()) {
                        let (controller_id, _) = controllers[i];
                        if controller_id == idx {
                            match axis {
                                controller::ControllerAxis::LeftX => controller_states[i].left_x = value,
                                controller::ControllerAxis::LeftY => controller_states[i].left_y = value,
                                _ => ()
                            }
                            break;
                        }
                    }

                },

                _ => ()
            }
        }
        let t2 = timer::get_ticks();
        for entity in entities.iter_mut() {
            entity.tick((t2 - t1) as u32, controller_states);
        }
        t1 = timer::get_ticks();
    }
}

fn init() {
    sdl2::init(sdl2::INIT_GAME_CONTROLLER);
    unsafe {
        controller::ll::SDL_GameControllerEventState(1); //Enables controller events
    }
    //Add mappings from file.
    let path = Path::new("assets/gamecontrollerdb.txt");
    let mut file = BufferedReader::new(File::open(&path));
    for line in file.lines() {
        let mapping = line.unwrap();
        let first_char = mapping.graphemes(true).next();
        if first_char != Some("#") && first_char != None { //check if first character is a '#' non-destructively
            let cmapping = CString::from_slice(mapping.as_bytes());
            unsafe {
                controller::ll::SDL_GameControllerAddMapping(cmapping.as_ptr());
            }
        }
    }
}

fn quit() {
    sdl2::quit();
}
