/*
 * Developed by Ellie Ang. (git@angm.xyz).
 * Last modified on 9/2/19 10:01 PM.
 * This file is under the GPL3 license. See LICENSE in the root directory of this repository for details.
 */

#[macro_use]
extern crate glium;
#[macro_use]
extern crate imgui;

mod chip8;
mod graphics;

use chip8::Chip8;
use imgui::{Ui, Condition};
use glium::glutin;
use std::process;
use std::fs;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let mut chip8 = Chip8::new();
    chip8.load_game(fs::read("games/TETRIS").expect("I CANT READ!"));

    let mut system = graphics::setup(chip8);
    loop {
        // TODO: Proper timing
        sleep(Duration::from_micros(16));
        graphics::input(&mut system);
        system.chip8.cycle();
        graphics::draw(&mut system);
    }
}

fn handle_input(event: glutin::WindowEvent, chip8: &mut Chip8) {
    match event {
        glutin::WindowEvent::CloseRequested => process::exit(0),
        _ => (),
    }
}

fn draw_gui(ui: &mut Ui, chip8: &Chip8) {
    ui.window(im_str!("Hello world"))
        .size([300.0, 100.0], Condition::FirstUseEver)
        .build(|| {
            ui.text(im_str!("Hello world!"));
            ui.text(im_str!("こんにちは世界！"));
            ui.text(im_str!("This...is...imgui-rs!"));
            ui.separator();
            let mouse_pos = ui.io().mouse_pos;
            ui.text(im_str!("Mouse Position: ({:.1},{:.1})", mouse_pos[0], mouse_pos[1]));
        })
}
