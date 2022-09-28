use std::{thread::sleep, time::Duration};

use afkrip::input::{Mouse, errors::MouseError};
use rand::Rng;

fn error_exit(err: MouseError) -> ! {
    error(err);
    std::process::exit(1)
}

fn error(error: MouseError) -> () {
    println!("{}", error.message());
}

fn main() {
    let mut mouse = match Mouse::new() {
        Ok(instance) => instance,
        Err(error) => error_exit(error) 
    };

    loop {
        sleep(Duration::from_secs(1));
        let idle = rs_idle::get_idle_time();
        let mut rng = rand::thread_rng();
        if idle > 5000 {
            let x: i32 = rng.gen_range(-50..50);
            let y: i32 = rng.gen_range(-50..50);
            println!("Idle: {}ms, moving mouse by x: {}, y: {}", idle, x, y);
            mouse.pointer_move(x, y).unwrap_or_else(error);
        }
    }
}
