use std::io::prelude::*;

pub struct IrcReader<T> {
	inner: std::io::Split<T>
}

impl<T: BufRead> IrcReader<T> {
	pub fn new(read: T) -> IrcReader<T> {
		IrcReader {
			inner: read.split(10)	// linefeed
		}
	}
}

impl<T: BufRead> Iterator for IrcReader<T> {
	type Item = std::io::Result<String>;

	fn next(&mut self) -> Option<std::io::Result<String>> {
		let mut buf = Vec::new();
		loop {
			match self.inner.next() {
				None => return None,
				Some(Err(e)) => return Some(Err(e)),
				Some(Ok(mut bytes)) => {
					if buf.len() > 0 {buf.push(10);}
					buf.append(&mut bytes);
					if buf.last() == Some(&13) {break}
				}
			}
		}
		buf.pop();	// remove CR char at end
		Some(Ok(match String::from_utf8(buf) {
			Ok(s) => s,
			Err(e) => std::iter::FromIterator::from_iter(e.into_bytes().into_iter().map(|b| b as char))
		}))
	}
}