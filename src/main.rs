mod chip8;

use chip8::Chip8;

fn main() {
    // TODO: Graphics and input init

    let mut chip8 = Chip8::new();
    loop {
        chip8.cycle();
    }
}
