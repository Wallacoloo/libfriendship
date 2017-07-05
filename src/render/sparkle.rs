//! Sparkle is a JIT-based renderer that uses llvm to compile effects as they
//! are loaded.
//! Other than the JIT aspect, it is mostly a literal reimplementation of
//! the reference renderer.

use std::collections::HashMap;
use std::ffi::CString;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;

use jagged_array::Jagged2;
use llvm;
use llvm::{Builder, Context, ContextType, ExecutionEngine, Function, Module};
use llvm_sys;
use llvm_sys::core::{
    LLVMContextCreate,
    LLVMContextDispose,
    LLVMDisposeModule,
    LLVMModuleCreateWithNameInContext,
    LLVMStructCreateNamed,
    LLVMStructSetBody,
};
use llvm_sys::{
    LLVMIntPredicate,
    LLVMRealPredicate,
};
use llvm_sys::prelude::*;
use llvm_sys::target::{LLVM_InitializeNativeTarget, LLVM_InitializeNativeAsmPrinter,
                   LLVM_InitializeNativeAsmParser};
use ndarray::Array2;
use streaming_iterator::StreamingIterator;

use render::Renderer;
use resman::AudioBuffer;
use routing::{Edge, Effect, GraphWatcher, NodeData, NodeHandle, RouteGraph};
use routing::effect::{PrimitiveEffect, EffectData};



#[derive(Debug)]
pub struct SparkleRenderer {
    // Top-level node map
    nodes: NodeMap,
    /// inputs[slot][sample] represents the value of the external input at
    /// the given slot and time. On OOB, it is assumed to be 0.
    inputs: Vec<Vec<f32>>,
    /// Next expected sample to be queried.
    /// This is tracked because if we do a seek, the inputs need to be zero'd.
    head: u64,
    // LLVM data below
    /// Llvm execution engines. Zipped against the modules.
    llvm_engines: Vec<ExecutionEngine>,
    /// Finalized llvm modules.
    llvm_modules: Vec<Module>,
    /// Module that has yet to be compiled.
    open_module: Option<Module>,
    /// LLVM struct { fn(time, slot, callback_type*)->f32, callback_type* }
    /// Used to pass callbak functions into get_output() to allow effects to access their inputs.
    callback_type: LLVMTypeRef,
    /// LLVM type for fn(time, slot, input_getter: callback_type*) -> f32
    sample_getter_type: LLVMTypeRef,
    /// Object that lets us build IR into basic blocks.
    llvm_builder: Builder,
    // NOTE: LLVM Context must be last member, otherwise jemalloc will try dropping
    // llvm-owned data
    /// Object that provides a context for LLVM calls.
    llvm_ctx: Context,
}

#[derive(Debug, Default)]
struct NodeMap {
    nodes: HashMap<NodeHandle, Node>,
    output_edges: Vec<Option<Edge>>,
}

#[derive(Debug)]
struct Node {
    data: MyNodeData,
    /// Inbound edges, indexed by slot idx.
    inbound: Vec<Option<Edge>>,
}

/// Struct to help build LLVM code for primitive effects.
struct FnBuilder<'a> {
    func: Function,
    ctx: &'a Context,
    builder: &'a mut Builder,
}

#[derive(Debug)]
enum MyNodeData {
    /// The node corresponds to a LLVM function with the provided name.
    LlvmFunc(String),
    /// External audio.
    Buffer(AudioBuffer),
}

#[derive(Copy, Clone)]
struct CallbackType {
    input_getter: *const fn(u64, u32, *const CallbackType) -> f32,
    userdata: *const CallbackType,
}

