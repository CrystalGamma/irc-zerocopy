use FirstCharExt;

#[cfg(test)]
mod tests {
	use super::{parse_irc_message, IrcMessage};
	#[test]
	fn parse_quit() {
		assert_eq!(parse_irc_message("QUIT"), Some(IrcMessage {command: "QUIT", prefix: None, args: Vec::new()}));
	}

	#[test]
	fn parse_nick() {
		assert_eq!(parse_irc_message("NICK CrystalGamma"), Some(IrcMessage {
			command: "NICK",
			prefix: None,
			args: vec!["CrystalGamma"]
		}))
	}

	#[test]
	fn parse_privmsg() {
		assert_eq!(parse_irc_message("PRIVMSG #channel :blah blah blah"), Some(IrcMessage {
			command: "PRIVMSG",
			prefix: None,
			args: vec!["#channel", "blah blah blah"]
		}))
	}

	#[test]
	fn parse_privmsg_incoming() {
		assert_eq!(parse_irc_message(":CrystalGamma!crystal@server.example.com PRIVMSG #channel :blah blah blah"), Some(IrcMessage {
			command: "PRIVMSG",
			prefix: Some("CrystalGamma!crystal@server.example.com"),
			args: vec!["#channel", "blah blah blah"]
		}))
	}
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct IrcMessage<'a> {
	pub command: &'a str,
	pub prefix: Option<&'a str>,
	pub args: Vec<&'a str>
}

pub fn parse_irc_message(msg: &str) -> Option<IrcMessage> {
	if msg.len() == 0 { return None }	// an empty line is not an IRC message
	let (prefix, main) = if msg.first_char() == ':' {
		match msg.find(' ') {
			Some(p) => (Some(&msg[1..p]), &msg[(p+1)..]),
			None => return None
		}
	} else { (None, msg) };
	let (cmd, rest) = main.find(' ').map(|p| (&main[..p], &main[(p+1)..])).unwrap_or((main, &main[..0]));
	let args = IrcSplit(rest).collect();
	Some(IrcMessage {
		command: cmd,
		prefix: prefix,
		args: args
	})
}

use std::fmt::{Display, Formatter, Error};

impl<'a> Display for IrcMessage<'a> {
	fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
		match self.prefix {
			Some(p) => try!(write!(fmt, ":{} {}", p, self.command)),
			None => try!(write!(fmt, "{}", self.command))
		}
		let (rest, tail) = match self.args.last() {
			Some(l) => if l.contains(' ') || l.first_char() == ':' {
				(&self.args[..(self.args.len()-1)], Some(l))
			} else {
				(&self.args[..], None)
			},
			None => (&self.args[..], None)
		};
		for arg in rest {
			try!(write!(fmt, " {}", arg));
		}
		match tail {
			Some(t) => try!(write!(fmt, " :{}", t)),
			None => {}
		}
		Ok(())
	}
}

struct IrcSplit<'a>(&'a str);

impl<'a> Iterator for IrcSplit<'a> {
	type Item = &'a str;
	fn next<'x>(&'x mut self) -> Option<&'a str> {
		let IrcSplit(s) = *self;
		if s.len() == 0 { return None }
		if s.first_char() != ':' {
			let (res, next) = s.find(' ').map(|p| (Some(&s[..p]), &s[(p+1)..])).unwrap_or((Some(s), &s[..0]));
			*self = IrcSplit(next);
			res
		} else {
			*self = IrcSplit(&s[..0]);
			Some(&s[1..])
		}
	}
}