mod cpu; 
use cpu::Cpu;
use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use crate::cpu::SCREEN_WIDTH;
use crate::cpu::SCREEN_HEIGHT;

extern crate sdl2;

const PIXEL_SCALE : u32 = 10;

#[derive(Parser)]
#[command(name = "chip8-rs")]
#[command(version = "1.0")]
#[command(about = "A chip8 emulator written in rust", long_about = None)]
struct Cli {
    #[arg(long, short)]
    file : String,
}
fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    println!("File name: {}", cli.file);

    let mut file_contents: Vec<u8> = Vec::new();
    File::open(&cli.file)?.read_to_end(&mut file_contents)?;
    println!("Opened and read from file {} OK", cli.file);
    println!("File size: {}", file_contents.len());

    let mut cpu = Cpu::new(&file_contents);
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(
            "CHIP-8 Emulator",
            (SCREEN_WIDTH as u32) * PIXEL_SCALE,
            (SCREEN_HEIGHT as u32) * PIXEL_SCALE,
        )
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;
    let frame_duration = std::time::Duration::from_millis(16); // 60 Hz

    'running: loop {
        let frame_start = std::time::Instant::now();

        // Handle SDL2 events (e.g., window close)
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        // Execute one CPU cycle
        cpu.fetch();
        cpu.decode_and_exec();

        // Render the framebuffer
        render_framebuffer(&mut canvas, cpu.get_framebuffer());

        // Cap the frame rate
        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
    Ok(())
}


fn render_framebuffer(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, framebuffer: &[[bool; SCREEN_WIDTH]; SCREEN_HEIGHT]) {
    canvas.set_draw_color(sdl2::pixels::Color::BLACK);
    canvas.clear();

    canvas.set_draw_color(sdl2::pixels::Color::WHITE);
    for (y, row) in framebuffer.iter().enumerate() {
        for (x, &pixel) in row.iter().enumerate() {
            if pixel {
                let rect = sdl2::rect::Rect::new(
                    (x as u32 * PIXEL_SCALE) as i32,
                    (y as u32 * PIXEL_SCALE) as i32,
                    PIXEL_SCALE,
                    PIXEL_SCALE,
                );
                canvas.fill_rect(rect).unwrap();
            }
        }
    }
    canvas.present();
}


