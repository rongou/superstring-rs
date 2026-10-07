//! Shortest common superstrings over arbitrary byte strings.
//!
//! [`preprocess`] removes redundant inputs. [`exact`] finds a shortest common
//! superstring for small instances using exponential-time subset dynamic
//! programming. Both functions make deterministic choices independent of input
//! order. Lengths count bytes, and inputs need not be valid UTF-8.
//!
//! The polynomial-time 2-approximation from OpenAI math family 128 is a planned
//! addition; it is not implemented by this crate yet.
//!
//! ```
//! use superstring::exact;
//!
//! let inputs = [b"abc".as_slice(), b"bcd".as_slice(), b"bc".as_slice()];
//! let output = exact(&inputs)?;
//! assert_eq!(output, b"abcd");
//! # Ok::<(), superstring::ExactError>(())
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod exact;
mod preprocess;

pub use exact::{ExactError, MAX_EXACT_STRINGS, exact};
pub use preprocess::preprocess;
