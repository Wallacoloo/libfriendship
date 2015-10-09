extern crate num;
use self::num::complex::Complex32;

mod effect;
mod effect_node;
mod effect_send;
mod effect_tree;
mod effect_tree_renderer;
mod multi_channel_partial_renderer;
mod partial;
mod partial_renderer;

use std::mem;
use effect_send::EffectSend;
use effect_tree::EffectTree;
use effect_tree_renderer::EffectTreeRenderer;
use multi_channel_partial_renderer::MultiChannelPartialRenderer;
use partial::Partial;
use partial_renderer::PartialRenderer;

fn main() {
    print!("Size of effect_node: {}\n", mem::size_of::<effect_node::EffectNode>());
    print!("Size of effect_send: {}\n", mem::size_of::<effect_send::EffectSend>());
    print!("Size of Option<&Effect>: {}\n", mem::size_of::<Option<&effect::Effect>>());
    print!("Size of Option<&u32>: {}\n", mem::size_of::<Option<&u32>>());
    print!("Size of Option<&EffectNode>: {}\n", mem::size_of::<Option<&effect_node::EffectNode>>());
    print!("Size of &u32: {}\n", mem::size_of::<&u32>());
    print!("Size of PartialStream: {}\n", mem::size_of::<effect_tree_renderer::PartialStream>());
    print!("Size of PartialRenderer: {}\n", mem::size_of::<PartialRenderer>());
    //print!("Size of Ptr<'a, Effect>: {}\n", mem::size_of::<ptr::Ptr<effect::Effect>>());

    // Test it out: try to render a 1000 rad/sec sine wave.
    let tree = EffectTree::new(2);
    let mut tree_rend = EffectTreeRenderer::new(&tree);
    let mut partial_rend = MultiChannelPartialRenderer::new(44100, tree.num_roots());

    tree_rend.feed(EffectSend::new(tree.root(0).clone(), 0), &Partial::new(0, Complex32::new(0.0f32, -1.0f32), 1000.0f32));
    tree_rend.feed(EffectSend::new(tree.root(1).clone(), 0), &Partial::new(0, Complex32::new(0.0f32, -1.0f32), 1001.0f32));

    while let Some((ch, partial)) = tree_rend.step() {
        print!("New partial: {:?}\n", partial);
        partial_rend.feed(ch, partial);
    }
    for _ in (0..100) {
        let out_l = partial_rend.step(0);
        let out_r = partial_rend.step(1);
        print!("Output: {} {}\n", out_l, out_r);
    }
    //print!("partial_rend: {:?}\n", partial_rend);

    // Remove the partial by feeding its opposite

    tree_rend.feed(EffectSend::new(tree.root(0).clone(), 0), &Partial::new(0, Complex32::new(0.0f32, 1.0f32), 1000.000000001f32));
    tree_rend.feed(EffectSend::new(tree.root(1).clone(), 0), &Partial::new(0, Complex32::new(0.0f32, 1.0f32), 1001.000000001f32));

    print!("Canceling existing partials\n");
    while let Some((ch, partial)) = tree_rend.step() {
        partial_rend.feed(ch, partial);
    }
    for _ in (0..4) {
        let out_l = partial_rend.step(0);
        let out_r = partial_rend.step(1);
        print!("Output: {} {}\n", out_l, out_r);
    }
    //print!("partial_rend: {:?}\n", partial_rend);
}
