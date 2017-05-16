use std::mem;

/// Transmute a native f32 to a native u32 in such a way that
/// the sign bit is the MSB of the u32.
pub fn pack_f32(value: f32) -> u32 {
    // At the time of writing, rust only supports platforms where float
    // endianness == int endianness. Nearly all platforms with hardware floats
    // are designed this way.
    // Therefore, a transmutation packs the float; the MSB (sign bit) of the
    // float becomes the MSB of the u32, as expected.
    //
    // This can never fail, so the function can be exported as safe.
    unsafe { mem::transmute(value) }
}

/// Transmute a native u32 to a native f32 in such a way that
/// the MSB of the u32 becomes the sign bit of the f32.
/// 
/// Inverse function of `pack_f32`.
pub fn unpack_f32(value: u32) -> f32 {
    unsafe { mem::transmute(value) }
}

/// Macro to create a GC collection from an array.
/// Examples:
/// ```
/// let v: HashMap<u8, u8> = collect_arr![(0, 4), (1, 8)];
/// ```
#[macro_export]
defmac!(collect_arr array => array.iter().cloned().collect());