impl Renderer for SparkleRenderer {
    fn fill_buffer(&mut self, buff: &mut Array2<f32>, idx: u64, inputs: Jagged2<f32>) {
        let (n_slots, n_times) = buff.dim().into();
        // Store inputs for future use
        {
            // If this is a seek operation, forget historical input values.
            if idx != self.head {
                for slot in &mut self.inputs {
                    *slot = Vec::new();
                    // NB: Improper indexing on 32-bit OS, but will OOM first.
                    slot.resize(idx as usize, 0f32);
                }
            }
            // Make sure we have storage space for all inputs.
            while self.inputs.len() < buff.len() {
                // NB: Improper indexing on 32-bit OS, but will OOM first.
                let mut v = Vec::with_capacity(idx as usize);
                v.resize(idx as usize, 0f32);
                self.inputs.push(v);
            }
            let mut stream = inputs.stream();
            let mut self_it = self.inputs.iter_mut();
            while let (Some(row), Some(vec_dest)) = (stream.next(), self_it.next()) {
                assert_eq!(vec_dest.len(), idx as usize);
                vec_dest.extend(row);
                assert!(vec_dest.len() <= idx as usize + n_times); // cannot send inputs ahead of outputs.
                let pad_val = vec_dest.last().cloned().unwrap_or(0f32);
                vec_dest.resize(idx as usize +n_times, pad_val);
            }
        }

        // Calculate outputs
        self.prep_execution();
        for slot in 0..n_slots as u32 {
            for time in idx..idx+n_times as u64 {
                buff[[slot as usize, (time - idx) as usize]] = self.get_sample(time, slot);
            }
        }
        // Keep track of the playhead
        self.head = idx + n_times as u64;
    }
}

impl GraphWatcher for SparkleRenderer {
    fn on_add_node(&mut self, handle: &NodeHandle, data: &NodeData) {
        let my_node_data = self.make_node(data);
        self.nodes.insert(*handle, Node::new(my_node_data));
    }
    fn on_del_node(&mut self, handle: &NodeHandle) {
        self.nodes.remove(handle);
    }
    fn on_add_edge(&mut self, edge: &Edge) {
        self.nodes.add_edge(edge);
    }
    fn on_del_edge(&mut self, edge: &Edge) {
        let inbound = if edge.to_full().is_toplevel() {
            &mut self.nodes.output_edges
        } else {
            &mut self.nodes.get_mut(&edge.to_full()).expect("Attempt to delete edge, but it was never created!").inbound
        };
        if let Some(stored_edge) = inbound.get_mut(edge.to_slot() as usize) {
            *stored_edge = None
        }
    }
}

