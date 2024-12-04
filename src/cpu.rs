#![allow(dead_code)]

use std::{fmt::write, usize};
use getrandom::*;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const MEMORY_SIZE: usize = 4096;
const NUM_REGISTERS: usize = 16;
const NUM_KEYS: usize = 16;
const FONT_DATA: [u8; 80] = 
    [ 0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
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
    pub keys: [bool; NUM_KEYS],
    framebuffer: [ [bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
    stack: [u16; MEMORY_SIZE],
    sp: u16,
    pc: u16,
    idx_register: u16,
    pub dt_register: u8,
    pub st_register: u8,
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
            keys: [false; NUM_KEYS],
            framebuffer: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            registers: [0; NUM_REGISTERS],
            stack: [0; MEMORY_SIZE], 
            sp: 0,
            pc: 0x200,
            dt_register: 0,
            st_register: 0,
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
        let nibble1 = ((self.current_insn & 0xF000) >> 12) as u8;
        let nibble2 = ((self.current_insn & 0x0F00) >> 8) as u8;
        let nibble3 = ((self.current_insn & 0x00F0) >> 4) as u8;
        let nibble4 = (self.current_insn & 0x000F) as u8;

        match (nibble1, nibble2, nibble3, nibble4) {
            (0, 0, 0xE, 0) => self.framebuffer = [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            (0, 0, 0xE, 0xE) => {
                self.pc = self.stack[usize::from(self.sp)];
            },
            (1, _, _, _) => self.pc = ((nibble2 as u16) << 8) | ((nibble3 as u16) << 4) | (nibble4 as u16),
            (2, _, _, _) => {
                self.sp+=1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = ((nibble2 as u16) << 8) | ((nibble3 as u16) << 4) | (nibble4 as u16);
            },
            (3, _, _, _) => {
                if self.registers[nibble2 as usize] == (((nibble3 as u16) << 4) | (nibble4 as u16)) as u8 {
                    self.pc+=2;
                }
            },
            (4, _, _, _) => {
                if self.registers[nibble2 as usize] != (((nibble3 as u16) << 4) | (nibble4 as u16)) as u8 {
                    self.pc+=2;
                }
            },
            (5, _, _, 0) => {
                if self.registers[nibble2 as usize] == self.registers[nibble3 as usize] {
                    self.pc+=2;
                }
            },
            (6, 0..=15, _, _) => self.registers[nibble2 as usize] = (nibble3 << 4) | nibble4,
            (7, 0..=15, _, _) => {
                let nn = (nibble3 << 4) | nibble4;
                self.registers[nibble2 as usize] = self.registers[nibble2 as usize].wrapping_add(nn);
            },
            (8, _, _, 0) => {
                self.registers[nibble2 as usize] = self.registers[nibble3 as usize];
            },
            (8, _, _, 1) => {
                self.registers[nibble2 as usize] |= self.registers[nibble3 as usize];
            },
            (8, _, _, 2) => {
                self.registers[nibble2 as usize] &= self.registers[nibble3 as usize];
            },
            (8, _, _, 3) => {
                self.registers[nibble2 as usize] ^= self.registers[nibble3 as usize];
            },
            (8, _, _, 4) => {
                let sum = self.registers[nibble2 as usize] as u16 + self.registers[nibble3 as usize] as u16;
                self.registers[0xF] = if sum > 0xFF { 1 } else { 0 }; // set vf (carry flag)
                self.registers[nibble2 as usize] = (sum & 0xFF) as u8; // store lower 8 bits in vx
            },
            (8, _, _, 5) => {
                match (self.registers[nibble2 as usize], self.registers[nibble3 as usize]) {
                    (vx, vy) if vx > vy => self.registers[0xF] = 1,
                    (vx, vy) if vy > vx => self.registers[0xF] = 0,
                    _ => {}, 
                }
                self.registers[nibble2 as usize] = self.registers[nibble2 as usize].wrapping_sub(self.registers[nibble3 as usize]);
            },
            (8, _, _, 6) => {
                self.registers[0xF] = self.registers[nibble2 as usize] & 0x01;
                self.registers[nibble2 as usize] /= 2;
            }
            (8, _, _, 7) => {
                match (self.registers[nibble2 as usize], self.registers[nibble3 as usize]) {
                    (vx, vy) if vx > vy => self.registers[0xF] = 0,
                    (vx, vy) if vy > vx => self.registers[0xF] = 1,
                    _ => {},
                }
                let sum = self.registers[nibble3 as usize].wrapping_sub(self.registers[nibble2 as usize]);
                self.registers[nibble2 as usize] = sum;
            },
            (8, _, _, 0xE) => {
                self.registers[0xF] = self.registers[nibble2 as usize] & 0x80;
                self.registers[nibble2 as usize] = self.registers[nibble2 as usize].wrapping_mul(2);
            },
            (9, _, _, 0) => {
                if self.registers[nibble2 as usize] != self.registers[nibble3 as usize] {
                    self.pc+=2;
                }
            },
            (0xA, _, _, _) => self.idx_register = ((nibble2 as u16) << 8) | ((nibble3 as u16) << 4) | (nibble4 as u16),
            (0xB, _, _, _) => self.pc = u16::from(self.registers[0_usize]) + (((nibble2 as u16) << 8) | ((nibble3 as u16) << 4) | (nibble4 as u16)),
            (0xC, _, _, _) => {
                let mut buf = [0u8; 1]; //create buffer to hold one byte
                getrandom(&mut buf).unwrap();
                let rand_byte : u8 = buf[0];
                self.registers[nibble2 as usize] = rand_byte & ((nibble3 << 4) | (nibble4))
            },
            (0xD, _, _, _) => self.update_framebuffer(nibble2, nibble3, nibble4),
            (0xE, _, 9, 0xE) => {
                if self.keys[self.registers[nibble2 as usize] as usize] {
                    self.pc+=2;
                }
            },
            (0xE, _, 0xA, 1) => {
                if !self.keys[self.registers[nibble2 as usize] as usize] {
                    self.pc+=2;
                }
            },
            (0xF, _, 0, 7) => self.registers[nibble2 as usize] = self.dt_register,
            (0xF, _, 0, 0xA) => {
               if let Some(index) = self.keys.iter().position(|&key| key) {
                   self.registers[nibble2 as usize] = index as u8;
               } else {
                   self.pc-=2;
               }
            },
            (0xF, _, 1, 5) => self.dt_register = self.registers[nibble2 as usize],
            (0xF, _, 1, 8) => self.st_register = self.registers[nibble2 as usize],
            (0xF, _, 1, 0xE) => self.idx_register += u16::from(self.registers[nibble2 as usize]),
            (0xF, _, 2, 9) => self.idx_register = u16::from(self.registers[nibble2 as usize])*0x05_u16,
            (0xF, _, 3, 3) => {
                let vx = self.registers[nibble2 as usize];
                let h: u8 = vx/100;
                let t: u8 = (vx % 100) / 10;
                let o: u8 = vx % 10;
                self.memory[self.idx_register as usize] = h;
                self.memory[self.idx_register as usize+1] = t;
                self.memory[self.idx_register as usize+2] = o;
            }
            (0xF, _, 5, 5) => {
                for reg in 0..=nibble2 as usize {
                    self.memory[self.idx_register as usize + reg] = self.registers[reg];
                }
            }
            (0xF, _, 6, 5) => {
                for reg in 0..=nibble2 as usize {
                    self.registers[reg] = self.memory[self.idx_register as usize + reg];
               }
            },
            _ => todo!("Instruction not implemented: {:#04X}", self.current_insn),
        }
    }

    fn update_framebuffer(&mut self, x: u8, y: u8, n: u8) {
        let vx = self.registers[usize::from(x)] as usize; //xcoord
        let vy = self.registers[usize::from(y)] as usize; //ycoord
        let mut vf = 0; //carry flag

        for row in 0..n {

            //iter over each column in sprite
            let sprite_byte = self.memory[self.idx_register as usize + row as usize];

            for bit in 0..8 { //iter over each bit
                let pixel_x = (vx + bit) % SCREEN_WIDTH; //wrap horizontally
                let pixel_y = (vy + row as usize) % SCREEN_HEIGHT; //wrap vertically

                let sprite_pixel = (sprite_byte >> (7 - bit)) & 1; //get pixel either 0 or 1
                //xor the current framebuffer pixel with the sprite pixel
                let existing_pixel = self.framebuffer[pixel_y][pixel_x];
                let new_pixel = existing_pixel ^ (sprite_pixel != 0);

                //update framebuffer
                self.framebuffer[pixel_y][pixel_x] = new_pixel;

                //check for pixel collision (turning off a pixel)
                if existing_pixel && !new_pixel {
                    vf = 1; //set carry flag
                }
            }
        }
        //set vf register to current state of carry flag
        self.registers[NUM_REGISTERS - 1] = vf;
    }

    pub fn draw_framebuffer_console(&self) {
        for row in self.framebuffer.iter() {
            for &pixel in row.iter() {
                if pixel {
                    print!("#");
                } else {
                    print!(" ");
                }
            }
            println!();
        }
    }

    pub fn get_framebuffer(&self) -> &[[bool; SCREEN_WIDTH]; SCREEN_HEIGHT] {
        &self.framebuffer
    }
}
