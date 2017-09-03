/// Macro to create a GC collection from an array.
/// Examples:
/// ```
/// let v: HashMap<u8, u8> = collect_arr![(0, 4), (1, 8)];
/// ```
#[macro_export]
macro_rules! collect_arr {
    ($val:expr) => {
        $val.iter().cloned().collect()
    }
}
