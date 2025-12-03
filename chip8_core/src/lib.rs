use rand::random;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use rodio::source::SineWave;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80 // F
];

pub struct Emu {
    program_counter: u16,
    ram:[u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    stack_pointer: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    delay_timer: u8,
    sound_timer: u8,
    #[allow(dead_code)]
    audio_stream: Option<OutputStream>,
    #[allow(dead_code)]
    audio_handle: Option<OutputStreamHandle>,
    beep_sink: Option<Sink>,
}

impl Emu {
    pub fn new() -> Self {
        let (stream, handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&handle).unwrap();
        sink.append(SineWave::new(440.0));
        sink.pause(); 
        let mut new_emu = Self {
            program_counter: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            stack_pointer: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            delay_timer: 0,
            sound_timer: 0,
            audio_stream: Some(stream),
            audio_handle: Some(handle),
            beep_sink: Some(sink),
        };

        for i in 0..FONTSET_SIZE {
            new_emu.ram[i] = FONTSET[i];
        }

        new_emu
    }

    fn push(&mut self, val: u16) {
        self.stack[self.stack_pointer as usize] = val;
        self.stack_pointer += 1;
    }
    fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }

    pub fn reset(&mut self) {
        self.program_counter = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.stack_pointer = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.delay_timer = 0;
        self.sound_timer = 0;
        for i in 0..FONTSET_SIZE {
            self.ram[i] = FONTSET[i];
        }
    }

    pub fn tick(&mut self) {
        let opcode: u16 = self.fetch();
        self.execute(opcode);
    }

    fn fetch(&mut self) -> u16 {
        let higher = (self.ram[self.program_counter as usize] as u16) << 8;
        let lower = self.ram[(self.program_counter + 1) as usize] as u16;
        let op: u16 = higher | lower;

        self.program_counter += 2;
        
        op
    }

