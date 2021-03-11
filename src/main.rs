mod chip8;
mod graphics;

use chip8::Chip8;
use graphics::System;
use std::{env, error::Error, ffi::OsStr, fs};
use tetra::{ContextBuilder};

pub const SCALE: f32 = 16.0;

fn main() {
    if let Some(file) = env::args_os().nth(1) {
        let res = exec(&file);
        if let Err(e) = res {
            eprintln!("Error: {}", e.to_string());
        }
    } else {
        eprintln!("Error: Please specify a CHIP8 executable to interpret.");
    }
}

fn exec(file: &OsStr) -> Result<(), Box<dyn Error>> {
    let file = fs::read(file)?;

    let mut chip8 = Chip8::new();
    chip8.load_game(file);

    ContextBuilder::new("chippy8", (64.0 * SCALE) as i32, (32.0 * SCALE) as i32)
        .quit_on_escape(true)
        .build()?
        .run(|_| Ok(System::new(chip8)))
        .map_err(|e| Box::new(e) as Box<dyn Error>)
}
