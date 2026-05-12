//! Test utility macros for the 7zippy test suite.
//!
//! [`assert_slices_eq!`] is the primary export: it behaves like `assert_eq!`
//! on byte slices but on mismatch prints up to four differing byte indices
//! with surrounding hex context, making it much easier to diagnose off-by-one
//! or partial-write bugs in codec output.

/// Assert that two byte slices are equal.
///
/// On mismatch the macro:
/// 1. Reports lengths if they differ.
/// 2. Finds the first differing byte index.
/// 3. Prints up to four `[index]: left=0xXX right=0xXX` lines.
/// 4. Shows ±16 bytes of hex context around the first difference.
/// 5. Panics.
///
/// # Examples
///
/// ```rust,ignore
/// use crate::assert_slices_eq;
/// assert_slices_eq!(&decoded, &expected);
/// assert_slices_eq!(&decoded, &expected, "round-trip failed");
/// ```
#[macro_export]
macro_rules! assert_slices_eq {
    ($a:expr, $b:expr) => {{
        let a: &[u8] = &$a[..];
        let b: &[u8] = &$b[..];
        if a != b {
            if a.len() != b.len() {
                panic!(
                    "assertion failed (left == right)\n  left  len: {}\n  right len: {}",
                    a.len(),
                    b.len(),
                );
            }
            let diffs: ::std::vec::Vec<usize> = a
                .iter()
                .zip(b.iter())
                .enumerate()
                .filter(|(_, (x, y))| x != y)
                .map(|(i, _)| i)
                .take(4)
                .collect();
            let first = diffs[0];
            let ctx_start = first.saturating_sub(16);
            let ctx_end = (first + 16).min(a.len());
            let diff_lines: ::std::string::String = diffs
                .iter()
                .map(|&i| format!("  [{i}]: left=0x{:02X}  right=0x{:02X}", a[i], b[i]))
                .collect::<::std::vec::Vec<_>>()
                .join("\n");
            panic!(
                "assertion failed (left == right)\nFirst {} differing byte(s):\n{diff_lines}\nContext (bytes {ctx_start}..{ctx_end}):\n  left:  {:02X?}\n  right: {:02X?}",
                diffs.len(),
                &a[ctx_start..ctx_end],
                &b[ctx_start..ctx_end],
            );
        }
    }};
    ($a:expr, $b:expr, $msg:expr) => {{
        let a: &[u8] = &$a[..];
        let b: &[u8] = &$b[..];
        if a != b {
            if a.len() != b.len() {
                panic!(
                    "assertion failed (left == right): {}\n  left  len: {}\n  right len: {}",
                    $msg,
                    a.len(),
                    b.len(),
                );
            }
            let diffs: ::std::vec::Vec<usize> = a
                .iter()
                .zip(b.iter())
                .enumerate()
                .filter(|(_, (x, y))| x != y)
                .map(|(i, _)| i)
                .take(4)
                .collect();
            let first = diffs[0];
            let ctx_start = first.saturating_sub(16);
            let ctx_end = (first + 16).min(a.len());
            let diff_lines: ::std::string::String = diffs
                .iter()
                .map(|&i| format!("  [{i}]: left=0x{:02X}  right=0x{:02X}", a[i], b[i]))
                .collect::<::std::vec::Vec<_>>()
                .join("\n");
            panic!(
                "assertion failed (left == right): {}\nFirst {} differing byte(s):\n{diff_lines}\nContext (bytes {ctx_start}..{ctx_end}):\n  left:  {:02X?}\n  right: {:02X?}",
                $msg,
                diffs.len(),
                &a[ctx_start..ctx_end],
                &b[ctx_start..ctx_end],
            );
        }
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn equal_slices_pass() {
        assert_slices_eq!([1u8, 2, 3], [1u8, 2, 3]);
    }

    #[test]
    #[should_panic(expected = "left  len")]
    fn different_lengths_panic() {
        assert_slices_eq!([1u8, 2], [1u8, 2, 3]);
    }

    #[test]
    #[should_panic(expected = "left=0x01  right=0x02")]
    fn different_bytes_panic() {
        assert_slices_eq!([1u8, 3], [2u8, 3]);
    }

    #[test]
    #[should_panic(expected = "custom message")]
    fn message_variant_includes_message() {
        assert_slices_eq!([1u8], [2u8], "custom message");
    }

    #[test]
    fn empty_slices_pass() {
        let a: &[u8] = &[];
        let b: &[u8] = &[];
        assert_slices_eq!(a, b);
    }
}