impl SparkleRenderer {
    /// Creates a LLVM function with signature:
    /// fn get_sample(time: u64, slot: u32, input_getter: &callback_type) -> f32) -> f32
    /// callback_type should be { input_getter, userdata },
    /// Returns the name of the function that can be used to get the effect's output.
    fn jit_effect(&mut self, effect: &Effect) -> String {
        let fname = format!("{}_get_output", effect.id().name());
        println!("jit: {}", fname);
        if self.get_fn(&fname).is_none() {
            // Effect hasn't been compiled yet; do so.
            let func = self.open_get_sample_fn(&fname);
            let mut fnbuilder = FnBuilder::new(func, &self.llvm_ctx, &mut self.llvm_builder);
            match *effect.data() {
                EffectData::Primitive(prim) => match prim {
                    PrimitiveEffect::F32Constant => fnbuilder.build_f32constant(),
                    PrimitiveEffect::Delay => fnbuilder.build_delay(),
                    PrimitiveEffect::Multiply => fnbuilder.build_multiply(),
                    PrimitiveEffect::Sum2 => fnbuilder.build_sum2(),
                    PrimitiveEffect::Divide => fnbuilder.build_divide(),
                    PrimitiveEffect::Minimum => fnbuilder.build_minimum(),
                    PrimitiveEffect::Modulo => fnbuilder.build_modulo(),
                },
                EffectData::RouteGraph(ref graph) => {
                    // TODO: for now, just use a dummy return value
                    //let ret = self.llvm_ctx.cons(20f32);
                    //builder.build_ret(ret);
                    /*bb_ret0 = Some(self.llvm_ctx.append_basic_block(&mut func, &(fname.clone() + "_ret0")));
                    let blocks: Vec<_> = graph.iter_outbound_edges().map(|edge| {
                        let br_name = format!("{}_edge_n{}s{}_n{}s{}", &fname.clone(),
                            edge.from_full(), edge.from_slot(), edge.to_full(), edge.to_slot());
                        let edgematch_bb = self.llvm_ctx.append_basic_block(&mut func, &br_name);
                        let desired_slot = self.llvm_ctx.cons(edge.to_slot());
                        (desired_slot, edgematch_bb)
                    }).collect();
                    builder.build_switch(slot, bb_ret0.unwrap(), blocks);*/
                },
                _ => panic!("Cannot JIT effect: {:?}", effect)
            }
        }

        fname
    }
    fn open_get_sample_fn(&mut self, fname: &str) -> Function {
        let ty = self.sample_getter_type;
        self.get_open_module().add_function(ty, fname)
    }
    /// Get or create an editable module
    fn get_open_module(&mut self) -> &mut Module {
        // TODO: use entry API
        if let Some(ref mut module) = self.open_module {
            module
        } else {
            self.open_module = Some(
                self.llvm_ctx.module_create_with_name(
                    &format!("mod{}", self.llvm_modules.len()))
            );
            self.open_module.as_mut().unwrap()
        }
    }
    /// Return a pointer to the compiled function with the provided name.
    /// Search across all execution engines.
    fn get_fn_ptr(&self, name: &str) -> Option<extern "C" fn()> {
        self.llvm_engines.iter().flat_map(|ee| {
            ee.get_function_address(name).into_iter()
        }).next()
    }
    /// Return the LLVM handle to the function with the given name, if any such exists.
    fn get_fn(&mut self, name: &str) -> Option<Function> {
        self.open_module.iter_mut().chain(self.llvm_modules.iter_mut()).filter_map(|module| {
            module.get_named_function(name)
        }).next()
    }
    /// Compile all outstanding functions.
    fn prep_execution(&mut self) {
        // IF there's an open module, compile it.
        if let Some(module) = self.open_module.take() {
            let ee = {
                module.dump();
                llvm::ExecutionEngine::create_for_module(&module).unwrap()
            };
            self.llvm_engines.push(ee);
        }
    }
    /// Allocate renderer data based on data from a RouteGraph node.
    fn make_node(&mut self, effect: &NodeData) -> MyNodeData {
        match *effect.data() {
            EffectData::Buffer(ref buff) => MyNodeData::Buffer(buff.clone()),
            EffectData::Primitive(_e) =>
                MyNodeData::LlvmFunc(self.jit_effect(effect)),
            EffectData::RouteGraph(ref _graph) =>
                MyNodeData::LlvmFunc(self.jit_effect(effect)),
        }
    }
    /// Get the output at a particular time and to a particular output slot.
    fn get_sample(&mut self, time: u64, slot: u32) -> f32 {
        let out_edge = self.nodes.output_edges.get(slot as usize);
        self.get_maybe_edge_value(time, out_edge)
    }
    /// Wrapper around `get_edge_value` that will return 0f32 if maybe_edge is not
    /// `Some(&Some(edge))`.
    fn get_maybe_edge_value(&self, time: u64,
        maybe_edge: Option<&Option<Edge>>) -> f32
    {
        if let Some(&Some(ref edge)) = maybe_edge {
            self.get_edge_value(time, &edge)
        } else {
            // Edge doesn't exist; value is zero.
            0f32
        }
    }
    /// Get the value on an edge at a specific time.
    /// This will recurse down, all the way to the input to this node itself.
    fn get_edge_value(&self, time: u64, edge: &Edge) -> f32 {
        let from = edge.from_full();
        let from_slot = edge.from_slot();
        if *from.node_handle() == None {
            println!("Read from input: {}, {}", time, from_slot);
            // reading from an input
            *self.inputs.get(from_slot as usize)
                .and_then(|v| v.get(time as usize))
                .unwrap_or(&0f32)
        } else {
            // Reading from another node within the DAG
            let node = &self.nodes[&from];
            match node.data {
                MyNodeData::LlvmFunc(ref fname) => {
                    let out_getter = self.get_fn_ptr(fname);
                    out_getter.map(|getter| unsafe {
                        let in_edge_getter = |time2: u64, slot2: u32| {
                            // get the input to this node.
                            let in_edge = node.inbound.get(slot2 as usize);
                            self.get_maybe_edge_value(time2, in_edge)
                        };
                        let f: extern "C" fn(u64, u32, *const CallbackType) -> f32 = mem::transmute(getter);
                        let callback = CallbackType {
                            input_getter: call_closure_from_c as *const fn(u64, u32, *const CallbackType) -> f32,
                            userdata: &mem::transmute(&in_edge_getter as &Fn(u64, u32) -> f32),
                        };
                        f(time, from_slot, &callback)
                    }).unwrap()
                }
                MyNodeData::Buffer(ref buf) => buf.get(time, from_slot),
            }
        }
    }
}

