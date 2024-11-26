mod cpu; 
use cpu::Cpu;
use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::Read;

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
    loop {
        cpu.fetch();
        cpu.decode_and_exec();
        cpu.draw_framebuffer_console();
    }
}
