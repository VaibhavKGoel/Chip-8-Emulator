use std::env;
use std::fs::File;
use std::io::Read;

use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::keyboard::Keycode;


use chip8_core::*;

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;
// const TICKS_PER_FRAME: usize = 10;

fn draw_screen(emu: &Emu, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(70,97,106));
    canvas.clear();

    let screen_buf = emu.get_display();
    canvas.set_draw_color(Color::RGB(214, 162, 173));
    for(i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;
            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }
    canvas.present();
}

fn key2button(key: Keycode) -> Option<usize> {
    //I am really not a fan of this keyboard config so I think we should change this 
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
fn game_speed(path: &str) -> usize {
    let name = path.to_lowercase();

    match name.as_str() {
        n if n.contains("15puzzle") => 15,
        n if n.contains("blinky") => 25,
        n if n.contains("blitz") => 2,
        n if n.contains("brix") || n.contains("vbrix") => 10,
        n if n.contains("connect4") => 1,
        n if n.contains("guess") => 15,
        n if n.contains("hidden") => 1,
        n if n.contains("invaders") => 12,
        n if n.contains("kaleid") => 9,
        n if n.contains("maze") => 9,
        n if n.contains("merlin") => 9,
        n if n.contains("missile") => 10,
        n if n.contains("pong2") => 10,
        n if n.contains("pong") => 10,
        n if n.contains("puzzle") => 5,
        n if n.contains("syzygy") => 6,
        n if n.contains("tank") => 11,
        n if n.contains("tetris") => 10,
        n if n.contains("tictac") => 12,
        n if n.contains("ufo") => 11,
        n if n.contains("vers") => 15,
        n if n.contains("wipeoff") => 13,
        // default for other games
        _ => 10,
    }
}
fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        return;
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut chip8 = Emu::new();
    let mut rom = File::open(&args[1]).expect("Unable to open file");
    let mut buffer = Vec::new();

    rom.read_to_end(&mut buffer).unwrap();
    chip8.load(&buffer);

    let ticks_per_frame = game_speed(&args[1]);

    'gameloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit{..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    break 'gameloop;
                },
                Event::KeyDown {keycode: Some(key), ..} =>  {
                    if let Some(k) = key2button(key) {
                        chip8.set_key(k, true);
                    }
                },
                Event::KeyUp {keycode: Some(key), ..} => {
                    if let Some(k) = key2button(key) {
                        chip8.set_key(k, false);
                    }
                }
                _ => ()
            }
        }
        for _i in 0..ticks_per_frame {
            chip8.tick();
        }
        chip8.timer_ticks();
        draw_screen(&chip8, &mut canvas);
    }

}
