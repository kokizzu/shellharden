use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::ops::Deref;

fn main() {
	let args: Vec<String> = env::args().collect();

	for arg in &args[1..] {
		match treatfile(arg) {
			Ok(()) => {},
			Err(e) => {
				println!("{}: {}", arg, e);
			},
		}
	}
}

#[derive(PartialEq)]
#[derive(Copy, Clone)]
enum StateType {
	Normal,
	StringSq,
	StringDq,
	Escape,
}

fn treatfile(path: &str) -> Result<(), std::io::Error> {
	const LOOKAHEAD :usize = 80;
	const SLACK     :usize = 48;
	const BUFSIZE   :usize = SLACK + LOOKAHEAD;
	let mut fill    :usize = 0;
	let mut buf = [0; BUFSIZE];

	let mut state = vec!{StateType::Normal};

	let mut fh = try!(File::open(path));
	let stdout = io::stdout();
	let mut out = stdout.lock();
	loop {
		let bytes = try!(fh.read(&mut buf[fill .. BUFSIZE]));
		fill += bytes;
		if fill <= LOOKAHEAD {
			if bytes != 0 {
				continue;
			}
			break;
		}
		let usable = fill - LOOKAHEAD;
		try!(stackmachine(&mut state, &mut out, &buf[0 .. fill], usable));
		for i in 0 .. LOOKAHEAD {
			buf[i] = buf[usable + i];
		}
		fill = LOOKAHEAD;
	}
	try!(stackmachine(&mut state, &mut out, &buf[0 .. fill], fill));
	Ok(())
}

fn stackmachine(
	state :&mut Vec<StateType>,
	out :&mut std::io::StdoutLock,
	buf :&[u8],
	usable :usize
) -> Result<(), std::io::Error> {
	for i in 0 .. usable {
		let curstate :StateType = *state.last().unwrap();
		let mut newstate = curstate;
		let mut pop = false;

		match (curstate, buf[i]) {
			(StateType::Normal, b'\"') => {
				newstate = StateType::StringDq;
			}
			(StateType::Normal, b'\'') => {
				newstate = StateType::StringSq;
			}
			(StateType::StringDq, b'\\') => {
				newstate = StateType::Escape;
			}
			(StateType::StringDq, b'\"') => {
				pop = true;
			}
			(StateType::StringSq, b'\'') => {
				pop = true;
			}
			(StateType::Escape, _) => {
				pop = true;
			}
			(_, _) => {}
		}

		if newstate != curstate {
			state.push(newstate);
			try!(out.write(color_by_state(newstate)));
		}
		try!(out.write(&buf[i .. i+1]));
		if pop {
			state.pop();
			newstate = *state.last().unwrap();
			try!(out.write(color_by_state(newstate)));
		}
	}
	Ok(())
}

fn color_by_state(state :StateType) -> &'static [u8] {
	match state {
		StateType::Normal => b"\x1b[m",
		StateType::StringDq => b"\x1b[0;31m",
		StateType::StringSq => b"\x1b[0;35m",
		StateType::Escape => b"\x1b[1;35m",
	}
}
