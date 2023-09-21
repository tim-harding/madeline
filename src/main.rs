mod dag;
mod jit;

use core::mem;
use cranelift_module::ModuleError;
use dag::{Dag, Intrinsic, Node, NodeKind};

fn main() -> Result<(), ModuleError> {
    let mut jit = jit::Jit::default();
    let mut program = Dag::new();
    let one = program.add_vertex(Node::with_kind(NodeKind::Constant(1.)));
    let two = program.add_vertex(Node::with_kind(NodeKind::Constant(2.)));
    let three = program.add_vertex(Node::with_kind(NodeKind::Constant(3.)));
    let add = program.add_vertex(Node::with_kind(NodeKind::Intrinsic(Intrinsic::Add(
        one, two,
    ))));
    let mul = program.add_vertex(Node::with_kind(NodeKind::Intrinsic(Intrinsic::Mul(
        add, three,
    ))));
    program.set_out_node(mul);
    let v: f32 = unsafe { run_code(&mut jit, &program, ()) }?;
    println!("{v}");
    Ok(())
}

unsafe fn run_code<I, O>(jit: &mut jit::Jit, code: &Dag, input: I) -> Result<O, ModuleError> {
    let code_ptr = jit.compile(code)?;
    let code_fn = mem::transmute::<_, fn(I) -> O>(code_ptr);
    Ok(code_fn(input))
}
