use std::ops::{Deref, DerefMut, Index, IndexMut, Range};

use font::FONT;

mod font;

/// Registers, often refered as `V0-VF`
#[derive(Debug, Default)]
pub struct Registers {
	regs: [u8; Self::COUNT],
}
impl Registers {
	pub const COUNT: usize = 16;
}

impl Index<Range<u8>> for Registers {
	type Output = [u8];

	fn index(&self, index: Range<u8>) -> &Self::Output {
		&self.regs[index.start as usize..index.end as usize]
	}
}
impl Index<u8> for Registers {
	type Output = u8;

	fn index(&self, index: u8) -> &Self::Output {
		&self.regs[index as usize]
	}
}
impl IndexMut<u8> for Registers {
	fn index_mut(&mut self, index: u8) -> &mut Self::Output {
		&mut self.regs[index as usize]
	}
}
impl Deref for Registers {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.regs
	}
}
impl DerefMut for Registers {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.regs
	}
}

/// CHIP-8 emulator itself
#[derive(Debug)]
pub struct Emu {
	pub screen: [bool; Self::SCREEN_BUF_SIZE],

	/// Program counter (PC), points to the current instruction in the memory
	pub pc: u16,
	/// Stack pointer (SP), points to the topmost level of the stack
	pub sp: u16,
	/// Memory pointer (I), points to the current address in the memory, used by some instructions
	pub index: u16,
	/// Stack, list of addresses to jump to when `return` instruction is called
	pub stack: [u16; Self::STACK_SIZE],

	/// Current program data
	pub program: [u8; Self::PROGRAM_SIZE],
	pub memory: [u8; Self::MEMORY_SIZE],
	pub regs: Registers,
	/// Delay timer (DT)
	pub delay_timer: u8,
	/// Sound timer (ST)
	pub sound_timer: u8,

	/// List of pressed keys
	pub pressed_keys: [bool; Self::KEYS_COUNT],
	/// Current instruction
	pub cur_ins: (u8, u8),
	/// If `Some(register)`, stop execution, wait for a keypress and write the pressed key into
	/// specified register, otherwise continue the execution
	pub wait_for_keypress: Option<u8>,
	/// Whether a pressed key check occured
	pub key_checked: bool,
	/// Steps per frame multiplier
	pub speed: f32,
	/// Whether any kind of error has occurred
	pub error: bool,
	/// CPU heat level in range `0.0..=1.0`
	pub heat: f32,
}
impl Default for Emu {
	fn default() -> Self {
		Self {
			screen: [false; Self::SCREEN_BUF_SIZE],

			pc: 0,
			sp: 0,
			index: 0,
			stack: [0; Self::STACK_SIZE],

			program: [0; Self::PROGRAM_SIZE],
			memory: [0; Self::MEMORY_SIZE],
			regs: Registers::default(),
			delay_timer: 0,
			sound_timer: 0,

			pressed_keys: [false; Self::KEYS_COUNT],
			cur_ins: (0, 0),
			wait_for_keypress: None,
			key_checked: false,
			speed: 1.0,
			error: false,
			heat: 0.0,
		}
	}
}
impl Emu {
	pub const SCREEN_WIDTH: usize = 64;
	pub const SCREEN_HEIGHT: usize = 32;
	/// Size of the screen data in bytes
	pub const SCREEN_BUF_SIZE: usize = Self::SCREEN_WIDTH * Self::SCREEN_HEIGHT;

	/// Starting address of programs in the memory
	/// NOTE: not all of the CHIP-8 programs start at this address for some reason
	pub const PROGRAM_START_ADDR: usize = 0x200;
	pub const MEMORY_SIZE: usize = 12288;
	/// Max program size
	pub const PROGRAM_SIZE: usize = Self::MEMORY_SIZE - Self::PROGRAM_START_ADDR;
	pub const STACK_SIZE: usize = 16;
	pub const KEYS_COUNT: usize = 16;

	/// Default execution speed
	pub const STEPS_PER_FRAME: u8 = 20;
	pub const MIN_SPEED: f32 = 0.0;
	pub const MAX_SPEED: f32 = 5.0;

	/// Each char in the font is 5 bytes long
	pub const CHAR_HEIGHT: u16 = 5;

