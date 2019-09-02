/*
 * Developed by Ellie Ang. (git@angm.xyz).
 * Last modified on 9/2/19 10:01 PM.
 * This file is under the GPL3 license. See LICENSE in the root directory of this repository for details.
 */

mod chip8;

use chip8::Chip8;

fn main() {
    // TODO: Graphics and input init

    let mut chip8 = Chip8::new();
    loop {
        chip8.cycle();
    }
}
