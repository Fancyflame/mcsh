use ir::simulate::SimulateResult;
use parse::parse_file;

use crate::atoi::Atoi;

mod atoi;
mod ir;
mod parse;

fn main() {
    let defs = parse_file(
        "
    // start of file

    const FOO = 12;

    fn call(a, b) {
        return a+b;
    }

    pub fn test(lhs, rhs) {
        let a = -20;
        let b = 30;

        if !(1 != 2 + 2) {
            call(1 + 1, 2*7);
        } else if a==b {
            call(1, 1);
        }else{
            // 一个中文注释
        }

        while a < 5 {
            a = a+1;
            if a > 35 {
                continue;
            }
            b=b+1;
        }

        a >< b;
        return call(a, b);
    }
    ",
    )
    .unwrap();

    let mut stack = Atoi::new();
    stack.parse(&defs).unwrap();
    let lm = stack.finish();
    let SimulateResult { result, log } = lm.simulate_pub("test");
    println!("log:\n{log}");
    println!("result: {result:?}");
}
