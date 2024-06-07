#[allow(warnings)]
mod bindings;

use clap::Parser;
use std::fmt;

use bindings::tut::calculator::{calculate, calculate::Op};

fn parse_operator(op: &str) -> anyhow::Result<Op> {
    match op {
        "add" => Ok(Op::Add),
        "mult" => Ok(Op::Mult),
        _ => anyhow::bail!("Unknown operation: {}", op),
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Op::Add => write!(f, "+"),
            Op::Mult => write!(f, "*"),
        }
    }
}

/// A CLI for executing mathematical expressions
/// using WebAssembly
#[derive(Parser)]
#[clap(name = "calculator", version = env!("CARGO_PKG_VERSION"))]
struct Command {
    /// The first operand
    x: u32,
    /// The second operand
    y: u32,
    /// Expression operator
    #[clap(value_parser = parse_operator)]
    op: Op,
}

impl Command {
    fn run(self) {
        let res = calculate::eval_expression(self.op, self.x, self.y);
        println!("{} {} {} = {res}", self.x, self.op, self.y);
    }
}

fn main() {
    let args = Command::parse();
    args.run();
}

