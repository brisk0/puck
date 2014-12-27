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

fn main() {
    init();
    loop {
        let event = event::poll_event();
        match event {
            event::Event::None => timer::delay(500),
            event::Event::Quit(_) => {quit(); break;},
            _ => println!("{}", event)
        }
    }
}

fn init() {
    sdl2::init(sdl2::INIT_GAME_CONTROLLER);
    unsafe {
        controller::ll::SDL_GameControllerEventState(1); //Enables controller events
    }
    //Add mappings from file. Replace with SDL_GameControllerAddMappingsFromFile when supported
    let path = Path::new("assets/gamecontrollerdb.txt");
    let mut file = BufferedReader::new(File::open(&path));
    for line in file.lines() {
        let mapping = line.unwrap();
        let first_char = mapping.graphemes(true).next();
        if first_char != Some("#") && first_char != None { //check if first character is a '#' non-destructively
            let cmapping = mapping.to_c_str();
            unsafe {
                controller::ll::SDL_GameControllerAddMapping(cmapping.as_ptr());
            }
        }
    }
}

fn quit() {
    sdl2::quit();
}
