// SPDX-License-Identifier: MPL-2.0
use std::io::Read;

pub fn yes_no_prompt() -> bool {
	let mut stdin = std::io::stdin().lock();
	let mut buf = [0_u8; 1];
	if stdin.read_exact(&mut buf).is_err() {
		return false;
	}
	let c = buf[0] as char;
	c == 'y' || c == 'Y'
}
