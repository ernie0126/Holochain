pub mod unsigned;

pub struct Fixturator<Curve, Item> {
    curve: std::marker::PhantomData<Curve>,
    item: std::marker::PhantomData<Item>,
    index: usize,
}

impl<Curve, Item> Fixturator<Curve, Item> {
    pub fn new() -> Self {
        Fixturator::<Curve, Item> {
            index: 0,
            curve: std::marker::PhantomData,
            item: std::marker::PhantomData,
        }
    }
}

pub enum Unpredictable {}
pub enum Predictable {}
pub enum Empty {}

pub trait Fixt {
    fn fixturator<Curve>() -> Fixturator<Curve, Self>
    where
        Self: Sized,
    {
        Fixturator::<Curve, Self>::new()
    }
}


// impl<I: Sized> Default for FixTT<I> {
//     fn default() -> Self {
//         Self::Empty
//     }
// }
//
// pub trait Fixt {
//     /// @TODO it would be nice to provide a default Input type if/when that becomes available
//     /// @see https://github.com/rust-lang/rust/issues/29661
//     /// type Input: Sized = ();
//     type Input: Sized;
//     fn fixt(fixtt: FixTT<Self::Input>) -> Self;
//     fn fixt_empty() -> Self
//     where
//         Self: Sized,
//     {
//         Self::fixt(FixTT::Empty)
//     }
//     fn fixt_a() -> Self
//     where
//         Self: Sized,
//     {
//         Self::fixt(FixTT::A)
//     }
//     fn fixt_b() -> Self
//     where
//         Self: Sized,
//     {
//         Self::fixt(FixTT::B)
//     }
//     fn fixt_c() -> Self
//     where
//         Self: Sized,
//     {
//         Self::fixt(FixTT::C)
//     }
//     fn fixt_random() -> Self
//     where
//         Self: Sized,
//     {
//         Self::fixt(FixTT::Random)
//     }
//     fn fixt_input(input: Self::Input) -> Self
//     where
//         Self: Sized,
//     {
//         Self::fixt(FixTT::Input(input))
//     }
// }
//
// impl FixT for () {
//     type Input = ();
//     fn fixt(fixtt: FixTT<Self::Input>) -> Self {
//         match fixtt {
//             FixTT::A => (),
//             FixTT::Empty => (),
//             // there's no more options for ()!
//             _ => unimplemented!(),
//         }
//     }
// }
//
// impl FixT for Vec<()> {
//     type Input = ();
//     fn fixt(fixtt: FixTT<Self::Input>) -> Self {
//         match fixtt {
//             FixTT::Empty => vec![],
//             FixTT::A => vec![()],
//             FixTT::B => vec![(), ()],
//             FixTT::C => vec![(), (), ()],
//             FixTT::Random => {
//                 let random_len = <usize>::fixt(FixTT::Input(FixTUSize::Range(0, 10)));
//                 vec![(); random_len]
//             }
//             // can't randomise () over a fixed length
//             FixTT::RandomSize(_) => unimplemented!(),
//             FixTT::Input(_) => unimplemented!(),
//         }
//     }
// }
//
// impl FixT for bool {
//     type Input = ();
//     fn fixt(fixtt: FixTT<Self::Input>) -> Self {
//         match fixtt {
//             FixTT::Empty => false,
//             FixTT::A => false,
//             FixTT::B => true,
//             // there is no third option for bool!
//             FixTT::C => unimplemented!(),
//             FixTT::Random => rand::random(),
//             FixTT::RandomSize(_) => unimplemented!(),
//             FixTT::Input(_) => unimplemented!(),
//         }
//     }
// }
//
// impl FixT for Vec<bool> {
//     type Input = ();
//     fn fixt(fixtt: FixTT<Self::Input>) -> Self {
//         match fixtt {
//             FixTT::Empty => vec![],
//             FixTT::A => vec![false],
//             FixTT::B => vec![true],
//             FixTT::C => vec![false, true],
//             FixTT::Random => {
//                 let mut
//                 let random_len = <usize>::fixt(FixTT::Input(FixTUSize::Range(0, 10)));
//                 let vec: Vec<bool> = (0..random_len).map(|_| rng.gen()).collect();
//                 vec
//             }
//         }
//     }
// }
//
// impl FixT for char {
//     type Input = ();
//     fn fixt(fixtt: FixTT<Self::Input>) -> Self {
//         match fixtt {
//             // ❤
//             FixTT::A => '\u{2764}',
//             // 💩
//             FixTT::B => '\u{1F4A9}',
//             // a
//             FixTT::C => '\u{0061}',
//             // null
//             FixTT::Empty => '\u{0000}',
//             FixTT::Random => rand::random(),
//             // chars have no length
//             FixTT::RandomSize(_) => unimplemented!(),
//             FixTT::Input(_) => unimplemented!(),
//         }
//     }
// }
//
// impl FixT for String {
//     type Input = ();
//     fn fixt(fixtt: FixTT<Self::Input>) -> Self {
//         match fixtt {
//             FixTT::A => "❤💩a".to_string(),
//             FixTT::B => "foo".to_string(),
//             FixTT::C => "bar".to_string(),
//             FixTT::Empty => "".to_string(),
//             FixTT::Random => Self::fixt(FixTT::RandomSize(10)),
//             FixTT::RandomSize(len) => {
//                 let mut rng = rand::thread_rng();
//                 let vec: Vec<char> = (0..len).map(|_| rng.gen()).collect();
//                 vec.into_iter().collect()
//             }
//             FixTT::Input(_) => unimplemented!(),
//         }
//     }
// }
//
// macro_rules! fixt_unsigned {
//     ( $t:ty, $tt:ident ) => {
//         pub enum $tt {
//             Range($t, $t),
//             Z
//         }
//         impl FixT for $t {
//             type Input = $tt;
//             fn fixt(fixtt: FixTT<Self::Input>) -> Self {
//                 match fixtt {
//                     FixTT::Empty => 0,
//                     FixTT::A => <$t>::min_value(),
//                     FixTT::B => 1,
//                     FixTT::C => <$t>::max_value(),
//                     FixTT::Random => rand::random(),
//                     FixTT::RandomSize(max) => <$t>::fixt(FixTT::Input($tt::Range(0, max as _))),
//                     FixTT::Input(fixt_unsigned) => match fixt_unsigned {
//                         $tt::Range(min, max) => {
//                             let mut rng = rand::thread_rng();
//                             rng.gen_range(min, max)
//                         },
//                         $$::Z => {
//                             100
//                         },
//                     },
//                 }
//             }
//         }
//     };
// }
//
// fixt_unsigned!(u8, FixTU8);
// fixt_unsigned!(u16, FixTU16);
// fixt_unsigned!(u32, FixTU32);
// fixt_unsigned!(u64, FixTU64);
// fixt_unsigned!(u128, FixTU128);
// fixt_unsigned!(usize, FixTUSize);
//
// macro_rules! fixt_signed {
//     ( $t:ty, $tt:ident ) => {
//         pub enum $tt {
//             Range($t, $t),
//         }
//         impl FixT for $t {
//             type Input = $tt;
//             fn fixt(fixtt: FixTT<Self::Input>) -> Self {
//                 match fixtt {
//                     FixTT::Empty => 0,
//                     FixTT::A => <$t>::min_value(),
//                     FixTT::B => 0,
//                     FixTT::C => <$t>::max_value(),
//                     FixTT::Random => rand::random(),
//                     FixTT::RandomSize(max) => <$t>::fixt(FixTT::Input($tt::Range(0, max as _))),
//                     FixTT::Input(fixt_unsigned) => match fixt_unsigned {
//                         $tt::Range(min, max) => {
//                             let mut rng = rand::thread_rng();
//                             rng.gen_range(min, max)
//                         }
//                     },
//                 }
//             }
//         }
//     };
// }
//
// fixt_signed!(i8, FixTI8);
// fixt_signed!(i16, FixTI16);
// fixt_signed!(i32, FixTI32);
// fixt_signed!(i64, FixTI64);
// fixt_signed!(i128, FixTI128);
// fixt_signed!(isize, FixTISize);
//
// macro_rules! fixt_float {
//     ( $t:ident, $tt:ident ) => {
//         pub enum $tt {
//             Range($t, $t),
//         }
//         impl FixT for $t {
//             type Input = $tt;
//             fn fixt(fixtt: FixTT<Self::Input>) -> Self {
//                 match fixtt {
//                     FixTT::Empty => 0.0,
//                     // NAN is the most common source of bugs in float handling, so it's the first
//                     // thing we should be testing
//                     FixTT::A => std::$t::NAN,
//                     FixTT::B => std::$t::NEG_INFINITY,
//                     FixTT::C => std::$t::INFINITY,
//                     FixTT::Random => rand::random(),
//                     FixTT::RandomSize(max) => {
//                         <$t>::fixt(FixTT::Input($tt::Range(0 as _, max as _)))
//                     }
//                     FixTT::Input(fixt_float) => match fixt_float {
//                         $tt::Range(min, max) => {
//                             let mut rng = rand::thread_rng();
//                             rng.gen_range(min, max)
//                         }
//                     },
//                 }
//             }
//         }
//     };
// }
//
// fixt_float!(f32, FixTF32);
// fixt_float!(f64, FixTF64);
//
// #[macro_export]
// /// a direct delegation of fixtures to the inner type for new types
// macro_rules! newtype_fixt {
//     ( $outer:ty, $inner:ty, $input:ty ) => {
//         impl FixT for $outer {
//             type Input = $input;
//             fn fixt(fixtt: FixTT<Self::Input>) -> Self {
//                 Self(<$inner>::fixt(fixtt))
//             }
//         }
//     };
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use hamcrest2::prelude::*;
//     use rstest::rstest;
//
//     #[rstest(i, o,
//         case(FixTT::default(), ()),
//         case(FixTT::Empty, ()),
//         case(FixTT::A, ()),
//     )]
//     /// tests the values for unit type (which is more limited than even basic_test can handle)
//     fn unit_test(i: FixTT<()>, o: ()) {
//         match i {
//             FixTT::Empty => assert_that!(&<()>::fixt_empty(), eq(&o)),
//             FixTT::A => assert_that!(&<()>::fixt_a(), eq(&o)),
//             _ => {}
//         }
//         assert_that!(&<()>::fixt(i), eq(&o));
//     }
//
//     #[rstest(
//         i,
//         o,
//         case(FixTT::default(), false),
//         case(FixTT::Empty, false),
//         case(FixTT::A, false),
//         case(FixTT::B, true)
//     )]
//     /// tests the values for unit type (which is more limited than even basic_test can handle)
//     fn bool_test(i: FixTT<()>, o: bool) {
//         match i {
//             FixTT::Empty => assert_that!(&<bool>::fixt_empty(), eq(&o)),
//             FixTT::A => assert_that!(&<bool>::fixt_a(), eq(&o)),
//             FixTT::B => assert_that!(&<bool>::fixt_b(), eq(&o)),
//             _ => {}
//         }
//         assert_that!(&<bool>::fixt(i), eq(&o));
//     }
//
//     macro_rules! basic_test {
//         ( $f:ident, $t:ty, $tt:ty, $d:expr, $e:expr, $a:expr, $b:expr, $c:expr ) => {
//             #[rstest(
//                 i,
//                 o,
//                 case(FixTT::default(), $e),
//                 case(FixTT::Empty, $e),
//                 case(FixTT::A, $a),
//                 case(FixTT::B, $b),
//                 case(FixTT::C, $c)
//             )]
//             fn $f(i: FixTT<$tt>, o: $t) {
//                 match i {
//                     FixTT::Empty => assert_that!(&<$t>::fixt_empty(), eq(&o)),
//                     FixTT::A => assert_that!(&<$t>::fixt_a(), eq(&o)),
//                     FixTT::B => assert_that!(&<$t>::fixt_b(), eq(&o)),
//                     FixTT::C => assert_that!(&<$t>::fixt_c(), eq(&o)),
//                     _ => {}
//                 }
//                 assert_that!(&<$t>::fixt(i), eq(&o));
//             }
//         };
//     }
//
//     // function name, type to test, input type, default, empty, a, b, c
//     basic_test!(
//         char_test,
//         char,
//         (),
//         '\u{0000}',
//         '\u{0000}',
//         '\u{2764}',
//         '\u{1F4A9}',
//         '\u{0061}'
//     );
//     basic_test!(
//         string_test,
//         String,
//         (),
//         String::from(""),
//         String::from(""),
//         String::from("❤💩a"),
//         String::from("foo"),
//         String::from("bar")
//     );
//
//     macro_rules! unsigned_test {
//         ( $f:ident, $t:ty, $tt:ty ) => {
//             basic_test!($f, $t, $tt, 0, 0, <$t>::min_value(), 1, <$t>::max_value());
//         };
//     }
//
//     unsigned_test!(u8_test, u8, FixTU8);
//     unsigned_test!(u16_test, u16, FixTU16);
//     unsigned_test!(u32_test, u32, FixTU32);
//     unsigned_test!(u64_test, u64, FixTU64);
//     unsigned_test!(u128_test, u128, FixTU128);
//     unsigned_test!(usize_test, usize, FixTUSize);
//
//     macro_rules! signed_test {
//         ( $f:ident, $t:ty, $tt:ty ) => {
//             basic_test!($f, $t, $tt, 0, 0, <$t>::min_value(), 0, <$t>::max_value());
//         };
//     }
//
//     signed_test!(i8_test, i8, FixTI8);
//     signed_test!(i16_test, i16, FixTI16);
//     signed_test!(i32_test, i32, FixTI32);
//     signed_test!(i64_test, i64, FixTI64);
//     signed_test!(i128_test, i128, FixTI128);
//     signed_test!(isize_test, isize, FixTISize);
//
//     macro_rules! float_test {
//         ( $f:ident, $t:ident, $tt:ty ) => {
//             #[rstest(
//                                                     i,
//                                                     o,
//                                                     case(FixTT::default(), 0.0),
//                                                     case(FixTT::Empty, 0.0),
//                                                     // hit NAN directly
//                                                     // case(FixTT::A, $a),
//                                                     case(FixTT::B, std::$t::NEG_INFINITY),
//                                                     case(FixTT::C, std::$t::INFINITY)
//                                                 )]
//             fn $f(i: FixTT<$tt>, o: $t) {
//                 match i {
//                     FixTT::Empty => assert_that!(&<$t>::fixt_empty(), eq(&o)),
//                     // FixTT::A => assert_that!(&<$t>::fixt_a(), eq(&o)),
//                     FixTT::B => assert_that!(&<$t>::fixt_b(), eq(&o)),
//                     FixTT::C => assert_that!(&<$t>::fixt_c(), eq(&o)),
//                     _ => {}
//                 }
//                 assert_that!(&<$t>::fixt(i), eq(&o));
//
//                 // this is redundantly called every case but it doesn't matter, we get NAN coverage
//                 assert_that!(<$t>::fixt(FixTT::A).is_nan(), is(true));
//                 assert_that!(<$t>::fixt_a().is_nan(), is(true));
//             }
//         };
//     }
//     float_test!(f32_test, f32, FixTF32);
//     float_test!(f64_test, f64, FixTF64);
//
//     /// show an example of a newtype delegating to inner fixtures
//     #[derive(Debug, PartialEq)]
//     struct MyNewType(u32);
//     newtype_fixt!(MyNewType, u32, FixTU32);
//     basic_test!(
//         new_type_test,
//         MyNewType,
//         FixTU32,
//         MyNewType(0),
//         MyNewType(0),
//         MyNewType(<u32>::min_value()),
//         MyNewType(1),
//         MyNewType(<u32>::max_value())
//     );
// }
