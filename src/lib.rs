#[derive(Clone, Debug, PartialEq, Eq)]
struct Element {
  name: String,
  attributes: Vec<(String, String)>,
  children: Vec<Element>,
}

pub fn the_letter_a(input: &str) -> Result<(&str, ()), &str> {
  match input.chars().next() {
    Some('a') => Ok((&input['a'.len_utf8()..], ())),
    _ => Err(input),
  }
}

fn match_literal(expected: &'static str) -> impl Fn(&str) -> Result<(&str, ()), &str> {
  move |input| match input.get(0..expected.len()) {
    Some(next) if next == expected => Ok((&input[expected.len()..], ())),
    _ => Err(input),
  }
}

// Takes alphabetic or dashes and matches them
fn identifier(input: &str) -> Result<(&str, String), &str> {
  let mut matched = String::new();
  let mut chars = input.chars();
  match chars.next() {
    Some(next) if next.is_alphabetic() => matched.push(next),
    _ => return Err(input),
  }

  while let Some(next) = chars.next() {
    if next.is_alphabetic() || next == '-' {
      matched.push(next);
    } else {
      break;
    }
  }

  let next_index = matched.len();
  Ok((&input[next_index..], matched))
}

// 1st implementation
// fn pair<P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Fn(&str) -> Result<(&str, (R1, R2)), &str>
// where
//   P1: Fn(&str) -> Result<(&str, R1), &str>,
//   P2: Fn(&str) -> Result<(&str, R2), &str>,
// {
//   move |input| match parser1(input) {
//     Ok((next_input, result1)) => match parser2(next_input) {
//       Ok((final_input, result2)) => Ok((final_input, (result1, result2))),
//       Err(err) => Err(err),
//     },
//     Err(err) => Err(err),
//   }
// }

// 1st implementation
// fn map<P, F, A, B>(parser: P, map_fn: F) -> impl Fn(&str) -> Result<(&str, B), &str>
// where
//   P: Fn(&str) -> Result<(&str, A), &str>,
//   F: Fn(A) -> B,
// {
//   // move |input| match parser(input) {
//   //   Ok((next_input, result)) => Ok((next_input, map_fn(result))),
//   //   Err(err) => Err(err),
//   // }
//   move |input| parser(input).map(|(next_input, result)| (next_input, map_fn(result)))
// }

type ParseResult<'a, Output> = Result<(&'a str, Output), &'a str>;

trait Parser<'a, Output> {
  fn parse(&self, input: &'a str) -> ParseResult<'a, Output>;
}

// so we're implementing the Parser trait for all types that conform to the trait bound
// specified, where F is a Fn(&'a str) -> ParseResult<Output>
// why can't we define a type ParserFn that does this?
impl<'a, F, Output> Parser<'a, Output> for F
where
  F: Fn(&'a str) -> ParseResult<Output>,
{
  fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
    self(input)
  }
}

fn map<'a, P, F, A, B>(parser: P, map_fn: F) -> impl Parser<'a, B>
where
  P: Parser<'a, A>,
  F: Fn(A) -> B,
{
  move |input| {
    parser
      .parse(input)
      .map(|(next_input, result)| (next_input, map_fn(result)))
  }
}

fn pair<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, (R1, R2)>
where
  P1: Parser<'a, R1>,
  P2: Parser<'a, R2>,
{
  move |input| {
    parser1.parse(input).and_then(|(next_input, result1)| {
      parser2
        .parse(next_input)
        .map(|(last_input, result2)| (last_input, (result1, result2)))
    })
  }
}

fn left<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R1>
where
  P1: Parser<'a, R1>,
  P2: Parser<'a, R2>,
{
  map(pair(parser1, parser2), |(left, _right)| left)
}

fn right<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R2>
where
  P1: Parser<'a, R1>,
  P2: Parser<'a, R2>,
{
  map(pair(parser1, parser2), |(_left, right)| right)
}

#[test]
fn literal_parser() {
  let parse_joe = match_literal("Hello Joe!");
  assert_eq!(Ok(("", ())), parse_joe("Hello Joe!"));
  assert_eq!(
    Ok((" Hello Robert!", ())),
    parse_joe("Hello Joe! Hello Robert!")
  );
  assert_eq!(Err("Hello Mike!"), parse_joe("Hello Mike!"));
}

#[test]
fn identifier_parser() {
  assert_eq!(
    Ok(("", "identifier-with-dash".to_string())),
    identifier("identifier-with-dash")
  );
  assert_eq!(
    Ok((" entirely an identifier", "not".to_string())),
    identifier("not entirely an identifier")
  );
  assert_eq!(
    Err("!not-and-identifier"),
    identifier("!not-and-identifier")
  );
}

#[test]
fn pair_combinator() {
  let tag_opener = pair(match_literal("<"), identifier);
  assert_eq!(
    Ok(("/>", ((), "my-first-element".to_string()))),
    tag_opener.parse("<my-first-element/>")
  );
  assert_eq!(Err("oops"), tag_opener.parse("oops"));
  assert_eq!(Err("!oops"), tag_opener.parse("<!oops"));
}
