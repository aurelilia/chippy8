use rand::random;
use tetra::input::Key;

static FONT_SET: [u16; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

const KEYS: [Key; 16] = [
    Key::V,
    Key::B,
    Key::N,
    Key::G,
    Key::H,
    Key::J,
    Key::T,
    Key::Y,
    Key::U,
    Key::Num6,
    Key::Num7,
    Key::Num8,
    Key::A,
    Key::Q,
    Key::W,
    Key::S,
];

/// The full data structure of the Chip8 - contains all state.
pub struct Chip8 {
    memory: [u8; 4096],
    reg: [u8; 16],

    i: u16,
    pc: u16,
    stack: Vec<u16>,
    gfx: [bool; 64 * 32],

    delay_timer: u8,
    sound_timer: u8,
}

impl Chip8 {
    /// Advance state of the emulator by 1 cycle.
    /// Returns if graphics should redraw.
    pub fn cycle(&mut self, is_pressed: impl Fn(Key) -> bool) -> bool {
        let opcode = self.advance();
        self.execute_opcode(opcode, is_pressed);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("Beep!")
            }
            self.sound_timer -= 1;
        }

        (opcode & 0xF000) == 0xD000
    }

    fn execute_opcode(&mut self, code: u16, is_pressed: impl Fn(Key) -> bool) {
        match code & 0xF000 {
            0x0000 => {
                match code & 0x00FF {
                    // Clear screen (0x00E0)
                    0x00E0 => self.gfx = [false; 64 * 32],

                    // Return from subroutine (0x00EE)
                    0x00EE => self.pc = self.stack.pop().expect("Invalid opcode"),

                    _ => println!("Unknown opcode: {:#X}", code),
                }
            }

            // Jump to NNN
            0x1000 => self.pc = nnn(code),

            // Call subroutine at address NNN
            0x2000 => {
                self.stack.push(self.pc);
                self.pc = nnn(code)
            }

            // Skip next instruction if X and NN match
            0x3000 if (self.reg[ux(code)] as u16) == (nn(code)) => self.pc += 2,
            0x3000 => (),

            // Skip next instruction if X and NN do not match
            0x4000 if (self.reg[ux(code)] as u16) != (nn(code)) => self.pc += 2,
            0x4000 => (),

            // Skip next instruction if X and Y match
            0x5000 if (self.reg[ux(code)]) == (self.reg[uy(code)]) => self.pc += 2,
            0x5000 => (),

            // Set X to NN
            0x6000 => self.reg[ux(code)] = nn(code) as u8,

            // Add NN to X; carry flag is not modified
            0x7000 => {
                self.reg[ux(code)] = (self.reg[ux(code)] as u16).overflowing_add(nn(code)).0 as u8
            }

            0x8000 => {
                match code & 0x000F {
                    // Set X to value of Y
                    0x0000 => self.reg[ux(code)] = self.reg[uy(code)],

                    // Set X to value of (X | Y)
                    0x0001 => self.reg[ux(code)] = self.reg[ux(code)] | self.reg[uy(code)],

                    // Set X to value of (X & Y)
                    0x0002 => self.reg[ux(code)] = self.reg[ux(code)] & self.reg[uy(code)],

                    // Set X to value of (X ^ Y)
                    0x0003 => self.reg[ux(code)] = self.reg[ux(code)] ^ self.reg[uy(code)],

                    // Add value of Y to X; set carry
                    0x0004 => {
                        let x = self.reg[ux(code)];
                        let y = self.reg[uy(code)];

                        let (result, carry) = x.overflowing_add(y);
                        self.reg[ux(code)] = result;
                        self.reg[0xF] = carry as u8;
                    }

                    // Subtract value of Y from X; set borrow
                    0x0005 => {
                        let x = self.reg[ux(code)];
                        let y = self.reg[uy(code)];

                        let (result, borrow) = x.overflowing_sub(y);
                        self.reg[ux(code)] = result;
                        self.reg[0xF] = !borrow as u8;
                    }

                    // Store least significant X bit in reg 0xF; shift X 1 to right
                    0x0006 => {
                        self.reg[0xF] = self.reg[ux(code)] & 0x01;
                        self.reg[ux(code)] = self.reg[ux(code)] >> 1;
                    }

                    // Subtract value of X from Y and store in X; set borrow
                    0x0007 => {
                        let x = self.reg[ux(code)];
                        let y = self.reg[uy(code)];

                        let (result, borrow) = y.overflowing_sub(x);
                        self.reg[ux(code)] = result;
                        self.reg[0xF] = !borrow as u8;
                    }

                    // Store most significant X bit in reg 0xF; shift X 1 to left
                    0x0008 => {
                        self.reg[0xF] = self.reg[ux(code)] & 0x80;
                        self.reg[ux(code)] = self.reg[ux(code)] << 1;
                    }

                    _ => println!("Unknown opcode: {:#X}", code),
                }
            }

            // Skip next instruction if X and Y do not match
            0x9000 if (self.reg[ux(code)]) != (self.reg[uy(code)]) => self.pc += 2,
            0x9000 => (),

            // Set I to NNN
            0xA000 => self.i = nnn(code),

            // Jump to NNN + V0
            0xB000 => self.pc = nnn(code) + self.reg[0] as u16,

            // Set X to ((random number) & NN)
            0xC000 => self.reg[ux(code)] = random::<u8>() & nn(code) as u8,

            // Draw sprite (https://en.wikipedia.org/wiki/CHIP-8#Opcode_table)
            0xD000 => {
                self.reg[0xF] = 0;

                let x = self.reg[us(x(code))] as u16;
                let y = self.reg[us(y(code))] as u16;
                for y_line in 0..n(code) {
                    let pixel = self.memory[us(self.i + y_line)];
                    for x_line in 0..8 {
                        if pixel & (0x80 >> x_line) != 0 {
                            if self.gfx[us(x + x_line as u16 + ((y + y_line) * 64))] {
                                self.reg[0xF] = 1;
                            }
                            self.gfx[us(x + x_line as u16 + ((y + y_line) * 64))] ^= true;
                        }
                    }
                }
            }

            0xE000 => {
                match code & 0x00FF {
                    // Skip next instruction if key X is pressed
                    0x009E if is_pressed(KEYS[ux(code)]) => self.pc += 2,
                    0x009E => (),

                    // Skip next instruction if key X is not pressed
                    0x00A1 if !is_pressed(KEYS[ux(code)]) => self.pc += 2,
                    0x00A1 => (),

                    _ => println!("Unknown opcode: {:#X}", code),
                }
            }

            0xF000 => {
                match code & 0x00FF {
                    // Set X to the delay timer
                    0x0007 => self.reg[ux(code)] = self.delay_timer,

                    // Halt until key press; then store in Register F
                    0x000A => {
                        // See if a key is pressed and set RegF if so
                        let key = KEYS.iter().enumerate().find(|(_, key)| is_pressed(**key));
                        if let Some((idx, _)) = key {
                            self.reg[0xF] = idx as u8;
                        } else {
                            // If not, just reset the PC to make the emu keep
                            // executing this instruction until a key is pressed
                            self.pc -= 2;
                        }
                    }

                    // Set delay timer to X
                    0x0015 => self.delay_timer = x(code) as u8,

                    // Set sound timer to X
                    0x0018 => self.sound_timer = x(code) as u8,

                    // Add X to I
                    0x001E => self.i += x(code),

                    // Set I to the location of the sprite of the character in X
                    0x0029 => self.i = x(code) * 5, // (every character is 5 bytes long)

                    // Stores binary-coded representation of X at (I until I+2)
                    0x0033 => {
                        let x = x(code);
                        self.memory[us(self.i)] = (x / 100) as u8;
                        self.memory[us(self.i + 1)] = ((x / 10) % 10) as u8;
                        self.memory[us(self.i + 2)] = ((x % 100) % 10) as u8;
                    }

                    // Write all registers to memory, stating at I
                    0x0055 => {
                        for (i, reg) in self.reg.iter().enumerate() {
                            self.memory[us(self.i + (i as u16))] = *reg;
                        }
                    }

                    // Write to all registers from memory, stating at I
                    0x0065 => {
                        let range = us(self.i)..us(self.i + 16);
                        for (i, dat) in self.memory[range].iter().enumerate() {
                            self.reg[i] = *dat;
                        }
                    }

                    _ => println!("Unknown opcode: {:#X}", code),
                }
            }

            _ => println!("Unknown opcode: {:#X}", code),
        }
    }

    /// Returns the current opcode and advances the program counter by 2
    fn advance(&mut self) -> u16 {
        self.pc += 2;
        (self.memory[us(self.pc - 2)] as u16) << 8 | (self.memory[us(self.pc - 1)] as u16)
    }

    /// Loads the specified game data into the emulator, ready for execution.
    pub fn load_game(&mut self, data: Vec<u8>) {
        for (i, byte) in data.iter().enumerate() {
            self.memory[0x200 + i] = *byte;
        }
    }

    pub fn pixels(&self) -> &[bool] {
        &self.gfx
    }

    pub fn new() -> Self {
        let mut chip8 = Self {
            memory: [0; 4096],
            reg: [0; 16],

            i: 0,
            pc: 0x200,
            stack: Vec::with_capacity(16),
            gfx: [false; 64 * 32],

            delay_timer: 0,
            sound_timer: 0,
        };

        for (i, byte) in FONT_SET.iter().enumerate() {
            let i = i * 2;
            chip8.memory[i] = (byte >> 8) as u8;
            chip8.memory[i + 1] = (byte & 0x00FF) as u8;
        }

        chip8
    }
}

fn us(u: u16) -> usize {
    u as usize
}

fn x(code: u16) -> u16 {
    (code & 0x0F00) >> 8
}

fn ux(code: u16) -> usize {
    us((code & 0x0F00) >> 8)
}

fn y(code: u16) -> u16 {
    (code & 0x00F0) >> 4
}

fn uy(code: u16) -> usize {
    us((code & 0x00F0) >> 4)
}

fn n(code: u16) -> u16 {
    code & 0x000F
}

fn nn(code: u16) -> u16 {
    code & 0x00FF
}

fn nnn(code: u16) -> u16 {
    code & 0x0FFF
}
