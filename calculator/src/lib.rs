#[allow(warnings)]
mod bindings;

use bindings::tut::adder::add::{add, mult};

use bindings::exports::tut::calculator::calculate::{Guest, Op};

struct Component;

impl Guest for Component {
    fn eval_expression(op: Op, x: u32, y: u32) -> u32 {
        match op {
            Op::Add => add(x, y),
            Op::Mult => mult(x, y),
        }
    }
}

bindings::export!(Component with_types_in bindings);
