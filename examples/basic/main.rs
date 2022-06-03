use std::env;
use std::io::{self, Write};

mod basic;
mod cookie;
mod data;
mod error_handling;
mod header;
mod logging;
mod middleware;
mod path_prams;
mod rate_limit;
mod routing;
mod serve_file;
mod serve_static;
mod state;
mod threading;

pub trait Example {
    fn name(&self) -> &'static str;
    fn exec(&self);
}

fn main() {
    let examples: Vec<Box<dyn Example>> = vec![
        Box::new(basic::Basic),
        Box::new(serve_file::ServeFile),
        Box::new(routing::Routing),
        Box::new(data::Data),
        Box::new(path_prams::PathParam),
        Box::new(header::Header),
        Box::new(state::State),
        Box::new(cookie::Cookie),
        Box::new(error_handling::ErrorHandling),
        Box::new(serve_static::ServeStatic),
        Box::new(middleware::MiddlewareExample),
        Box::new(logging::Logging),
        Box::new(rate_limit::RateLimit),
        Box::new(threading::Threading),
    ];

    if let Some(run_arg) = env::args().nth(1) {
        return examples
            .iter()
            .find(|x| x.name() == run_arg)
            .unwrap()
            .exec();
    };

    for (i, item) in examples.iter().enumerate() {
        println!(
            "[{: >w$}] {}",
            i,
            item.name(),
            w = place_count(examples.len() - 1)
        );
    }

    let run_index = input("\n[*] Index ❯ ").unwrap();
    let run_index = match run_index.parse::<usize>() {
        Ok(i) => i,
        Err(_) => return println!("[*] Das not a number..."),
    };

    if run_index >= examples.len() {
        return println!("[*] Invaild Id");
    }

    println!("[*] Starting `{}`\n", examples[run_index].name());
    examples[run_index].exec();
}

fn place_count(mut inp: usize) -> usize {
    let mut inc = 1;
    while inp >= 10 {
        inp /= 10;
        inc += 1;
    }

    inc
}

fn input(inp: &str) -> Option<String> {
    print!("{}", inp);

    let mut buff = String::new();
    io::stdout().flush().ok()?;
    io::stdin().read_line(&mut buff).ok()?;
    while buff.ends_with('\n') || buff.ends_with('\r') {
        buff.pop();
    }

    Some(buff)
}
