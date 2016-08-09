/// Apply a parser over every element of an iterable.
macro_rules! foreach {
    ($input:expr, $iterable:expr, $element:ident => $submac:ident!( $($args:tt)* )) => {{
        let mut input = $input;
        for $element in $iterable.into_iter() {
            match $submac!(input, $($args)*) {
                $crate::nom::IResult::Done(new_input, _) => {
                    input = new_input;
                },
                $crate::nom::IResult::Error(_) => {
                    return $crate::nom::IResult::Error($crate::nom::Err::Position($crate::nom::ErrorKind::Many0, $input));
                },
                $crate::nom::IResult::Incomplete(_) => {
                    return $crate::nom::IResult::Incomplete($crate::nom::Needed::Unknown);
                },
            }
        }

        $crate::nom::IResult::Done(input, ())
    }};
}

/// Makes a function from a parser combination with arguments.
macro_rules! named_args {
    (pub $func_name:ident ( $( $arg:ident : $typ:ty ),* ) < $return_type:ty > , $submac:ident!( $($args:tt)* ) ) => {
        pub fn $func_name(input: &[u8], $( $arg : $typ ),*) -> IResult<&[u8], $return_type> {
            $submac!(input, $($args)*)
        }
    };
    (pub $func_name:ident < 'a > ( $( $arg:ident : $typ:ty ),* ) < $return_type:ty > , $submac:ident!( $($args:tt)* ) ) => {
        pub fn $func_name<'a>(input: &'a [u8], $( $arg : $typ ),*) -> IResult<&'a [u8], $return_type> {
            $submac!(input, $($args)*)
        }
    };
    ($func_name:ident ( $( $arg:ident : $typ:ty ),* ) < $return_type:ty > , $submac:ident!( $($args:tt)* ) ) => {
        fn $func_name(input: &[u8], $( $arg : $typ ),*) -> IResult<&[u8], $return_type> {
            $submac!(input, $($args)*)
        }
    };
    ($func_name:ident < 'a > ( $( $arg:ident : $typ:ty ),* ) < $return_type:ty > , $submac:ident!( $($args:tt)* ) ) => {
        fn $func_name<'a>(input: &'a [u8], $( $arg : $typ ),*) -> IResult<&'a [u8], $return_type> {
            $submac!(input, $($args)*)
        }
    };
}

/// Apply a parser if the condition is true. Otherwise, map the input with the expression.
macro_rules! parse_if_else {
    ($input:expr, $cond:expr, $parser:ident, $mapper:expr) => {{
        if $cond {
            $parser($input)
        }
        else {
            let (input, output) = $input.split_at(0);
            $crate::nom::IResult::Done(input, $mapper(output))
        }
    }};
}
