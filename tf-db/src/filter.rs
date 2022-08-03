use std::str::FromStr;

use nom::{
	branch::alt,
	bytes::complete::{tag, take_while1},
	character::complete::{char, digit1, multispace0, one_of},
	combinator::{eof, map, map_opt, opt, recognize},
	error::ParseError,
	multi::fold_many0,
	sequence::{delimited, preceded, tuple},
	IResult,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
	And(Box<Filter>, Box<Filter>),
	Or(Box<Filter>, Box<Filter>),
	Not(Box<Filter>),
	LessThan {
		tag: String,
		threshold: f32,
		inclusive: bool,
	},
}

fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
	inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
	F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
	delimited(multispace0, inner, multispace0)
}

fn tag_name(i: &str) -> IResult<&str, String> {
	map(
		take_while1(|c: char| c.is_ascii_alphabetic() || c == '_'),
		ToOwned::to_owned,
	)(i)
}

fn float(input: &str) -> IResult<&str, &str> {
	alt((
		// Case one: .42
		recognize(tuple((
			char('.'),
			digit1,
			opt(tuple((one_of("eE"), opt(one_of("+-")), digit1))),
		))), // Case two: 42e42 and 42.42e42
		recognize(tuple((
			digit1,
			opt(preceded(char('.'), digit1)),
			one_of("eE"),
			opt(one_of("+-")),
			digit1,
		))), // Case three: 42. and 42.42
		recognize(tuple((digit1, char('.'), opt(digit1)))),
	))(input)
}

fn threshold(i: &str) -> IResult<&str, f32> {
	map_opt(float, |n: &str| match n.parse::<f32>() {
		Ok(n) if n >= 0.0 && n <= 1.0 => Some(n),
		_ => None,
	})(i)
}

// tag comparisons
fn filter3(i: &str) -> IResult<&str, Filter> {
	println!("filter3 {:?}", i);
	alt((map(
		tuple((
			tag_name,
			ws(alt((tag("<="), tag("<"), tag(">="), tag(">"), tag("=")))),
			threshold,
		)),
		|(tag, op, threshold)| match op {
			"<" => Filter::LessThan {
				tag,
				threshold,
				inclusive: false,
			},
			"<=" => Filter::LessThan {
				tag,
				threshold,
				inclusive: true,
			},
			">" => Filter::Not(Box::new(Filter::LessThan {
				tag,
				threshold,
				inclusive: true,
			})),
			">=" => Filter::Not(Box::new(Filter::LessThan {
				tag,
				threshold,
				inclusive: false,
			})),
			"=" => Filter::And(
				Box::new(Filter::LessThan {
					tag: tag.clone(),
					threshold,
					inclusive: true,
				}),
				Box::new(Filter::Not(Box::new(Filter::LessThan {
					tag,
					threshold,
					inclusive: false,
				}))),
			),
			_ => unreachable!(),
		},
	),))(i)
}

// consider negation
fn filter2(i: &str) -> IResult<&str, Filter> {
	println!("filter2 {:?}", i);
	let (i, neg) = opt(ws(char('!')))(i)?;
	let (i, filter) = filter3(i)?;
	if neg.is_some() {
		Ok((i, Filter::Not(Box::new(filter))))
	} else {
		Ok((i, filter))
	}
}

// aggregates ANDs
fn filter1(i: &str) -> IResult<&str, Filter> {
	println!("filter1 {:?}", i);
	let (i, first) = filter2(i)?;
	fold_many0(
		preceded(ws(tag("&")), filter2),
		move || first.clone(),
		|lhs: Filter, rhs: Filter| Filter::And(Box::new(lhs), Box::new(rhs)),
	)(i)
}

// most general, aggregates ORs
fn filter0(i: &str) -> IResult<&str, Filter> {
	println!("filter {:?}", i);
	let (i, first) = filter1(i)?;
	fold_many0(
		preceded(ws(tag("|")), filter1),
		move || first.clone(),
		|lhs: Filter, rhs: Filter| Filter::Or(Box::new(lhs), Box::new(rhs)),
	)(i)
}

fn filter(i: &str) -> IResult<&str, Filter> {
	alt((
		filter0,
		map(tag(""), |_| Filter::LessThan {
			tag: String::from("a"),
			threshold: 1.0,
			inclusive: true,
		}),
	))(i)
}

impl FromStr for Filter {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> anyhow::Result<Self> {
		let s = tuple((filter, eof))(s)
			.map_err(|e| anyhow::anyhow!("failed to parse: {:?}", e))?
			.1;
		Ok(s.0)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_empty() {
		assert_eq!(
			Filter::from_str("").unwrap(),
			Filter::LessThan {
				tag: String::from("a"),
				threshold: 1.0,
				inclusive: true,
			}
		);
	}

	#[test]
	fn test_conditional() {
		assert_eq!(
			Filter::from_str("foo > 0.5").unwrap(),
			Filter::Not(Box::new(Filter::LessThan {
				tag: String::from("foo"),
				threshold: 0.5,
				inclusive: true,
			}))
		);
	}

	#[test]
	fn test_complex() {
		assert_eq!(
			Filter::from_str("foo > 0.5 & bar < 0.2 | !foo < 0.9 | baz = 0.3").unwrap(),
			Filter::Or(
				Box::new(Filter::Or(
					Box::new(Filter::And(
						Box::new(Filter::Not(Box::new(Filter::LessThan {
							tag: String::from("foo"),
							threshold: 0.5,
							inclusive: true
						}))),
						Box::new(Filter::LessThan {
							tag: String::from("bar"),
							threshold: 0.2,
							inclusive: false
						})
					)),
					Box::new(Filter::Not(Box::new(Filter::LessThan {
						tag: String::from("foo"),
						threshold: 0.9,
						inclusive: false
					}))),
				)),
				Box::new(Filter::And(
					Box::new(Filter::LessThan {
						tag: String::from("baz"),
						threshold: 0.3,
						inclusive: true
					}),
					Box::new(Filter::Not(Box::new(Filter::LessThan {
						tag: String::from("baz"),
						threshold: 0.3,
						inclusive: false
					})))
				)),
			),
		);
	}
}
