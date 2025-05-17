use std::{collections::HashMap, fmt::{self, Display}, ops::Range, sync::LazyLock};

use console::style;
use itertools::Itertools;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HumanDate {
	pub year: u32,
	pub month: Option<Month>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Month {
	January,
	February,
	March,
	April,
	May,
	June,
	July,
	August,
	September,
	October,
	November,
	December
}

impl HumanDate {
	pub fn parse(mut input: &str) -> Result<Self, ParseError> {
		let mut month: Option<Month> = None;
		let mut skip_dot: bool = false;

		let mut tokens = Tokenize::new(input.trim_end());

		match tokens.next().ok_or_else(|| ParseError::new(input.to_owned(), ErrorPosition::Global, ErrorReason::Insufficient(Box::new([Expected::Month, Expected::Year]))))? {
			Ok(Token { pos, data: TokenData::String(s) }) => {
				match Month::parse(s) {
					Ok(parse) => {
						month = Some(parse.month);
						skip_dot = parse.is_short;
					},
					Err(_) => return Err(ParseError::new(input.to_owned(), ErrorPosition::Range(ErrorRange::new(pos)), ErrorReason::BadElement(MONTH_EXPECTED))),
				}
			}
			Ok(Token { data: TokenData::Number(year), .. }) => return Ok(Self { year: year as u32, month: None }),
			_ => return Err(ParseError::new(input.to_owned(), ErrorPosition::Global, ErrorReason::Unrecognized(Box::new([Expected::Month, Expected::Year]))))
		}

		let year: u32 = match {
			let token = tokens.next().ok_or_else(|| ParseError::new(input.to_owned(), ErrorPosition::Global, ErrorReason::Insufficient(Box::new([Expected::Period, Expected::Year]))))?.map_err(|e| {
				ParseError { input: input.to_owned(), position: e.position, reason: ErrorReason::BadElement(ParseError::EXPECTED_FORMAT) }
			})?;

			match token.data {
				TokenData::Punctuation(_) => {
					if !skip_dot { return Err(ParseError::new(input.to_owned(), ErrorPosition::Char(ErrorLocation::new(token.pos.start)), ErrorReason::Unrecognized(Box::new([Expected::Year])))) }

					tokens.next().ok_or_else(|| ParseError::new(input.to_owned(), ErrorPosition::Global, ErrorReason::Insufficient(Box::new([Expected::Year]))))?.map_err(|e| {
						ParseError { input: input.to_owned(), position: e.position, reason: ErrorReason::BadElement(ParseError::EXPECTED_FORMAT) }
					})?
				},
				_ => token
			}
		} {
			Token { data: TokenData::Number(year), .. } => year as u32,
			token => return Err(ParseError::new(input.to_owned(), ErrorPosition::Range(ErrorRange::new(token.pos)), ErrorReason::Unrecognized(Box::new([Expected::Year]))))
		};

		if tokens.next().is_some() { return Err(ParseError::new(input.to_owned(), ErrorPosition::Global, ErrorReason::ExtraComponent)); }

		return Ok(Self { year, month })
	}
}

impl Month {
	fn get_map() -> &'static HashMap<&'static str, MonthParse> {
		static STRING_MAP: LazyLock<HashMap<&str, MonthParse>> = LazyLock::new(|| HashMap::from([
			("January",   MonthParse::new(Month::January,   false)), ("Jan",  MonthParse::new(Month::January,   true)),
			("February",  MonthParse::new(Month::February,  false)), ("Feb",  MonthParse::new(Month::February,  true)),
			("March",     MonthParse::new(Month::March,     false)),
			("April",     MonthParse::new(Month::April,     false)),
			("May",       MonthParse::new(Month::May,       false)),
			("June",      MonthParse::new(Month::June,      false)),
			("July",      MonthParse::new(Month::July,      false)),
			("August",    MonthParse::new(Month::August,    false)), ("Aug",  MonthParse::new(Month::August,    true)),
			("September", MonthParse::new(Month::September, false)), ("Sept", MonthParse::new(Month::September, true)),
			("October",   MonthParse::new(Month::October,   false)), ("Oct",  MonthParse::new(Month::October,   true)),
			("November",  MonthParse::new(Month::November,  false)), ("Nov",  MonthParse::new(Month::November,  true)),
			("December",  MonthParse::new(Month::December,  false)), ("Dec",  MonthParse::new(Month::December,  true)),
		]));

		&*STRING_MAP
	}

	pub fn parse(value: &str) -> Result<MonthParse, ()> {
		Self::get_map().get(value).copied().ok_or(())
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
	pub input: String,
	pub position: ErrorPosition,
	pub reason: ErrorReason
}

impl ParseError {
	pub const EXPECTED_FORMAT: &str = "Dates must be a month (written in full or abbreviated with an optional period) followed by an integer year";

	pub const fn new(input: String, position: ErrorPosition, reason: ErrorReason) -> Self {
		Self { input, position, reason }
	}

	pub fn help(&self) -> String {
		let select = match &self.position {
			ErrorPosition::Global => style("^".repeat(self.input.chars().count())).bold().red().to_string(),
			ErrorPosition::Char(loc) => " ".repeat(loc.char_index(&self.input)) + &style("^").bold().red().to_string(),
			ErrorPosition::Range(range) => {
				let range = range.char_range(&self.input);

				" ".repeat(range.start) + &style("^".repeat(range.end - range.start)).bold().red().to_string()
			},
		};

		let help: String = match &self.reason {
			ErrorReason::BadElement(msg) => format!("{msg}"),
			ErrorReason::Unrecognized(expects) => format!("Expected one of [{}]", expects.iter().map(|e| format!("{e:?}")).join(", ")),
			ErrorReason::ExtraComponent => return "".to_owned(),
			ErrorReason::Insufficient(expects) => format!("Expected continuation with one of [{}]", expects.iter().map(|e| format!("{e:?}")).join(", "))
		};

		format!("{} {}\n{} {}\n{}: {help}", style("|").bold().cyan(), self.input, style("|").bold().cyan(), select, style("Note").bold().cyan())
	}
}

impl Display for ParseError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self.reason {
			ErrorReason::BadElement(msg) =>  write!(f, "Improperly formatted element"),
			ErrorReason::Unrecognized(expects) => write!(f, "Unrecognized element"),
			ErrorReason::ExtraComponent => write!(f, "Extra unexpected information at end of input"),
			ErrorReason::Insufficient(expects) => write!(f, "Not enough information")
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorPosition {
	Global,
	Char(ErrorLocation),
	Range(ErrorRange)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorLocation {
	pub byte: usize
}

impl ErrorLocation {
	pub fn new(byte: usize) -> Self {
		Self { byte }
	}

	pub fn char_index(&self, input: &str) -> usize {
		input.char_indices().enumerate().find(|(_, (byte, _))| self.byte == *byte).unwrap().0
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorRange {
	pub bytes: Range<usize>
}

impl ErrorRange {
	pub fn new(bytes: Range<usize>) -> Self {
		Self { bytes }
	}

	// TODO: Don't duplicate work.
	pub fn char_range(&self, input: &str) -> Range<usize> {
		input.char_indices().enumerate().find(|(_, (byte, _))| self.bytes.start == *byte).unwrap().0
		..
		input.char_indices().enumerate().find(|(_, (byte, _))| self.bytes.end == *byte).unwrap().0
	}
}

static MONTH_EXPECTED: &str = "Months must be written either in full, or with a three-letter abbreviation (except for March, April, May, June, and July)";
static YEAR_EXPECTED: &str = "Years must be written as a positive integer";
static PUNCTUATION_EXPECTED: &str = "A period after the month is only allowed if it is abbreviated";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorReason {
	/// A parsed component was improperly formatted.
	BadElement(&'static str),
	/// A parsed component was unexpected.
	Unrecognized(Box<[Expected]>),
	/// An extra component was encountered.
	ExtraComponent,
	/// Not enough components were provided.
	Insufficient(Box<[Expected]>)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expected {
	Whitespace,
	Period,
	Month,
	Year
}

#[derive(Debug, Clone, Copy)]
pub struct MonthParse {
	pub month: Month,
	pub is_short: bool
}

impl MonthParse {
	pub const fn new(month: Month, is_short: bool) -> MonthParse {
		Self { month, is_short }
	}
}

struct Tokenize<'a> {
	len: usize,
	data: &'a str,
	end: bool
}

impl<'a> Tokenize<'a> {
	pub fn new(data: &'a str) -> Self {
		Self { len: 0, data, end: false }
	}
}

impl<'a> Iterator for Tokenize<'a> {
	type Item = Result<Token<'a>, TokenError>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.end { return None; }

		let result = Token::parse(&mut self.data);

		if let Err(TokenError { reason: TokenErrorReason::Empty, .. }) = &result { self.end = true; return None; }

		Some(result.map(|mut token| {
			let pos = &mut token.pos;

			let len = pos.end;

			pos.start += self.len;
			pos.end += self.len;

			self.len += len;

			token
		}).map_err(|mut err| {
			self.end = true;

			match &mut err.position {
				ErrorPosition::Global => (),
				ErrorPosition::Char(i) => i.byte += self.len,
				ErrorPosition::Range(r) => {
					r.bytes.start += self.len;
					r.bytes.end += self.len;
				},
			}

			err
		}))
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token<'a> {
	/// Byte range over the parsed string that this token resides in.
	pos: Range<usize>,
	data: TokenData<'a>
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharType {
	Invalid,
	Alphabetic,
	Numeric,
	Punctuation
}

fn get_char_type(c: char) -> CharType {
	if c.is_ascii_alphabetic() {
		CharType::Alphabetic
	} else if c.is_ascii_digit() {
		CharType::Numeric
	} else if c == '.' {
		CharType::Punctuation
	} else {
		CharType::Invalid
	}
}


#[derive(Debug, Clone, PartialEq, Eq)]
struct TokenError {
	position: ErrorPosition,
	reason: TokenErrorReason
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenErrorReason {
	Empty,
	Unrecognized(char),
	InternalParseError
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenExpectedType {
	Alphabetic,
	Numeric
}

impl<'a> Token<'a> {
	pub fn parse(reference: &mut &'a str) -> Result<Self, TokenError> {
		let mut input = reference.char_indices();

		enum State {
			String,
			Number
		}

		let (start, start_char) = input.by_ref().find(|(_, c)| !c.is_whitespace()).ok_or_else(|| {
			TokenError { position: ErrorPosition::Global, reason: TokenErrorReason::Empty }
		})?;

		let state: State = match get_char_type(start_char) {
			CharType::Alphabetic => State::String,
			CharType::Numeric => State::Number,
			CharType::Punctuation if start_char == '.' => {
				*reference = &reference[(start + start_char.len_utf8())..];
				return Ok(Token { pos: start..(start + start_char.len_utf8()), data: TokenData::Punctuation('.') });
			},
			_ => return Err(TokenError { position: ErrorPosition::Char(ErrorLocation::new(start)), reason: TokenErrorReason::Unrecognized(start_char) })
		};

		let (last, last_char) = input.peekable().peeking_take_while(|(_, c)| match state {
			State::String => c.is_ascii_alphabetic(),
			State::Number => c.is_ascii_digit()
		}).last().unwrap_or((start, ' '));

		let range = start..(last + last_char.len_utf8());
		let string = &reference[range.clone()];

		let data = match state {
			State::String => TokenData::String(string),
			State::Number => TokenData::Number(string.parse().map_err(|_| {
				TokenError { position: ErrorPosition::Range(ErrorRange::new(range.clone())), reason: TokenErrorReason::InternalParseError }
			})?),
		};

		*reference = &reference[(last + last_char.len_utf8())..];
		Ok(Token { pos: range, data })
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenData<'a> {
	String(&'a str),
	Number(u64),
	Punctuation(char)
}

#[cfg(test)]
mod tests {
    use std::convert::identity;

    use console::style;

    use super::*;

	fn check_token(mut string: &str, token: Token, residual: &str) {
		assert_eq!(Token::parse(&mut string), Ok(token));
		assert_eq!(string, residual);
	}

	#[test]
	fn parse_tokens() {
		check_token("str.x", Token { pos: 0..3, data: TokenData::String("str") }, ".x");
		check_token("x.", Token { pos: 0..1, data: TokenData::String("x") }, ".");
		check_token(".x", Token { pos: 0..1, data: TokenData::Punctuation('.') }, "x");

		check_token("x ", Token { pos: 0..1, data: TokenData::String("x") }, " ");

		check_token(" .", Token { pos: 1..2, data: TokenData::Punctuation('.') }, "");
		check_token(" x", Token { pos: 1..2, data: TokenData::String("x") }, "");

		check_token("50", Token { pos: 0..2, data: TokenData::Number(50) }, "");
	}

	#[test]
	fn tokenize() {
		assert_eq!(Tokenize::new(" str .  5 ").collect::<Vec<_>>(), &[
			Ok(Token { pos: 1..4, data: TokenData::String("str") }),
			Ok(Token { pos: 5..6, data: TokenData::Punctuation('.') }),
			Ok(Token { pos: 8..9, data: TokenData::Number(5) })
		]);
	}

	#[test]
	fn parse_date() {
		assert_eq!(HumanDate::parse("February 2024"), Ok(HumanDate { year: 2024, month: Some(Month::February) }));
		assert_eq!(HumanDate::parse("Feb 2024"), Ok(HumanDate { year: 2024, month: Some(Month::February) }));
		assert_eq!(HumanDate::parse("Feb. 2024"), Ok(HumanDate { year: 2024, month: Some(Month::February) }));
		assert_eq!(HumanDate::parse("2024"), Ok(HumanDate { year: 2024, month: None }));

		assert!(HumanDate::parse("Febr 2024").is_err());
		// assert!(HumanDate::parse("Feb . 2024").is_err());
		// assert!(HumanDate::parse("Feb.2024").is_err());
		assert!(HumanDate::parse("February. 2024").is_err());
		assert!(HumanDate::parse(". 2024").is_err());


		assert!(HumanDate::parse("February").is_err());
		assert!(HumanDate::parse("").is_err());
		assert!(HumanDate::parse("February 2024 X").is_err());
	}
}