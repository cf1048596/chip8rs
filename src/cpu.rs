#![allow(dead_code)]

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const MEMORY_SIZE: usize = 4096;
const NUM_REGISTERS: usize = 16;
const FONT_DATA: [u8; 80] = [ 0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
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
0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Cpu {
    memory: [u8; MEMORY_SIZE],
    framebuffer: [ [bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
    stack: Vec<u8>,
    sp: u16,
    pc: u16,
    idx_register: u16,
    registers: [u8; NUM_REGISTERS],
    current_insn: u16,
}

impl Cpu {
    pub fn new(read_rom: &[u8]) -> Self {
        let mut memory : [u8; 4096]= [0; MEMORY_SIZE];
        //copy font info into first 512 bytes
        memory[0..FONT_DATA.len()].copy_from_slice(&FONT_DATA);
        //copy rom memory into address 0x200 in hexadecimal/ 512 in binary
        memory[512..(512 + read_rom.len())].copy_from_slice(read_rom);
        //our starting address for the program counter is 512/0x200 where our binary is read into
        Self {
            memory,
            framebuffer: [ [false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            registers: [0; NUM_REGISTERS],
            stack: Vec::new(),
            sp: 0,
            pc: 0x200,
            idx_register: 0,
            current_insn: 0,
        }
    }

    pub fn fetch(&mut self) {
        //chip8 is big endian for reference so the first byte is the leading part of the insn
        let high_byte = self.memory[usize::from(self.pc)] as u16;
        let low_byte = self.memory[usize::from(self.pc) + 1] as u16;
        self.current_insn = (high_byte << 8) | low_byte;
        self.pc+=2;
    }

    pub fn decode_and_exec(&mut self) {
        let nibble1 = ((self.current_insn &0xF000) >> 12) as u8;
        let nibble2 = ((self.current_insn &0x0F00) >> 8) as u8;
        let nibble3 = ((self.current_insn &0x00F0) >> 4) as u8;
        let nibble4 = (self.current_insn &0x000F) as u8;
        match (nibble1, nibble2, nibble3, nibble4) {
            (0,0, 0b1110, 0) => self.framebuffer = [[false; SCREEN_WIDTH]; SCREEN_HEIGHT], 
            (0, 0, 0b1110, 0b1110) => todo!(),
            (1, 0..=15, 0..=15, 0..=15) => self.pc = (nibble2+nibble3+nibble4).into(),
            (6, 0..=15, _, _) => self.registers[nibble2 as usize] = nibble3+nibble4,
            (7, 0..=15, _, _) => self.registers[nibble2 as usize] = self.registers[nibble2 as usize].wrapping_add(nibble3 + nibble4),
            (10, 0..=15, 0..=15, 0..=15) => self.idx_register = (nibble2+nibble3+nibble4).into(),
            (_d, _, _, _) => self.update_framebuffer(nibble2, nibble3, nibble4),
        }
    }

    fn update_framebuffer(&mut self, x: u8, y: u8, n: u8) {
        // Get the register values for vx and vy from the second and third nibble of the DXYN instruction
        let vx = self.registers[usize::from(x)] as u32;
        let vy = self.registers[usize::from(y)] as u32;

        // Iterate over the sprite rows
        for row in 0..n {
            let bits = self.memory[usize::from(self.idx_register + u16::from(row))];
            let cy = (vy + row as u32) % SCREEN_HEIGHT as u32; // Y-coordinate, wrapping around if needed

            // Iterate over each column in the sprite (8 bits per row)
            for col in 0..8 {
                let cx = (vx + col as u32) % SCREEN_WIDTH as u32; // X-coordinate, wrapping around if needed
                let mut curr_col = self.framebuffer[cy as usize][cx as usize]; // Get the current pixel value
                let bit_val = bits & (0x80 >> col); // Check if the current bit is set

                if bit_val > 0 {
                    // If the pixel is already on, set the collision flag
                    if curr_col {
                        self.registers[NUM_REGISTERS - 1] = 1; // Set VF (collision flag) to 1
                    }
                    // Toggle the pixel
                    curr_col = !curr_col;
                }

                // Update the framebuffer with the new value
                self.framebuffer[cy as usize][cx as usize] = curr_col;
            }
        }
    }

    pub fn draw_framebuffer_console(&self) {
        // Iterate through each row of the framebuffer
        for row in self.framebuffer.iter() {
            // Iterate through each pixel in the row
            for &pixel in row.iter() {
                // Print '#' for on (true) pixels, and a space for off (false) pixels
                if pixel {
                    print!("#");
                } else {
                    print!(" ");
                }
            }
            println!(); // Move to the next line after each row
        }
    }
}

