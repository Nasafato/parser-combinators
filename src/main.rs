use parser_combinators as pc;

fn main() {
    // let input = r"a";
    let input = r"b";
    match pc::the_letter_a(input) {
        Ok((input, output)) => println!("{} => ", input,),
        Err(err) => println!("Err: {}", err),
    };
}