    fn execute(&mut self, opcode: u16) {
        let digit_one = (opcode >> 12) & 0x0F;
        let digit_two = (opcode >> 8) & 0x0F;
        let digit_three = (opcode >> 4) & 0x0F;
        let digit_four = opcode & 0x0F;
        match (digit_one, digit_two, digit_three, digit_four) {
            (0, 0, 0, 0) => return,
            (0, 0, 0xE, 0) => self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            (0, 0, 0xE, 0xE) => self.program_counter = self.pop(),
            (1, _, _, _) => self.program_counter = opcode & 0xFFF,
            (2, _, _, _) => {
                self.push(self.program_counter);
                self.program_counter = opcode & 0xFFF;
            },
            (3, _, _, _) => {
                if (self.v_reg[digit_two as usize]) == (opcode & 0x0FF) as u8 {
                    self.program_counter += 2;
                }
            },
            (4, _, _, _) => {
                if (self.v_reg[digit_two as usize]) != (opcode & 0x0FF) as u8 {
                    self.program_counter += 2;
                }
            },
            (5, _, _, 0) => {
                if (self.v_reg[digit_two as usize]) == (self.v_reg[digit_three as usize]) {
                    self.program_counter += 2;
                }
            },
            (6, _, _, _) => self.v_reg[digit_two as usize] = (opcode & 0xFF) as u8,
            (7, _, _, _) => self.v_reg[digit_two as usize] = self.v_reg[digit_two as usize].wrapping_add((opcode & 0xFF) as u8),
            (8, _, _, 0) => self.v_reg[digit_two as usize] = self.v_reg[digit_three as usize],
            (8, _, _, 1) => self.v_reg[digit_two as usize] |= self.v_reg[digit_three as usize],
            (8, _, _, 2) => self.v_reg[digit_two as usize] &= self.v_reg[digit_three as usize],
            (8, _, _, 3) => self.v_reg[digit_two as usize] ^= self.v_reg[digit_three as usize],
            (8, _, _, 4) => {
                let (sum, carry) = self.v_reg[digit_two as usize].overflowing_add(self.v_reg[digit_three as usize]);
                self.v_reg[digit_two as usize] = sum;
                self.v_reg[0xF] = if carry {1} else {0};
            },
            (8, _, _, 5) => {
                let (sum, negative_carry) = self.v_reg[digit_two as usize].overflowing_sub(self.v_reg[digit_three as usize]);
                self.v_reg[digit_two as usize] = sum;
                self.v_reg[0xF] = if negative_carry {0} else {1};
            },
            (8, _, _, 6) => {
                let dropped_bit = self.v_reg[digit_two as usize] & 1;
                self.v_reg[digit_two as usize] = self.v_reg[digit_two as usize] >> 1;
                self.v_reg[0xF] = dropped_bit;
            },
            (8, _, _, 7) => {
                let (sum, negative_carry) = self.v_reg[digit_three as usize].overflowing_sub(self.v_reg[digit_two as usize]);
                self.v_reg[digit_two as usize] = sum;
                self.v_reg[0xF] = if negative_carry {0} else {1};
            },
            (8, _, _, 0xE) => {
                let dropped_bit = if self.v_reg[digit_two as usize] > 127 {1} else {0};
                self.v_reg[digit_two as usize] = self.v_reg[digit_two as usize] << 1;
                self.v_reg[0xF] = dropped_bit;
            },
            (9, _, _, 0) => {
                if (self.v_reg[digit_two as usize]) != (self.v_reg[digit_three as usize]) {
                    self.program_counter += 2;
                }
            },
            (0xA, _, _, _) => self.i_reg = opcode & 0xFFF,
            (0xB, _, _, _) => self.program_counter = (opcode & 0xFFF) + (self.v_reg[0] as u16),
            (0xC, _, _, _) => {
                let rand_val: u8 = random();
                self.v_reg[digit_two as usize] = rand_val & (opcode & 0xFF) as u8;
            },
            (0xD, _, _, _) => {
                let x_coord = self.v_reg[digit_two as usize] as u16;
                let y_coord = self.v_reg[digit_three as usize] as u16;
                let rows = digit_four;
                let mut flipped = false;
                
                for y_line in 0..rows {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];
                    for x_line in 0..8 {
                        if(pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;
                            let idx = x + SCREEN_WIDTH * y;
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            },
            (0xE, _, 9, 0xE) => {
                let vx = self.v_reg[digit_two as usize];
                let key = self.keys[vx as usize];
                if key {
                    self.program_counter += 2;
                }
            },
            (0xE, _, 0xA, 1) => {
                let vx = self.v_reg[digit_two as usize];
                let key = self.keys[vx as usize];
                if !key {
                    self.program_counter += 2;
                }
            },
            (0xF, _, 0, 7) => self.v_reg[digit_two as usize] = self.delay_timer,
            (0xF, _, 0, 0xA) => {
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        pressed = true;
                        self.v_reg[digit_two as usize] = i as u8;
                        break;
                    }
                }
                if !pressed {
                    self.program_counter -= 2;
                }
            },
            (0xF, _, 1, 5) => self.delay_timer = self.v_reg[digit_two as usize],
            (0xF, _, 1, 8) => self.sound_timer = self.v_reg[digit_two as usize],
            (0xF, _, 1, 0xE) => self.i_reg = self.i_reg.wrapping_add(self.v_reg[digit_two as usize] as u16),
            (0xF, _, 2, 9) => self.i_reg = 5 * self.v_reg[digit_two as usize] as u16,
            (0xF, _, 3, 3) => {
                self.ram[self.i_reg as usize] = self.v_reg[digit_two as usize] / 100;
                self.ram[self.i_reg as usize + 1] = (self.v_reg[digit_two as usize] / 10) % 10;
                self.ram[self.i_reg as usize + 2] = self.v_reg[digit_two as usize] % 10;
            },
            (0xF, _, 5, 5) => {
                for x in 0..=digit_two {
                    self.ram[self.i_reg as usize + x as usize] = self.v_reg[x as usize];
                }
            },
            (0xF, _, 6, 5) => {
                for i in 0..=(digit_two as usize) {
                    self.v_reg[i] = self.ram[(self.i_reg as usize) + i];
                }
            },
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", opcode),
        }
    }

    pub fn timer_ticks(&mut self) { 
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
            if let Some(sink) = &self.beep_sink {
                sink.play();
            }
        } else {
            if let Some(sink) = &self.beep_sink {
                sink.pause();
            }
        }
    }
    
    // pub fn beep(&self) {
    //      let (_stream, handle) = OutputStream::try_default().unwrap(); 
    //      let sink = Sink::try_new(&handle).unwrap(); 
    //      sink.append(SineWave::new(440.0).take_duration(std::time::Duration::from_millis(100))); 
    //      sink.detach(); 
    // }
    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn set_key(&mut self, key: usize, press: bool) {
        self.keys[key] = press;
    }

    pub fn load(&mut self, bytes: &[u8]) {
        let start = START_ADDR as usize;
        self.ram[start..(start + bytes.len())].copy_from_slice(bytes);
    }
    
}