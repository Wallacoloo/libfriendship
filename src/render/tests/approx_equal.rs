
/// Should pass if the two audio buffers "sound" similar.
/// i.e. all their components have similar frequency and magnitude.
/// Note: for simplicity, this is currently a pretty strict comparison
pub fn assert_similar_audio(audio1 : &[f32], audio2 : &[f32]) {
    println!("Testing for similar audio data");
    assert_eq!(audio1.len(), audio2.len());
    // get the magnitude of the first source so that we can determine an
    // appropriate error threshold
    let mean_square1 = audio1.iter().fold(0f32, |acc, v| acc + v*v) / (audio1.len() as f32);
    // we care only for the maximum *square* error:
    // 0.00001 = 1 part^2 per 100,000
    let error_thresh = mean_square1 * 0.00001f32;

    let mut all_pass = true;

    for (a1, a2) in audio1.iter().zip(audio2.iter()) {
        let sq_err = (a2-a1)*(a2-a1);
        println!("Expected {}, got {} ({} square error)", a1, a2, sq_err);
        all_pass = all_pass && (sq_err < error_thresh);
        //assert!(sq_err < error_thresh, "Expected {} (apx.), got {} (err_thresh {})", a1, a2, error_thresh);
    }
    assert!(all_pass, "Some audio was not as expected; run with `cargo test -- --nocapture` for more info");
    println!("Audio is similar");
}
