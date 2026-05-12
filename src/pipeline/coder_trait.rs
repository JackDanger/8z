//! The `Coder` trait that every codec implements.

use crate::container::MethodId;
use crate::error::EightZResult;

/// A coder converts a packed byte slice into an unpacked byte slice, or vice versa.
///
/// 7z folders are pipelines of coders; the `Coder` trait abstracts each stage.
/// Most coders are 1-in/1-out; complex coders (BCJ2) are wired up via a multi-stream
/// folder with bonds.
pub trait Coder: Send + Sync {
    /// Decode `packed` → unpacked bytes. `unpacked_size` is the spec-declared
    /// output size (for sanity-checking and pre-allocation).
    fn decode(&self, packed: &[u8], unpacked_size: u64) -> EightZResult<Vec<u8>>;

    /// Encode `unpacked` → packed bytes. The packed size is whatever the coder
    /// produces; the caller records it as the folder's pack_size.
    fn encode(&self, unpacked: &[u8]) -> EightZResult<Vec<u8>>;

    /// The 7z method ID this coder advertises (returned to the container writer
    /// when building the `Coder` metadata record).
    fn method_id(&self) -> MethodId;

    /// Codec-specific properties byte blob (e.g. LZMA's 5-byte props). Empty for Copy.
    fn properties(&self) -> Vec<u8> {
        Vec::new()
    }
}