	/// CPU hot level
	const HOT_LEVEL: f32 = 0.05;
	const CIRITICAL_HEAT_LEVEL: f32 = 0.99;
	/// How much the heat level will increase on each jump
	const HEAT_SPEED: f32 = 0.0015 / 20.0;
	/// How much the heat level will decrease on each frame
	const COOL_SPEED: f32 = 0.001;

	/// Load the CHIP-8 program into the memory
	/// Crops program data if it larger than [`Emu::PROGRAM_SIZE`]
	pub fn load(&mut self, program: &[u8]) {
		// Store program
		let len = program.len().min(Self::PROGRAM_SIZE);
		self.program[..len].copy_from_slice(&program[..len]);

		self.setup();
	}
	/// Reset everything, load default font and [`Emu::program`] to the memory
	pub fn setup(&mut self) {
		self.reset();

		// Load the font first
		self.memory[..FONT.len()].copy_from_slice(FONT);

		// Load program
		let start = Self::PROGRAM_START_ADDR;
		let len = self.program.len();

		self.memory[start..start + len].copy_from_slice(&self.program);

		self.pc = start as u16;
	}
	/// Reset everything
	pub fn reset(&mut self) {
		*self = Self {
			program: self.program,
			speed: self.speed,
			heat: self.heat,
			..Default::default()
		};
	}

	pub fn update(&mut self) {
		self.update_timers();

		// Execute current program with speed of 20 instructions per frame
		let n = (Self::STEPS_PER_FRAME as f32 * self.speed).round() as u8;
		for _ in 0..n {
			self.step();
		}
	}
	/// Execute the current instruction and increment the program counter by 2
	pub fn step(&mut self) {
		// Wrap program counter to the program start if it reached the end of the memeory
		// Program never stops its execution
		if self.pc >= self.memory.len() as u16 - 1 {
			self.pc = Self::PROGRAM_START_ADDR as u16;
		}

		// Wait for a keypress
		if self.wait_for_keypress.is_some() {
			return;
		}

		let pc = self.pc as usize;
		self.cur_ins = (self.memory[pc], self.memory[pc + 1]);

		self.pc += 2;
		self.execute();
	}
	/// Execute instruction
	/// Returns whether the program counter should increment or not
	fn execute(&mut self) {
		// (0xAA << 8) | 0xBB = 0xAABB
		let ins = {
			let a = self.cur_ins.0 as u16;
			let b = self.cur_ins.1 as u16;
			(a << 8) | b
		};

		// Split instruction into 4 nibbles
		let a = ((ins & 0xF000) >> 12) as u8;
		let b = ((ins & 0x0F00) >> 8) as u8;
		let c = ((ins & 0x00F0) >> 4) as u8;
		let d = (ins & 0x000F) as u8;

		let x = b;
		let y = c;
		let n = d; // nibble
		let addr = ins & 0x0FFF; // nnn
		let byte = (ins & 0x00FF) as u8; // kk

		macro_rules! skip_if {
			($condition:expr) => {
				if $condition {
					self.pc += 2;
				}
			};
		}

		match (a, b, c, d) {
			// Clear screen
			(0, 0, 0xe, 0) => self.screen.fill(false),
			// Draw sprite
			(0xd, _, _, _) => self.screen_draw(x, y, n),

			// Return
			(0, 0, 0xe, 0xe) => {
				let addr = self.stack_pop();
				self.jump(addr);
			}
			// Jump
			(1, _, _, _) => self.jump(addr),
			// Jump to `addr + V0`
			(0xb, _, _, _) => self.jump(addr + self.regs[0] as u16),
			// Call a subroutine
			(2, _, _, _) => {
				self.stack_push();
				self.jump(addr);
			}

			// Skip if `Vx == byte`
			(3, _, _, _) => skip_if!(self.regs[x] == byte),
			// Skip if `Vx != byte`
			(4, _, _, _) => skip_if!(self.regs[x] != byte),
			// Skip if `Vx == Vy`
			(5, _, _, 0) => skip_if!(self.regs[x] == self.regs[y]),
			// Skip if `Vx != Vy`
			(9, _, _, 0) => skip_if!(self.regs[x] != self.regs[y]),

			// Wait for a keypress
			(0xf, _, 0, 0xa) => self.wait_for_keypress = Some(x),
			// Skip if `Vx == pressed key`
			(0xe, _, 9, 0xe) => skip_if!(self.is_key_pressed(self.regs[x])),
			// Skip if `Vx != pressed key`
			(0xe, _, 0xa, 1) => skip_if!(!self.is_key_pressed(self.regs[x])),

			// `Vx = byte`
			(6, _, _, _) => self.regs[x] = byte,
			// `Vx = Vx + byte`
			(7, _, _, _) => self.regs[x] = self.regs[x].wrapping_add(byte),
			// `Vx = Vy`
			(8, _, _, 0) => self.regs[x] = self.regs[y],
			// `Vx = Vx + Vy`
			(8, _, _, 4) => self.add_vx_vy(x, y),
			// `Vx = Vx - Vy`
			(8, _, _, 5) => self.sub_vx_vy(x, y),
			// `Vx = random & byte`
			(0xc, _, _, _) => self.set_rand(x, byte),

			// `Vx = Vx | Vy`
			(8, _, _, 1) => self.regs[x] |= self.regs[y],
			// `Vx = Vx & Vy`
			(8, _, _, 2) => self.regs[x] &= self.regs[y],
			// `Vx = Vx ^ Vy`
			(8, _, _, 3) => self.regs[x] ^= self.regs[y],

			// `Vx = Vx >> 1`
			(8, _, _, 6) => self.shift_right(x),
			// `Vx = Vy - Vx`
			(8, _, _, 7) => self.sub_vy_vx(y, x),
			// `Vx = Vx << 1`
			(8, _, _, 0xe) => self.shift_left(x),

			// `Vx = DT`
			(0xf, _, 0, 7) => self.regs[x] = self.delay_timer,
			// `DT = Vx`
			(0xf, _, 1, 5) => self.delay_timer = self.regs[x],
			// `ST = Vx`
			(0xf, _, 1, 8) => self.sound_timer = self.regs[x],

			// `I = addr`
			(0xa, _, _, _) => self.index = addr,
			// `I = I + Vx`
			(0xf, _, 1, 0xe) => self.inc_index(self.regs[x] as u16),
			// `I = Vx * 5`
			(0xf, _, 2, 9) => self.set_index_sprite_addr(x),
			// Store Binary Coded Decimal (BCD) representation of `Vx`
			(0xf, _, 3, 3) => self.store_bcd(x),
			// Store `V0..=Vx` to `I..=I+x`
			(0xf, _, 5, 5) => self.store_through(x),
			// Read from `I..=I+x` to `V0..=Vx`
			(0xf, _, 6, 5) => self.read_through(x),

			(0, _, _, _) => (/* no-op */),
			_ => self.error = true,
		}
	}
	/// Decrement timers
	pub fn update_timers(&mut self) {
		self.delay_timer = self.delay_timer.saturating_sub(1);
		self.sound_timer = self.sound_timer.saturating_sub(1);
	}