extern "C" fn call_closure_from_c(time: u64, slot: u32, closure_info: *const CallbackType) -> f32 {
    unsafe {
        let closure: &Fn(u64, u32) -> f32 = mem::transmute(*closure_info);
        closure(time, slot)
    }
}


impl Default for SparkleRenderer {
    fn default() -> SparkleRenderer {
        // Initialize core LLVM features
        llvm::link_in_mcjit();
        llvm::initialize_native_target();
        llvm::initialize_native_asm_printer();

        let llvm_ctx = Context::new();
        let llvm_builder = llvm_ctx.create_builder();
        let llvm_modules = Default::default();
        let llvm_engines = Default::default();
        let open_module = None;

        // Create the callback_type struct.
        // It is recursive, and has a dependency on some fn-ptrs,
        // so its creation is staggered.
        let callback_type = {
            let c_name = CString::new("SampleGetter").unwrap();
            unsafe {
                LLVMStructCreateNamed(llvm_ctx.ptr, c_name.as_ptr())
            }
        };

        let sample_getter_type = llvm::function_type(
            f32::get_type_in_context(&llvm_ctx),
            vec![
                u64::get_type_in_context(&llvm_ctx),
                u32::get_type_in_context(&llvm_ctx),
                llvm::pointer_type(callback_type, 0)
            ],
        /* is_var_arg */false);

        unsafe {
            let mut element_types = vec![llvm::pointer_type(sample_getter_type, 0), llvm::pointer_type(callback_type, 0)];
            let is_packed = false;
            LLVMStructSetBody(callback_type, element_types.as_mut_ptr(),
                element_types.len() as u32, is_packed as i32)
        }

        let (head, inputs, nodes) = Default::default();

        SparkleRenderer {
            head, inputs, nodes,
            llvm_ctx, llvm_builder, llvm_modules, llvm_engines, open_module, callback_type, sample_getter_type
        }
    }
}

impl NodeMap {
    /// Add an edge that connects two nodes within this graph.
    fn add_edge(&mut self, edge: &Edge) {
        let inbound = if edge.to_full().is_toplevel() {
            &mut self.output_edges
        } else {
            &mut self.nodes.get_mut(&edge.to_full()).unwrap().inbound
        };
        let slot: u32 = edge.to_slot().into();
        let slot = slot as usize;
        // allocate space to store the edge.
        if inbound.len() <= slot { inbound.resize(slot+1, None) }

        inbound[slot] = Some(edge.clone());
    }
}

impl Node {
    fn new(data: MyNodeData) -> Self {
        Node {
            data: data,
            inbound: Vec::new(),
        }
    }
}

impl Deref for NodeMap {
    type Target = HashMap<NodeHandle, Node>;
    fn deref(&self) -> &Self::Target {
        &self.nodes
    }
}

impl DerefMut for NodeMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.nodes
    }
}


