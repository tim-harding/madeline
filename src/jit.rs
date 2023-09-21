use crate::dag::{Dag, Intrinsic, Node, NodeKind};
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module, ModuleError};
use std::collections::HashSet;

const FLOAT: cranelift::codegen::ir::Type = cranelift::codegen::ir::types::F32;

pub struct Jit {
    builder_context: FunctionBuilderContext,
    ctx: codegen::Context,
    module: JITModule,
}

impl Default for Jit {
    fn default() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();
        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

        let module = JITModule::new(builder);
        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            module,
        }
    }
}

impl Jit {
    pub fn compile(&mut self, dag: &Dag) -> Result<*const u8, ModuleError> {
        self.translate(dag);
        let id = self
            .module
            // TODO: Pick a proper function name
            .declare_function("test", Linkage::Export, &self.ctx.func.signature)?;

        // Define the function to jit. This finishes compilation, although
        // there may be outstanding relocations to perform. Currently, jit
        // cannot finish relocations until all functions to be called are
        // defined. For this toy demo for now, we'll just finalize the
        // function below.
        self.module.define_function(id, &mut self.ctx)?;

        // Now that compilation is finished, we can clear out the context state.
        self.module.clear_context(&mut self.ctx);

        // Finalize the functions which we just defined, which resolves any
        // outstanding relocations (patching in addresses, now that they're
        // available).
        self.module.finalize_definitions().unwrap();

        let code = self.module.get_finalized_function(id);
        Ok(code)
    }

    fn translate(&mut self, dag: &Dag) {
        let inputs: Vec<_> = dag
            .iter()
            .filter_map(|(id, node)| (node.kind == NodeKind::Input).then_some(*id))
            .collect();

        for input in inputs.iter() {
            self.ctx.func.signature.params.push(AbiParam::new(FLOAT));
        }

        self.ctx.func.signature.returns.push(AbiParam::new(FLOAT));

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        for (i, input) in inputs.iter().enumerate() {
            let variable = Variable::from_u32(*input);
            builder.declare_var(variable, FLOAT);
            builder.def_var(variable, builder.block_params(entry_block)[i]);
        }

        let mut translator = Translator {
            builder,
            dag,
            defined_variables: HashSet::new(),
        };
        translator.translate(dag.out_node());
        let mut builder = translator.into_builder();

        let return_variable = Variable::from_u32(dag.out_node());
        let return_value = builder.use_var(return_variable);
        builder.ins().return_(&[return_value]);
        builder.finalize();
    }
}

struct Translator<'a> {
    builder: FunctionBuilder<'a>,
    dag: &'a Dag,
    // TODO: Reuse allocation
    defined_variables: HashSet<u32>,
}

impl<'a> Translator<'a> {
    pub fn translate(&mut self, node: u32) -> Value {
        let node_id = node;
        let binding = Node::with_kind(NodeKind::Constant(0.));
        let node = self.dag.node(node).unwrap_or(&binding);
        match node.kind {
            NodeKind::Passthrough(input) => self.translate(input),

            NodeKind::Constant(constant) => self.builder.ins().f32const(constant),

            NodeKind::Intrinsic(intrinsic) => {
                let variable = Variable::from_u32(node_id);
                if !self.defined_variables.contains(&node_id) {
                    self.defined_variables.insert(node_id);
                    self.builder.declare_var(variable, FLOAT);
                    let value = match intrinsic {
                        Intrinsic::Add(a, b) => {
                            let a = self.translate(a);
                            let b = self.translate(b);
                            self.builder.ins().fadd(a, b)
                        }
                        Intrinsic::Sub(a, b) => {
                            let a = self.translate(a);
                            let b = self.translate(b);
                            self.builder.ins().fsub(a, b)
                        }
                        Intrinsic::Mul(a, b) => {
                            let a = self.translate(a);
                            let b = self.translate(b);
                            self.builder.ins().fmul(a, b)
                        }
                        Intrinsic::Div(a, b) => {
                            let a = self.translate(a);
                            let b = self.translate(b);
                            self.builder.ins().fdiv(a, b)
                        }
                    };
                    self.builder.def_var(variable, value);
                }
                self.builder.use_var(variable)
            }

            NodeKind::Input => {
                let variable = Variable::from_u32(node_id);
                self.builder.use_var(variable)
            }
        }
    }

    pub fn into_builder(self) -> FunctionBuilder<'a> {
        self.builder
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone, Copy)]
pub enum TranslationError {
    #[error("Node {0} is missing an input")]
    MissingInput(u32),
}