	pub fn heat_up(&mut self) {
		self.heat += Self::HEAT_SPEED;
		self.heat = self.heat.min(1.0);
	}
	pub fn cool_down(&mut self, multiplier: f32) {
		self.heat -= Self::COOL_SPEED * multiplier;
		self.heat = self.heat.max(0.0);
	}

	pub fn set_speed(&mut self, speed: f32) {
		self.speed = speed.clamp(Self::MIN_SPEED, Self::MAX_SPEED);
	}
	pub fn inc_speed(&mut self, diff: f32) {
		self.set_speed(self.speed + diff);
	}

	pub fn set_pressed_key(&mut self, key: u8, is_pressed: bool, just_pressed: bool) {
		self.pressed_keys[key as usize] = is_pressed;

		if just_pressed {
			if let Some(x) = self.wait_for_keypress {
				self.regs[x] = key;
				self.wait_for_keypress = None;
			}
		}
	}
	pub fn is_key_pressed(&mut self, key: u8) -> bool {
		self.key_checked = true;
		self.pressed_keys[key as usize % Emu::KEYS_COUNT]
	}

	pub fn jump(&mut self, addr: u16) {
		self.heat_up();
		self.pc = addr;
	}

	pub fn mem_get(&mut self, addr: u16) -> u8 {
		self.memory[addr as usize]
	}
	pub fn inc_index(&mut self, addr: u16) {
		self.index = (self.index + addr) % self.memory.len() as u16;
	}

	/// Push current program counter to the stack
	pub fn stack_push(&mut self) {
		self.stack[self.sp as usize] = self.pc;
		self.sp += 1;

		// Wrap around on overflow because it's fun
		if self.sp as usize >= self.stack.len() {
			self.sp = 0;
			self.error = true;
		}
	}
	/// Pop the last address in the stack
	pub fn stack_pop(&mut self) -> u16 {
		// Return random address on underflow because it's fun too
		if self.sp == 0 {
			self.error = true;
			let addr = (quad_rand::rand() % self.memory.len() as u32) as u16;
			return addr;
		}

		self.sp -= 1;
		self.stack[self.sp as usize]
	}