impl<'a> FnBuilder<'a> {
    fn new(mut func: Function, ctx: &'a Context, builder: &'a mut Builder) -> Self {
        let bb = ctx.append_basic_block(&mut func, "entry_point");
        builder.position_at_end(bb);
        Self{ func, ctx, builder }
    }
    /// Perform the computations associated with PrimitiveEffect::F32Constant
    fn build_f32constant(&mut self) {
        let f32_type = f32::get_type_in_context(&self.ctx);
        let slot = self.slot();
        let slot_as_f32 = self.builder.build_bit_cast(slot, f32_type, "slot_as_f32");
        self.builder.build_ret(slot_as_f32);
    }
    /// Perform the computations associated with PrimitiveEffect::Delay
    fn build_delay(&mut self) {
        self.guard_slot_ne_0();
        let time = self.time();
        // Amount to delay input by
        let delay_frames = self.read_input(time, 1);
        let delay_frames_u64 = self.checked_fp_to_u64(delay_frames, "delay_frames_u64");
        let delayed_time = self.checked_sub(time, delay_frames_u64, "delayed_time");
        let result = self.read_input(delayed_time, 0);
        self.builder.build_ret(result);
    }
    /// Perform the computations associated with PrimitiveEffect::Multiply
    fn build_multiply(&mut self) {
        self.guard_slot_ne_0();
        let time = self.time();
        let (input0, input1) = self.read_inputs(time);
        let result = self.builder.build_fmul(input0, input1, "result");
        self.builder.build_ret(result);
    }
    /// Perform the computations associated with PrimitiveEffect::Sum2
    fn build_sum2(&mut self) {
        self.guard_slot_ne_0();
        let time = self.time();
        let (input0, input1) = self.read_inputs(time);
        let result = self.builder.build_fadd(input0, input1, "result");
        self.builder.build_ret(result);
    }
    /// Perform the computations associated with PrimitiveEffect::Divide
    fn build_divide(&mut self) {
        self.guard_slot_ne_0();
        let time = self.time();
        let (input0, input1) = self.read_inputs(time);
        let result = self.builder.build_fdiv(input0, input1, "result");
        self.builder.build_ret(result);
    }
    /// Perform the computations associated with PrimitiveEffect::Minimum
    fn build_minimum(&mut self) {
        self.guard_slot_ne_0();
        let time = self.time();
        let (input0, input1) = self.read_inputs(time);
        let is_s0_lt_s1 = self.builder.build_fcmp(LLVMRealPredicate::LLVMRealULT, input0, input1, "is_s0_lt_s1");
        let result = self.builder.build_select(is_s0_lt_s1, input0, input1, "result");
        self.builder.build_ret(result);
    }
    /// Perform the computations associated with PrimitiveEffect::Modulo
    fn build_modulo(&mut self) {
        let f32_0 = self.ctx.cons(0f32);
        self.guard_slot_ne_0();
        let time = self.time();
        let (input0, input1) = self.read_inputs(time);
        let signed_result = self.builder.build_frem(input0, input1, "signed_result");
        // `signed_result` has same sign as dividend. If negative, correct that.
        let result_if_neg = self.builder.build_fadd(signed_result, input1, "result_if_neg");
        let is_result_neg = self.builder.build_fcmp(LLVMRealPredicate::LLVMRealULT, signed_result, f32_0, "is_result_neg");
        let result = self.builder.build_select(is_result_neg, result_if_neg, signed_result, "result");
        self.builder.build_ret(result);
    }
    /// Unpack the function's `time` argument.
    fn time(&self) -> LLVMValueRef {
        self.func.get_param(0).unwrap()
    }
    /// Unpack the function's `slot` argument.
    fn slot(&self) -> LLVMValueRef {
        self.func.get_param(1).unwrap()
    }
    /// Unpack the function's callback ptr/data argument.
    fn in_getter(&self) -> LLVMValueRef {
        self.func.get_param(2).unwrap()
    }
    /// Insert code to test if slot != 0 and return 0f32 if true.
    fn guard_slot_ne_0(&mut self) {
        let u32_0 = self.ctx.cons(0u32);
        let f32_0 = self.ctx.cons(0f32);
        let slot = self.slot();
        let bb_nonzero = self.ctx.append_basic_block(&mut self.func, "slot_ne_0");
        let bb_eqzero = self.ctx.append_basic_block(&mut self.func, "slot_eq_0");
        let is_slot_nonzero = self.builder.build_icmp(LLVMIntPredicate::LLVMIntNE, slot, u32_0, "is_slot_nonzero");
        self.builder.build_cond_br(is_slot_nonzero, bb_nonzero, bb_eqzero);
        self.builder.position_at_end(bb_nonzero);
        self.builder.build_ret(f32_0);
        self.builder.position_at_end(bb_eqzero);
    }
    /// Casts the value to a u64, or returns 0f32 from the function
    /// if the value doesn't fit in a u64.
    fn checked_fp_to_u64(&mut self, fp: LLVMValueRef, u64_name: &str) -> LLVMValueRef {
        let f32_0 = self.ctx.cons(0f32);
        let f32_2pow64 = self.ctx.cons(18446744073709551616f32);
        let bb_out_of_range = self.ctx.append_basic_block(&mut self.func, "fp_to_u64_fail");
        let bb_gte_0 = self.ctx.append_basic_block(&mut self.func, "fp_to_u64_gte_0");
        let bb_good_cast = self.ctx.append_basic_block(&mut self.func, "fp_to_u64_success");
        // Guard fp < 0
        let is_lt_0 = self.builder.build_fcmp(LLVMRealPredicate::LLVMRealULT,
            fp, f32_0, "is_lt_0");
        self.builder.build_cond_br(is_lt_0, bb_out_of_range, bb_gte_0);
        // Guard fp >= 2^64
        self.builder.position_at_end(bb_gte_0);
        let is_gte_2pow64 = self.builder.build_fcmp(LLVMRealPredicate::LLVMRealUGE,
            fp, f32_2pow64, "is_gte_2pow64");
        self.builder.build_cond_br(is_gte_2pow64, bb_out_of_range, bb_good_cast);
        // Impl the out_of_range code path
        self.builder.position_at_end(bb_out_of_range);
        self.builder.build_ret(f32_0);
        // Perform the cast
        self.builder.position_at_end(bb_good_cast);
        self.builder.build_fp_to_ui(fp,
            u64::get_type_in_context(self.ctx), u64_name)
    }
    /// Subtracts `neg` from `pos`, but returns 0f32 from the function
    /// if the value would underflow.
    fn checked_sub(&mut self, pos: LLVMValueRef, neg: LLVMValueRef, out_name: &str) -> LLVMValueRef {
        let f32_0 = self.ctx.cons(0f32);
        let bb_underflow = self.ctx.append_basic_block(&mut self.func, "checked_sub_undeflow");
        let bb_normal = self.ctx.append_basic_block(&mut self.func, "checked_sub_success");
        let is_sub_neg = self.builder.build_icmp(LLVMIntPredicate::LLVMIntUGT, neg, pos, "is_sub_neg");
        self.builder.build_cond_br(is_sub_neg, bb_underflow, bb_normal);
        // Impl the underflow code path
        self.builder.position_at_end(bb_underflow);
        self.builder.build_ret(f32_0);
        // Perform the subtraction
        self.builder.position_at_end(bb_normal);
        self.builder.build_sub(pos, neg, out_name)

    }
    /// Call the `in_getter` callback with the provided time/slot.
    fn read_input(&mut self, time: LLVMValueRef, slot: u32) -> LLVMValueRef {
        let (in_getter_fn, in_getter_arg) = self.load_getters();
        self.builder.build_call(Function::from_value_ref(in_getter_fn),
            vec![time, self.ctx.cons(slot), in_getter_arg], &format!("input_slot{}", slot))
    }
    /// Read the inputs to slot 0 and slot 1 at the given time.
    fn read_inputs(&mut self, time: LLVMValueRef) -> (LLVMValueRef, LLVMValueRef) {
        (self.read_input(time, 0), self.read_input(time, 1))
    }
    /// Unpack the callback function and its argument.
    fn load_getters(&mut self) -> (LLVMValueRef, LLVMValueRef) {
        let in_getter = self.in_getter();
        let in_getter_struct = self.builder.build_load(in_getter, "in_getter_struct");
        let in_getter_fn = self.builder.build_extract_value(in_getter_struct, 0, "in_getter_fn");
        let in_getter_arg = self.builder.build_extract_value(in_getter_struct, 1, "in_getter_arg");
        (in_getter_fn, in_getter_arg)
    }
}
