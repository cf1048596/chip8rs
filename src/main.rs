mod cpu; 
use cpu::Cpu;
use std::error::Error;
use crate::cpu::SCREEN_WIDTH;
use crate::cpu::SCREEN_HEIGHT;
use macroquad::prelude::*;
use std::fs::File;
use std::io::Read;

const PIXEL_SCALE : u32 = 10;
const CYCLES_PER_FRAME: u32 = 5;

#[macroquad::main("chip8rs")]
async fn main() -> Result<(), Box<dyn Error>> {

    let mut file_contents: Vec<u8> = Vec::new();
    let filename = "pong.rom";
    File::open(filename)?.read_to_end(&mut file_contents)?;
    //println!("Opened and read from file {} OK", cli.file);
    let mut cpu = Cpu::new(&file_contents);

    loop {
        cpu.keys = handle_input();
        for _ in 0..CYCLES_PER_FRAME {
            cpu.fetch();
            cpu.decode_and_exec();
        }

        //decrease timers
        if cpu.dt_register > 0 {
            cpu.dt_register -= 1;
        }
        if cpu.st_register > 0 {
            cpu.st_register -= 1;
        }

        //render framebuffer
        render_framebuffer(cpu.get_framebuffer());

        //cap fps (60 FPS)
        next_frame().await;
    }
    Ok(())
}

fn render_framebuffer(framebuffer: &[[bool; SCREEN_WIDTH]; SCREEN_HEIGHT]) {
    //clear screen to black
    clear_background(BLACK);

    //get current screen width & height
    let window_width = screen_width();
    let window_height = screen_height();

    //calculate the scaling factors to fit the framebuffer to the screen
    let pixel_width = window_width / SCREEN_WIDTH as f32;
    let pixel_height = window_height / SCREEN_HEIGHT as f32;

    for (y, row) in framebuffer.iter().enumerate() {
        for (x, &pixel) in row.iter().enumerate() {
            if pixel {
                let x_pos = x as f32 * pixel_width;
                let y_pos = y as f32 * pixel_height;

                //draw a rectangle for each "on" pixel
                draw_rectangle(x_pos, y_pos, pixel_width, pixel_height, WHITE);
            }
        }
    }
}

//handle input and update the keys array
fn handle_input() -> [bool; 16] {

    let mut keys_state = [false; 16];
        get_keys_released()
            .iter()
            .for_each
            (|key| { match key {
                KeyCode::Key1 => keys_state[1] = false,
                KeyCode::Key2 => keys_state[2] = false,
                KeyCode::Key3 => keys_state[3] = false,
                KeyCode::Key4 => keys_state[0xC] = false,
                KeyCode::Q => keys_state[4] = false,
                KeyCode::W => keys_state[5] = false,
                KeyCode::E => keys_state[6] = false,
                KeyCode::R => keys_state[0xD] = false,
                KeyCode::A => keys_state[7] = false,
                KeyCode::S => keys_state[8] = false,
                KeyCode::D => keys_state[9] = false,
                KeyCode::F => keys_state[0xE] = false,
                KeyCode::Z => keys_state[0xA] = false,
                KeyCode::X => keys_state[0x0] = false,
                KeyCode::C => keys_state[0xB] = false,
                KeyCode::V => keys_state[0xF] = false,
                _ => {},
                }
            });

        get_keys_down()
            .iter()
            .for_each
            (|key| { match key {
                KeyCode::Key1 => keys_state[1] = true,
                KeyCode::Key2 => keys_state[2] = true,
                KeyCode::Key3 => keys_state[3] = true,
                KeyCode::Key4 => keys_state[0xC] = true,
                KeyCode::Q => keys_state[4] = true,
                KeyCode::W => keys_state[5] = true,
                KeyCode::E => keys_state[6] = true,
                KeyCode::R => keys_state[0xD] = true,
                KeyCode::A => keys_state[7] = true,
                KeyCode::S => keys_state[8] = true,
                KeyCode::D => keys_state[9] = true,
                KeyCode::F => keys_state[0xE] = true,
                KeyCode::Z => keys_state[0xA] = true,
                KeyCode::X => keys_state[0x0] = true,
                KeyCode::C => keys_state[0xB] = true,
                KeyCode::V => keys_state[0xF] = true,
                _ => {},
                }
            });
        keys_state
}
