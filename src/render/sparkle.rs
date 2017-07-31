//! Sparkle is a JIT-based renderer that uses llvm to compile effects as they
//! are loaded.
//! Other than the JIT aspect, it is mostly a literal reimplementation of
//! the reference renderer.

use std::collections::HashMap;
use std::ffi::CString;
use std::mem;
use std::ops::{Deref, DerefMut};

use jagged_array::Jagged2;
use llvm;
use llvm::{Builder, Context, ContextType, ExecutionEngine, Function, Module};
use llvm_sys;
use llvm_sys::core::{
    LLVMGetUndef,
    LLVMStructCreateNamed,
    LLVMStructSetBody,
};
use llvm_sys::{
    LLVMIntPredicate,
    LLVMRealPredicate,
};
use llvm_sys::prelude::*;
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
struct FnBuilder<'ctx> {
    /// Function being built
    func: Function,
    /// Handle into LLVM API
    ctx: &'ctx Context,
    /// Handle to the LLVM Builder, which creates the asm instructions
    builder: &'ctx mut Builder,
    /// LLVM struct { fn(time, slot, callback_type*)->f32, callback_type* }
    /// Used to pass callbak functions into get_output() to allow effects to access their inputs.
    callback_type: LLVMTypeRef,
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
    fn jit_effect(&mut self, module: &mut Module, effect: &Effect) -> (Function, String) {
        let fname = format!("{}_get_output", effect.id().name());
        println!("jit: {}", fname);
        let llvm_ctx = Context{ ptr: self.llvm_ctx.ptr };
        let func = match self.get_fn(&fname) {
            Some(func) => func,
            None => {
                // Effect hasn't been compiled yet; do so.
                let sample_getter_type = self.sample_getter_type;
                let func = module.add_function(sample_getter_type, &fname);
                let mut builder = llvm_ctx.create_builder();
                let mut fnbuilder = FnBuilder::new(func, &llvm_ctx, &mut builder, &self);
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
                        // Plan: walk the graph depth-first s.t. the inputs to any
                        // node are processed before the node itself.
                        // Then, we can greate a function `node_get_input(in_time, in_slot, userdata:
                        // *const CallbackType) for each node trivially.
                        let build_inp_getter = |active_fnbuilder: &mut FnBuilder,
                            node_hnd: &NodeHandle,
                            input_getters: &HashMap<NodeHandle, Function>,
                            module: &mut Module,
                            me: &mut Self
                        | {
                            active_fnbuilder.build_slotswitch(graph.iter_edges_to(node_hnd).map(|edge| {
                                if edge.from_full().is_toplevel() {
                                    // Reading from the toplevel input
                                    (   edge.to_slot(),
                                        edge.from_slot(),
                                        None
                                    )
                                } else {
                                    let from_data = graph.get_data(&edge.from_full()).unwrap();
                                    (   edge.to_slot(),
                                        edge.from_slot(),
                                        Some((
                                            me.jit_effect(module, &from_data).0,
                                            &input_getters[&edge.from_full()]
                                        ))
                                    )
                                }
                            }).collect());
                        };
                        let mut input_getters: HashMap<NodeHandle, Function> = Default::default();
                        for ref node_hnd in graph.iter_nodes_dep_first() {
                            // Create a switch statement that branches on the requested slot (i.e.
                            // to_slot) and maps to from_slot and the appropriate getter function.
                            let input_get_fname = format!("{}_n{}_get_input", effect.id().name(), node_hnd);
                            let input_get_fn = module.add_function(sample_getter_type, &input_get_fname);
                            let mut input_builder = llvm_ctx.create_builder();
                            let mut input_fnbuilder = FnBuilder::new(input_get_fn, &llvm_ctx, &mut input_builder, &self);
                            build_inp_getter(&mut input_fnbuilder, node_hnd, &input_getters, module, self);
                            input_getters.insert(*node_hnd, input_fnbuilder.func);
                        }
                        // Build the toplevel getter directly into the main function
                        build_inp_getter(&mut fnbuilder, &NodeHandle::toplevel(), &input_getters, module, self)
                    },
                    _ => panic!("Cannot JIT effect: {:?}", effect)
                }
                fnbuilder.func
            }
        };
        // due to a bad API, we created two owners of the llvm_ctx earlier.
        mem::forget(llvm_ctx);

        (func, fname)
    }
    /// Get or create an editable module
    fn take_open_module(&mut self) -> Module {
        // TODO: use entry API
        if let Some(module) = self.open_module.take() {
            module
        } else {
            self.llvm_ctx.module_create_with_name(
                &format!("mod{}", self.llvm_modules.len()))
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
            EffectData::Primitive(_) | EffectData::RouteGraph(_) => {
                // Jit the effect into an open module
                let mut module = self.take_open_module();

                let ret = MyNodeData::LlvmFunc(self.jit_effect(&mut module, effect).1);
                self.open_module = Some(module);
                ret
            }
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
            llvm_ctx, llvm_modules, llvm_engines, open_module, callback_type, sample_getter_type
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


impl<'ctx> FnBuilder<'ctx> {
    fn new(mut func: Function, ctx: &'ctx Context, builder: &'ctx mut Builder, renderer: &SparkleRenderer) -> Self {
        let bb = ctx.append_basic_block(&mut func, "entry_point");
        builder.position_at_end(bb);
        Self{ func, ctx, builder, callback_type: renderer.callback_type }
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
        let in_getter = self.load_getters();
        // Amount to delay input by
        let delay_frames = self.read_input(time, 1, in_getter);
        let delay_frames_u64 = self.checked_fp_to_u64(delay_frames, "delay_frames_u64");
        let delayed_time = self.checked_sub(time, delay_frames_u64, "delayed_time");
        let result = self.read_input(delayed_time, 0, in_getter);
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
    /// use `load_getters()` to generate the input for `in_getter`
    fn read_input(&mut self, time: LLVMValueRef, slot: u32, in_getter: (LLVMValueRef, LLVMValueRef)) -> LLVMValueRef {
        let (in_getter_fn, in_getter_arg) = in_getter;
        self.builder.build_call(Function::from_value_ref(in_getter_fn),
            vec![time, self.ctx.cons(slot), in_getter_arg], &format!("input_slot{}", slot))
    }
    /// Read the inputs to slot 0 and slot 1 at the given time.
    fn read_inputs(&mut self, time: LLVMValueRef) -> (LLVMValueRef, LLVMValueRef) {
        let in_getter = self.load_getters();
        (self.read_input(time, 0, in_getter), self.read_input(time, 1, in_getter))
    }
    /// Unpack the callback function and its argument.
    fn load_getters(&mut self) -> (LLVMValueRef, LLVMValueRef) {
        let in_getter = self.in_getter();
        let in_getter_struct = self.builder.build_load(in_getter, "in_getter_struct");
        let in_getter_fn = self.builder.build_extract_value(in_getter_struct, 0, "in_getter_fn");
        let in_getter_arg = self.builder.build_extract_value(in_getter_struct, 1, "in_getter_arg");
        (in_getter_fn, in_getter_arg)
    }
    /// Branch based on the output slot being queried.
    /// Each case entry is as follows: (slot_to_match, slot_to_query, (node_fn,
    /// get_input_to_node_fn))
    /// 
    /// That is, each case generates code like
    /// ```
    /// if slot == slot_to_match {
    ///     return node_fn(time, slot_to_query, (get_input_to_node_fn, &in_getter))
    /// }
    /// ```
    /// If only the first two arguments are provided, then that branch represents
    /// reading from the toplevel input.
    fn build_slotswitch<'a>(&'a mut self, cases: Vec<(u32, u32, Option<(Function, &'a Function)>)>) {
        let f32_0 = self.ctx.cons(0f32);
        let bb_nomatch = self.ctx.append_basic_block(&mut self.func, "match_slot_none");
        // First, generate the basic blocks for each branch option
        let blocks = cases.iter().map(|&(ref match_slot, ref _source_slot, ref _source_info)| {
            let bb_name = format!("match_slot_{}", match_slot);
            (self.ctx.cons(*match_slot), self.ctx.append_basic_block(&mut self.func, &bb_name))
        }).collect();
        self.build_switch(self.slot(), bb_nomatch, &blocks);

        // populate each branch of the switch statement
        self.builder.position_at_end(bb_nomatch);
        self.builder.build_ret(f32_0);
        for ((_cond, bb), (__cond, source_slot, source_info)) in blocks.into_iter().zip(cases.into_iter()) {
            self.builder.position_at_end(bb);
            let in_getter = self.in_getter();
            let time = self.time();
            match source_info {
                // Reading from a toplevel input
                None => {
                    let (in_getter_fn, in_getter_arg) = self.load_getters();
                    // Need to wrap the pointer to be able to treat it as a function.
                    let pseudo_in_getter = Function{ ptr: in_getter_fn };
                    let result = self.builder.build_call(pseudo_in_getter,
                        vec![time, self.ctx.cons(source_slot), in_getter_arg],
                        "result");
                    self.builder.build_ret(result);
                    //mem::forget(pseudo_in_getter);
                }
                // Reading from another node with its own input getter
                Some((node_fn, new_in_getter)) => {
                    let u32_0 = self.ctx.cons(0u32);
                    let u32_1 = self.ctx.cons(1u32);
                    let wrapped_in_getter = self.builder.build_alloca(
                        self.callback_type, "wrapped_in_getter");
                    let addr_of_in_getter_0 = self.builder.build_gep(
                        wrapped_in_getter, vec![u32_0, u32_0], "addr_of_in_getter_0");
                    self.builder.build_store(new_in_getter.ptr, addr_of_in_getter_0);
                    let addr_of_in_getter_1 = self.builder.build_gep(
                        wrapped_in_getter, vec![u32_0, u32_1], "addr_of_in_getter_1");
                    self.builder.build_store(in_getter, addr_of_in_getter_1);
                    let result = self.builder.build_call(node_fn,
                        vec![time, self.ctx.cons(source_slot), wrapped_in_getter],
                        "result");
                    self.builder.build_ret(result);
                }
            }
        }
    }
    /// Build a switch statement.
    /// ```
    /// switch `value` {
    ///     `cases`[0].0:
    ///         `cases`[0].1
    ///         break
    ///     [...]
    ///     default: `default`
    /// }
    /// ```
    fn build_switch(&self, value: LLVMValueRef, default: LLVMBasicBlockRef,
                        cases: &Vec<(LLVMValueRef, LLVMBasicBlockRef)>) -> LLVMValueRef {
        unsafe {
            let switch = llvm_sys::core::LLVMBuildSwitch(self.builder.ptr, value, default, cases.len() as u32);
            for case in cases {
                llvm_sys::core::LLVMAddCase(switch, case.0, case.1);
            }
            switch
        }
    }
}
