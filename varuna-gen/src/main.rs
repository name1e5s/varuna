use varuna_gen::config::Config;

fn main() {
    println!("Hello, world!");
    let c = Config::from_file("varuna-gen/example-package/example.toml").unwrap();
    println!("{:?}", c);
}
