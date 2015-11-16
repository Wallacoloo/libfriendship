extern crate num;
use self::num::complex::Complex32;

mod automation;
mod partial;
mod render;
mod tree;

use std::mem;
use partial::Partial;
use render::reference::nchannel_partial_renderer::NChannelPartialRenderer;
use render::reference::partial_renderer::PartialRenderer;

fn main() {
    print!("init");
    //print!("Size of PartialRenderer: {}\n", mem::size_of::<PartialRenderer>());

    // Test it out: try to render a 1000 rad/sec sine wave.
    /*let tree = EffectTree::new(2);
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
    }*/
    //print!("partial_rend: {:?}\n", partial_rend);
}
