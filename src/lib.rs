#![feature(collections)]
#![feature(io)]

pub mod parse;
pub use parse::{IrcMessage, parse_irc_message};
pub mod reader;
pub use reader::IrcReader;

#[derive(Copy, Debug)]
pub struct TargetList<'a>(&'a str);

impl<'a> TargetList<'a> {
	pub fn unwrap(self) -> &'a str {
		let TargetList(inner) = self;
		return inner;
	}
	pub fn from_str(inner: &'a str) -> TargetList<'a> {
		TargetList(inner)
	}
	pub fn iter(&self) -> std::str::Split<'a, char> {
		let &TargetList(ref inner) = self;
		inner.split(',')
	}
}

#[derive(Debug)]
pub enum TypedMessage<'a> {
	Talk(TargetList<'a>, &'a str),
	Msg(&'a str, TargetList<'a>, &'a str),
	Notify(TargetList<'a>, &'a str),
	Notice(&'a str, TargetList<'a>, &'a str),
	SetNick(&'a str),
	NickChanged(&'a str, &'a str),
	Ping(Vec<&'a str>),
	Pong(Vec<&'a str>),
	Joined(&'a str, TargetList<'a>),
	Topic(&'a str, Option<&'a str>),
	Join(TargetList<'a>, Option<TargetList<'a>>),
	Register(&'a str, &'a str),	// user, real name
	Welcome(&'a str),
	YourHost(&'a str),
	MotdStart(&'a str),
	Motd(&'a str),
	MotdEnd(&'a str),
	Other(IrcMessage<'a>)
}
pub use TypedMessage::*;

pub fn analyse_message(msg: IrcMessage) -> TypedMessage {
	match msg.command {
		"PING" => Ping(msg.args),
		"001" if msg.args.len() == 2 => Welcome(msg.args[1]),
		"002" if msg.args.len() == 2 => YourHost(msg.args[1]),
		"375" if msg.args.len() == 2 => MotdStart(msg.args[1]),
		"372" if msg.args.len() == 2 => Motd(msg.args[1]),
		"376" if msg.args.len() == 2 => MotdEnd(msg.args[1]),
		"JOIN" if msg.args.len() == 1 => match msg.prefix {
			Some(p) => Joined(p, TargetList::from_str(msg.args[0])),
			None => Join(TargetList::from_str(msg.args[0]), None)
		},
		"NICK" if msg.args.len() == 1 => match msg.prefix {
			Some(p) => NickChanged(p, msg.args[0]),
			None => SetNick(msg.args[0])
		},
		"PRIVMSG" if msg.args.len() == 2 => match msg.prefix {
			Some(p) => Msg(p, TargetList::from_str(msg.args[0]), msg.args[1]),
			None => Talk(TargetList::from_str(msg.args[0]), msg.args[1])
		},
		"NOTICE" if msg.args.len() == 2 => match msg.prefix {
			Some(p) => Notice(p, TargetList::from_str(msg.args[0]), msg.args[1]),
			None => Notify(TargetList::from_str(msg.args[0]), msg.args[1])
		},
		"331" if msg.args.len() == 2 => Topic(msg.args[0], None),
		"332" if msg.args.len() == 2 => Topic(msg.args[0], Some(msg.args[1])),
		_ => Other(msg)
	}
}

impl<'a> TypedMessage<'a> {
	pub fn to_dumb(self) -> IrcMessage<'a> {
		match self {
			Pong(targets) => IrcMessage {
				prefix: None,
				command: "PONG",
				args: targets
			},
			Join(channels, keys) => IrcMessage {
				prefix: None,
				command: "JOIN",
				args: match keys {
					Some(list) => vec![channels.unwrap(), list.unwrap()],
					None => vec![channels.unwrap()]
				}
			},
			Talk(list, text) => IrcMessage {
				prefix: None,
				command: "PRIVMSG",
				args: vec![list.unwrap(), text]
			},
			Notify(list, text) => IrcMessage {
				prefix: None,
				command: "NOTICE",
				args: vec![list.unwrap(), text]
			},
			SetNick(nick) => IrcMessage {
				prefix: None,
				command: "NICK",
				args: vec![nick]
			},
			Register(user, real_name) => IrcMessage {
				prefix: None,
				command: "USER",
				args: vec![user, "8", "-", real_name]
			},
			_ => unimplemented!()
		}
	}
	pub fn is_motd(&self) -> bool {
		match self {
			&Motd(..) | &MotdStart(..) | &MotdEnd(..) => true,
			_ => false
		}
	}
}

pub fn nick_from_mask(mask: &str) -> &str {
	mask.find('!').or_else(|| mask.find('@')).map_or(mask, |p| &mask[..p])
}

pub fn is_channel_name(s: &str) -> bool {
	if s.len() > 2 {
		let c = s.char_at(0);
		c == '#' || c == '!' || c == '&' || c == '+'
	} else {false}
}