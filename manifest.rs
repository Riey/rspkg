use std::env;

fn main() {
    let mut args = env::args();
    args.next();
    let command = args.next().unwrap();

    match command.as_str() {
        "build" => {
            let dummy = "{ \"out\": \".\" }";
            println!("{}", dummy);
        }
        "dependencies" => println!("[]"),
        _ => panic!("Unknown command: {}", command),
    }
}
