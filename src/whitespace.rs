
/**
 * This file is copy-adapted from the original Nom implementation of `tuple`
 * and the corresponding `Tuple` trait.
 */

use nom::{
    character::complete::multispace0,
    error::ParseError,
    traits::InputTakeAtPosition,
    IResult,
};

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! succ (
  (0, $submac:ident ! ($($rest:tt)*)) => ($submac!(1, $($rest)*));
  (1, $submac:ident ! ($($rest:tt)*)) => ($submac!(2, $($rest)*));
  (2, $submac:ident ! ($($rest:tt)*)) => ($submac!(3, $($rest)*));
  (3, $submac:ident ! ($($rest:tt)*)) => ($submac!(4, $($rest)*));
  (4, $submac:ident ! ($($rest:tt)*)) => ($submac!(5, $($rest)*));
  (5, $submac:ident ! ($($rest:tt)*)) => ($submac!(6, $($rest)*));
  (6, $submac:ident ! ($($rest:tt)*)) => ($submac!(7, $($rest)*));
  (7, $submac:ident ! ($($rest:tt)*)) => ($submac!(8, $($rest)*));
  (8, $submac:ident ! ($($rest:tt)*)) => ($submac!(9, $($rest)*));
  (9, $submac:ident ! ($($rest:tt)*)) => ($submac!(10, $($rest)*));
  (10, $submac:ident ! ($($rest:tt)*)) => ($submac!(11, $($rest)*));
  (11, $submac:ident ! ($($rest:tt)*)) => ($submac!(12, $($rest)*));
  (12, $submac:ident ! ($($rest:tt)*)) => ($submac!(13, $($rest)*));
  (13, $submac:ident ! ($($rest:tt)*)) => ($submac!(14, $($rest)*));
  (14, $submac:ident ! ($($rest:tt)*)) => ($submac!(15, $($rest)*));
  (15, $submac:ident ! ($($rest:tt)*)) => ($submac!(16, $($rest)*));
  (16, $submac:ident ! ($($rest:tt)*)) => ($submac!(17, $($rest)*));
  (17, $submac:ident ! ($($rest:tt)*)) => ($submac!(18, $($rest)*));
  (18, $submac:ident ! ($($rest:tt)*)) => ($submac!(19, $($rest)*));
  (19, $submac:ident ! ($($rest:tt)*)) => ($submac!(20, $($rest)*));
  (20, $submac:ident ! ($($rest:tt)*)) => ($submac!(21, $($rest)*));
);/// helper trait for the tuple combinator

///
/// this trait is implemented for tuples of parsers of up to 21 elements
pub trait WhitespaceSeparatedTuple<I,O,E> {
  /// parses the input and returns a tuple of results of each parser
  fn parse(&self, input: I) -> IResult<I,O,E>;
}

impl<Input, Output, Error: ParseError<Input>, F: Fn(Input) -> IResult<Input, Output, Error> > WhitespaceSeparatedTuple<Input, (Output,), Error> for (F,) {
   fn parse(&self, input: Input) -> IResult<Input,(Output,),Error> {
     self.0(input).map(|(i,o)| (i, (o,)))
   }
}

macro_rules! whitespace_separated_tuple_trait(
  ($name1:ident $ty1:ident, $name2: ident $ty2:ident, $($name:ident $ty:ident),*) => (
    whitespace_separated_tuple_trait!(__impl $name1 $ty1, $name2 $ty2; $($name $ty),*);
  );
  (__impl $($name:ident $ty: ident),+; $name1:ident $ty1:ident, $($name2:ident $ty2:ident),*) => (
    whitespace_separated_tuple_trait_impl!($($name $ty),+);
    whitespace_separated_tuple_trait!(__impl $($name $ty),+ , $name1 $ty1; $($name2 $ty2),*);
  );
  (__impl $($name:ident $ty: ident),+; $name1:ident $ty1:ident) => (
    whitespace_separated_tuple_trait_impl!($($name $ty),+);
    whitespace_separated_tuple_trait_impl!($($name $ty),+, $name1 $ty1);
  );
);

macro_rules! whitespace_separated_tuple_trait_impl(
  ($($name:ident $ty: ident),+) => (
    impl<
      Input: Clone + InputTakeAtPosition, $($ty),+ , Error: ParseError<Input>,
      $($name: Fn(Input) -> IResult<Input, $ty, Error>),+
    > WhitespaceSeparatedTuple<Input, ( $($ty),+ ), Error> for ( $($name),+ ) {

      fn parse(&self, input: Input) -> IResult<Input, ( $($ty),+ ), Error> {
        whitespace_separated_tuple_trait_inner!(0, self, input, (), $($name)+)

      }
    }
  );
);

macro_rules! whitespace_separated_tuple_trait_inner(
  ($it:tt, $self:expr, $input:expr, (), $head:ident $($id:ident)+) => ({
    let (i, o) = $self.$it($input.clone())?;
    let (i, _) = multispace0(i)?;

    succ!($it, whitespace_separated_tuple_trait_inner!($self, i, ( o ), $($id)+))
  });
  ($it:tt, $self:expr, $input:expr, ($($parsed:tt)*), $head:ident $($id:ident)+) => ({
    let (i, o) = $self.$it($input.clone())?;
    let (i, _) = multispace0(i)?;

    succ!($it, whitespace_separated_tuple_trait_inner!($self, i, ($($parsed)* , o), $($id)+))
  });
  ($it:tt, $self:expr, $input:expr, ($($parsed:tt)*), $head:ident) => ({
    let (i, o) = $self.$it($input.clone())?;

    Ok((i, ($($parsed)* , o)))
  });
);

whitespace_separated_tuple_trait!(FnA A, FnB B, FnC C, FnD D, FnE E, FnF F, FnG G, FnH H, FnI I, FnJ J, FnK K, FnL L,
  FnM M, FnN N, FnO O, FnP P, FnQ Q, FnR R, FnS S, FnT T, FnU U);

/// applies a tuple of parsers one by one and returns their results as a tuple
///
/// ```rust
/// # use nom::{Err, error::ErrorKind};
/// use nom::sequence::tuple;
/// use nom::character::complete::{alpha1, digit1};
/// let parser = tuple((alpha1, digit1, alpha1));
///
/// assert_eq!(parser("abc123def"), Ok(("", ("abc", "123", "def"))));
/// assert_eq!(parser("123def"), Err(Err::Error(("123def", ErrorKind::Alpha))));
/// ```
pub fn whitespace_separated_tuple<I: Clone, O, E: ParseError<I>, List: WhitespaceSeparatedTuple<I,O,E>>(l: List)  -> impl Fn(I) -> IResult<I, O, E> {
  move |i: I| {
    l.parse(i)
  }
}
