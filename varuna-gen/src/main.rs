use minijinja::Environment;
use std::{collections::BTreeMap, env};
use varuna_gen::gen_target;
fn main() {
    let args = get_args();
    let target_id = args
        .get("target_id")
        .map(|s| s.parse::<usize>().expect("failed to parse target_id"));
    gen_target("./", args, target_id, &Environment::new).unwrap();
}

fn get_args() -> BTreeMap<String, String> {
    let mut args = BTreeMap::new();
    for arg in env::args().skip(1) {
        let mut parts = arg.splitn(2, '=');
        let key = parts.next().expect("failed to get key");
        let value = parts.next().unwrap_or("");
        args.insert(key.to_string(), value.to_string());
    }
    args
}
