use rand::random;
use tetra::input::Key;

static FONT_SET: [u8; 80] = [
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
    Key::V, // 0
    Key::B, // 1
    Key::N, // 2
    Key::G, // 3
    Key::H, // 4
    Key::J, // 5
    Key::T, // 6
    Key::Y, // 7
    Key::U, // 8
    Key::Num6, // 9
    Key::Num7, // A
    Key::Num8, // B
    Key::A, // C
    Key::Q, // D
    Key::W, // E
    Key::S, // F
];

const FPS: usize = 60;
const CLOCK_SPEED: usize = 540;

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
    /// Advance state of the emulator by 1 'tick'.
    /// A tick happens when `delay_timer` is counted
    /// down, and this should therefore be called at 60Hz.
    pub fn tick(&mut self, is_pressed: impl Fn(Key) -> bool) {
        for _ in 0..(CLOCK_SPEED / FPS) {
            self.cycle(&is_pressed);
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("Beep!")
            }
            self.sound_timer -= 1;
        }
    }

    /// Advance state of the emulator by 1 cycle.
    fn cycle(&mut self, is_pressed: &impl Fn(Key) -> bool) {
        let opcode = self.advance();
        self.execute_opcode(opcode, is_pressed);
    }

    fn execute_opcode(&mut self, code: u16, is_pressed: &impl Fn(Key) -> bool) {
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

            // Skip next instruction if VX and NN match
            0x3000 if (self.reg[ux(code)] as u16) == (nn(code)) => self.pc += 2,
            0x3000 => (),

            // Skip next instruction if VX and NN do not match
            0x4000 if (self.reg[ux(code)] as u16) != (nn(code)) => self.pc += 2,
            0x4000 => (),

            // Skip next instruction if VX and VY match
            0x5000 if (self.reg[ux(code)]) == (self.reg[uy(code)]) => self.pc += 2,
            0x5000 => (),

            // Set VX to NN
            0x6000 => self.reg[ux(code)] = nn(code) as u8,

            // Add NN to VX; carry flag is not modified
            0x7000 => {
                self.reg[ux(code)] = (self.reg[ux(code)] as u16).overflowing_add(nn(code)).0 as u8
            }

            0x8000 => {
                match code & 0x000F {
                    // Set VX to value of VY
                    0x0000 => self.reg[ux(code)] = self.reg[uy(code)],

                    // Set VX to value of (VX | VY)
                    0x0001 => self.reg[ux(code)] |= self.reg[uy(code)],

                    // Set VX to value of (VX & VY)
                    0x0002 => self.reg[ux(code)] &= self.reg[uy(code)],

                    // Set VX to value of (VX ^ VY)
                    0x0003 => self.reg[ux(code)] ^= self.reg[uy(code)],

                    // Add value of VY to VX; set VF to carry
                    0x0004 => {
                        let x = self.reg[ux(code)];
                        let y = self.reg[uy(code)];

                        let (result, carry) = x.overflowing_add(y);
                        self.reg[ux(code)] = result;
                        self.reg[0xF] = carry as u8;
                    }

                    // Subtract value of VY from VX; set borrow
                    0x0005 => {
                        let x = self.reg[ux(code)];
                        let y = self.reg[uy(code)];

                        let (result, borrow) = x.overflowing_sub(y);
                        self.reg[ux(code)] = result;
                        self.reg[0xF] = !borrow as u8;
                    }

                    // Store least significant VX bit in reg 0xF; shift VX 1 to right
                    0x0006 => {
                        self.reg[0xF] = self.reg[ux(code)] & 1;
                        self.reg[ux(code)] >>= 1;
                    }

                    // Subtract value of VX from VY and store in VX; set borrow
                    0x0007 => {
                        let x = self.reg[ux(code)];
                        let y = self.reg[uy(code)];

                        let (result, borrow) = y.overflowing_sub(x);
                        self.reg[ux(code)] = result;
                        self.reg[0xF] = !borrow as u8;
                    }

                    // Store most significant VX bit in reg 0xF; shift VX 1 to left
                    0x000E => {
                        self.reg[0xF] = self.reg[ux(code)] >> 7;
                        self.reg[ux(code)] <<= 1;
                    }

                    _ => println!("Unknown opcode: {:#X}", code),
                }
            }

            // Skip next instruction if VX and VY do not match
            0x9000 if (self.reg[ux(code)]) != (self.reg[uy(code)]) => self.pc += 2,
            0x9000 => (),

            // Set I to NNN
            0xA000 => self.i = nnn(code),

            // Jump to NNN + V0
            0xB000 => self.pc = nnn(code) + self.reg[0] as u16,

            // Set VX to ((random number) & NN)
            0xC000 => self.reg[ux(code)] = random::<u8>() & nn(code) as u8,

            // Draw sprite (https://en.wikipedia.org/wiki/CHIP-8#Opcode_table)
            0xD000 => {
                self.reg[0xF] = 0;

                let x = self.reg[us(x(code))] as u16;
                let y = self.reg[us(y(code))] as u16;

                for row in 0..n(code) {
                    let line = self.memory[us(self.i + row)];
                    for column in 0..8 {
                        let pixel = (line & (1 << (7 - column))) != 0;
                        let idx = us(x + column as u16 + ((y + row) * 64));
                        if idx >= self.gfx.len() {
                            continue;
                        }

                        if self.gfx[idx] && pixel {
                            self.reg[0xF] = 1;
                        }
                        self.gfx[idx] ^= pixel;
                    }
                }
            }

            0xE000 => {
                match code & 0x00FF {
                    // Skip next instruction if key VX is pressed
                    0x009E if is_pressed(KEYS[us8(self.reg[ux(code)])]) => self.pc += 2,
                    0x009E => (),

                    // Skip next instruction if key VX is not pressed
                    0x00A1 if !is_pressed(KEYS[us8(self.reg[ux(code)])]) => self.pc += 2,
                    0x00A1 => (),

                    _ => println!("Unknown opcode: {:#X}", code),
                }
            }

            0xF000 => {
                match code & 0x00FF {
                    // Set X to the delay timer
                    0x0007 => self.reg[ux(code)] = self.delay_timer,

                    // Halt until key press; then store in VX
                    0x000A => {
                        // See if a key is pressed and set RegF if so
                        let key = KEYS.iter().enumerate().find(|(_, key)| is_pressed(**key));
                        if let Some((idx, _)) = key {
                            self.reg[ux(code)] = idx as u8;
                        } else {
                            // If not, just reset the PC to make the emu keep
                            // executing this instruction until a key is pressed
                            self.pc -= 2;
                        }
                    }

                    // Set delay timer to VX
                    0x0015 => self.delay_timer = self.reg[ux(code)],

                    // Set sound timer to VX
                    0x0018 => self.sound_timer = self.reg[ux(code)],

                    // Add VX to I
                    0x001E => self.i += self.reg[ux(code)] as u16,

                    // Set I to the location of the sprite of the character in VX
                    0x0029 => self.i = (self.reg[ux(code)] * 5) as u16, // (every character is 5 bytes long)

                    // Stores binary-coded representation of VX at (I until I+2)
                    0x0033 => {
                        let mut x = self.reg[ux(code)];
                        let a = (x / 100) as u8;
                        x -= a * 100;
                        let b = (x / 10) as u8;
                        x -= b * 10; 

                        self.memory[us(self.i)] = a;
                        self.memory[us(self.i + 1)] = b;
                        self.memory[us(self.i + 2)] = x as u8;
                    }

                    // Write all registers to memory, stating at I
                    0x0055 => {
                        for (i, reg) in self.reg.iter().take(ux(code)).enumerate() {
                            self.memory[us(self.i + (i as u16))] = *reg;
                        }
                        self.i += x(code) + 1; 
                    }

                    // Write to all registers from memory, stating at I
                    0x0065 => {
                        let range = us(self.i)..us(self.i + x(code));
                        for (i, dat) in self.memory[range].iter().enumerate() {
                            self.reg[i] = *dat;
                        }
                        self.i += x(code) + 1; 
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
            chip8.memory[i] = *byte;
        }

        chip8
    }
}

fn us(u: u16) -> usize {
    u as usize
}

fn us8(u: u8) -> usize {
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
