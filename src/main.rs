extern crate num;
use self::num::complex::Complex32;

mod effects;

mod effect_node;
mod effect_send;
mod effect_tree;
mod effect_tree_renderer;
mod partial;
mod partial_renderer;

use std::mem;
use effect_send::EffectSend;
use effect_tree::EffectTree;
use effect_tree_renderer::EffectTreeRenderer;
use partial::Partial;
use partial_renderer::PartialRenderer;

fn main() {
    print!("Size of effect_node: {}\n", mem::size_of::<effect_node::EffectNode>());
    print!("Size of effect_send: {}\n", mem::size_of::<effect_send::EffectSend>());
    print!("Size of Option<&Effect>: {}\n", mem::size_of::<Option<&effects::effect::Effect>>());
    print!("Size of Option<&u32>: {}\n", mem::size_of::<Option<&u32>>());
    print!("Size of Option<&EffectNode>: {}\n", mem::size_of::<Option<&effect_node::EffectNode>>());
    print!("Size of &u32: {}\n", mem::size_of::<&u32>());
    print!("Size of PartialStream: {}\n", mem::size_of::<effect_tree_renderer::PartialStream>());
    //print!("Size of Ptr<'a, Effect>: {}\n", mem::size_of::<ptr::Ptr<effects::effect::Effect>>());

    // Test it out: try to render a 1000 rad/sec sine wave.
    let tree = EffectTree::new();
    let mut tree_rend = EffectTreeRenderer::new(&tree);
    let mut partial_rend = PartialRenderer::new(44100);
    tree_rend.feed(EffectSend::new(tree.root().node(), 0), &Partial::new(0, Complex32::new(0.0f32, -1.0f32), 1000.0f32, 0));

    while let Some(partial) = tree_rend.step() {
        print!("New partial: {:?}\n", partial);
        partial_rend.feed(&partial);
    }
    for _ in (0..100) {
        print!("Output: {}\n", partial_rend.step());
    }
    print!("partial_rend: {:?}\n", partial_rend);

    // Remove the partial by feeding its opposite

    tree_rend.feed(EffectSend::new(tree.root().node(), 0), &Partial::new(0, Complex32::new(0.0f32, 1.0f32), 1000.000000001f32, 0));

    while let Some(partial) = tree_rend.step() {
        partial_rend.feed(&partial);
    }
    print!("partial_rend: {:?}\n", partial_rend);
}