	pub fn screen_draw(&mut self, x: u8, y: u8, n: u8) {
		let vx = self.regs[x] as usize;
		let vy = self.regs[y] as usize;
		let mut overlap = false;

		for row in 0..n as usize {
			let mut byte = self.mem_get(self.index + row as u16);
			let py = (vy + row) % Self::SCREEN_HEIGHT;

			for col in 0..8 {
				if byte & 0x80 > 0 {
					let px = (vx + col) % Self::SCREEN_WIDTH;
					let idx = px + py * Self::SCREEN_WIDTH;

					if self.screen[idx] {
						self.screen[idx] = false;
						overlap = true;
					} else {
						self.screen[idx] = true;
					}
				}

				byte <<= 1;
			}
		}

		self.regs[0xf] = overlap as u8;
	}

	/// `Vx += Vy; VF = overflow`
	pub fn add_vx_vy(&mut self, x: u8, y: u8) {
		let (byte, overflow) = self.regs[x].overflowing_add(self.regs[y]);
		self.regs[x] = byte;
		self.regs[0xF] = overflow as u8;
	}
	/// `Vx -= Vy; VF = NOT underflow`
	pub fn sub_vx_vy(&mut self, x: u8, y: u8) {
		let vx = self.regs[x];
		let vy = self.regs[y];

		self.regs[0xF] = (vx > vy) as u8;
		self.regs[x] = vx.wrapping_sub(vy);
	}
	/// `Vx = Vy - Vx; VF = NOT underflow`
	pub fn sub_vy_vx(&mut self, x: u8, y: u8) {
		let vx = self.regs[x];
		let vy = self.regs[y];

		self.regs[0xF] = (vy > vx) as u8;
		self.regs[x] = vy.wrapping_sub(vx);
	}

	/// Set `VF` to the least-significant bit of `Vx` and then divide `Vx` by 2
	pub fn shift_right(&mut self, x: u8) {
		self.regs[0xF] = self.regs[x] & 0x1;
		self.regs[x] >>= 1;
	}
	/// Set `VF` to the most-significant bit of `Vx` and then multiply `Vx` by 2
	pub fn shift_left(&mut self, x: u8) {
		self.regs[0xF] = (self.regs[x] & 0x80) >> 7;
		self.regs[x] <<= 1;
	}

	/// `Vx = random value & byte`
	pub fn set_rand(&mut self, x: u8, byte: u8) {
		self.regs[x] = (quad_rand::rand() % 255) as u8 & byte
	}

	/// Store hundreds, tens and ones of `Vx` at `I`, `I+1` and `I+3` respectively
	#[allow(clippy::identity_op)]
	pub fn store_bcd(&mut self, x: u8) {
		let vx = self.regs[x];
		let i = self.index as usize;

		self.memory[i + 0] = vx / 100; // Hundreds
		self.memory[i + 1] = (vx % 100) / 10; // Tens
		self.memory[i + 2] = vx % 10; // Ones
	}
	/// Store values of `V0..=Vx` to `I..=I+x`
	pub fn store_through(&mut self, x: u8) {
		let i = self.index as usize;
		for v in 0..=x {
			self.memory[i + v as usize] = self.regs[v];
		}
	}
	/// Read values from `I..=I+x` to `V0..=Vx`
	pub fn read_through(&mut self, x: u8) {
		let i = self.index as usize;
		for v in 0..=x {
			self.regs[v] = self.memory[i + v as usize];
		}
	}
	/// Set `I` to the address of the `Vx`'th sprite
	pub fn set_index_sprite_addr(&mut self, x: u8) {
		self.index = self.regs[x] as u16 * Self::CHAR_HEIGHT;
	}

	/// Whether the CPU is hot
	pub fn is_hot(&self) -> bool {
		self.heat >= Self::HOT_LEVEL
	}
	/// Returns whether the CPU is about to EXPLODE!!! BE AWARE OF THAT!!!! NOOOOOOOO
	pub fn is_critical_heat(&self) -> bool {
		self.heat >= Self::CIRITICAL_HEAT_LEVEL
	}
}
