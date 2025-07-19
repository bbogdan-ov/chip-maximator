pub trait FloatMath {
	/// Snap float to the specified step
	fn snap_floor(self, step: f32) -> f32;
	fn snap_round(self, step: f32) -> f32;
}
impl FloatMath for f32 {
	fn snap_floor(self, step: f32) -> f32 {
		(self / step).floor() * step
	}
	fn snap_round(self, step: f32) -> f32 {
		(self / step).round() * step
	}
}

pub trait Lerp {
	fn lerp(self, to: Self, alpha: f32) -> Self
	where
		Self: Sized;
}
impl Lerp for f32 {
	fn lerp(self, to: Self, alpha: f32) -> Self {
		self + (to - self) * alpha
	}
}

pub trait ToStrBytes {
	/// Convert into array of bytes representing up to 4 ASCII chars (in decimal form)
	/// It is faster than `self.to_string()` because no allocations happen
	fn to_str_bytes(self) -> [u8; 4];
	/// Convert into array of bytes representing up to 4 ASCII chars (in hex form)
	/// It is faster than `format!("{:x}", self)` because no allocations happen
	fn to_hex_str_bytes(self, put_zeros: bool) -> [u8; 4];
}
impl ToStrBytes for u32 {
	fn to_str_bytes(self) -> [u8; 4] {
		debug_assert!(self <= 9999);
		to_str_bytes(self)
	}
	fn to_hex_str_bytes(self, put_zeros: bool) -> [u8; 4] {
		debug_assert!(self <= 0xffff);
		to_hex_str_bytes(self, put_zeros)
	}
}
impl ToStrBytes for i32 {
	fn to_str_bytes(self) -> [u8; 4] {
		(self as u32).to_str_bytes()
	}
	fn to_hex_str_bytes(self, put_zeros: bool) -> [u8; 4] {
		(self as u32).to_hex_str_bytes(put_zeros)
	}
}
impl ToStrBytes for u16 {
	fn to_str_bytes(self) -> [u8; 4] {
		(self as u32).to_str_bytes()
	}
	fn to_hex_str_bytes(self, put_zeros: bool) -> [u8; 4] {
		(self as u32).to_hex_str_bytes(put_zeros)
	}
}
impl ToStrBytes for u8 {
	fn to_str_bytes(self) -> [u8; 4] {
		to_str_bytes(self as u32)
	}
	fn to_hex_str_bytes(self, put_zeros: bool) -> [u8; 4] {
		to_hex_str_bytes(self as u32, put_zeros)
	}
}

fn to_str_bytes(n: u32) -> [u8; 4] {
	/// Starting index of numbers in the ASCII table
	const NUMS_START: u8 = 48;

	let mut arr = [0; 4];

	if n < 10 {
		arr[0] = NUMS_START + n as u8; // Ones
	} else if n < 100 {
		arr[0] = NUMS_START + (n / 10) as u8; // Tens
		arr[1] = NUMS_START + (n % 10) as u8; // Ones
	} else if n < 1000 {
		arr[0] = NUMS_START + (n / 100) as u8; // Hundreds
		arr[1] = NUMS_START + (n % 100 / 10) as u8; // Tens
		arr[2] = NUMS_START + (n % 10) as u8; // Ones
	} else {
		arr[0] = NUMS_START + (n / 1000) as u8; // Thousands
		arr[1] = NUMS_START + (n % 1000 / 100) as u8; // Hundreds
		arr[2] = NUMS_START + (n % 100 / 10) as u8; // Tens
		arr[3] = NUMS_START + (n % 10) as u8; // Ones
	}

	arr
}
fn to_hex_str_bytes(n: u32, put_zeros: bool) -> [u8; 4] {
	/// Starting index of numbers in the ASCII table
	const NUMS_START: u8 = 48;
	/// Starting index of uppercase letters in the ASCII table
	const LETTERS_START: u8 = 65;

	let mut arr = if put_zeros { [b'0'; 4] } else { [0; 4] };

	#[inline(always)]
	fn to_hex(n: u32) -> u8 {
		if n <= 0x9 {
			NUMS_START + n as u8 // 0-9
		} else {
			LETTERS_START + n as u8 - 10 // a-f
		}
	}

	if n <= 0xf {
		if put_zeros {
			arr[3] = to_hex(n);
		} else {
			arr[0] = to_hex(n);
		}
	} else if n <= 0xff {
		if put_zeros {
			arr[2] = to_hex(n / 16);
			arr[3] = to_hex(n % 16);
		} else {
			arr[0] = to_hex(n / 16);
			arr[1] = to_hex(n % 16);
		}
	} else if n <= 0xfff {
		if put_zeros {
			arr[1] = to_hex(n / 256);
			arr[2] = to_hex(n % 256 / 16);
			arr[3] = to_hex(n % 16);
		} else {
			arr[0] = to_hex(n / 256);
			arr[1] = to_hex(n % 256 / 16);
			arr[2] = to_hex(n % 16);
		}
	} else {
		arr[0] = to_hex(n / 4096);
		arr[1] = to_hex(n % 4096 / 256);
		arr[2] = to_hex(n % 256 / 16);
		arr[3] = to_hex(n % 16);
	}

	arr
}

// Tests
#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn str_bytes() {
		assert_eq!(2.to_str_bytes(), [b'2', 0, 0, 0]);
		assert_eq!(12.to_str_bytes(), [b'1', b'2', 0, 0]);
		assert_eq!(123.to_str_bytes(), [b'1', b'2', b'3', 0]);
		assert_eq!(1234.to_str_bytes(), [b'1', b'2', b'3', b'4']);
		assert_eq!(9999.to_str_bytes(), [b'9', b'9', b'9', b'9']);

		assert_eq!(0x9.to_hex_str_bytes(false), [b'9', 0, 0, 0]);
		assert_eq!(0xa.to_hex_str_bytes(false), [b'A', 0, 0, 0]);
		assert_eq!(0x12.to_hex_str_bytes(false), [b'1', b'2', 0, 0]);
		assert_eq!(0xab.to_hex_str_bytes(false), [b'A', b'B', 0, 0]);
		assert_eq!(0x123.to_hex_str_bytes(false), [b'1', b'2', b'3', 0]);
		assert_eq!(0xabc.to_hex_str_bytes(false), [b'A', b'B', b'C', 0]);
		assert_eq!(0x1bc.to_hex_str_bytes(false), [b'1', b'B', b'C', 0]);
		assert_eq!(0x1b2c.to_hex_str_bytes(false), [b'1', b'B', b'2', b'C']);
		assert_eq!(0xffff.to_hex_str_bytes(false), [b'F', b'F', b'F', b'F']);
	}

	#[test]
	#[should_panic]
	fn str_bytes_panic_1() {
		let _ = 10_000.to_str_bytes();
	}
	#[test]
	#[should_panic]
	fn str_bytes_panic_2() {
		let _ = 65536.to_hex_str_bytes(false);
	}
}
