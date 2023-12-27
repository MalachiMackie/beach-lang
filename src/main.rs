mod ast;

use ast::builder::Builder;

fn main() {
    let builder = Builder::new();

    println!("{:?}", builder);
}
