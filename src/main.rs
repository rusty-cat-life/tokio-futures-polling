use std::env;
use tokio_futures_polling::{run, Config};

fn main() {
    let config = Config::new(env::args());

    run(config);
}