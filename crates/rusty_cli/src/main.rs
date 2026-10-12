use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("running rusty cloud CLI");
    for ii in args.iter() {
        println!("passed arguements were: {}", ii);
    }

}
