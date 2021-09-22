mod frame;

pub use frame::*;

pub fn to_arrow(frames: Vec<Frame>, ref_id: String) -> Vec<Vec<u8>> {
    frames
        .into_iter()
        .map(|x| x.to_arrow(ref_id.clone()))
        .collect()
}
