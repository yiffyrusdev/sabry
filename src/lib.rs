#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg_attr(docsrs, doc(cfg(feature = "build")))]
#[cfg(feature = "build")]
pub use sabry_build::buildmagic::buildy;
#[cfg_attr(docsrs, doc(cfg(feature = "procmacro")))]
#[cfg(feature = "procmacro")]
pub use sabry_procmacro::*;


// reexports
#[cfg_attr(docsrs, doc(cfg(all(feature = "build", feature = "internals"))))]
#[cfg(all(feature = "build", feature = "internals"))]
pub use sabry_build::*;

#[cfg_attr(docsrs, doc(cfg(all(feature = "procmacro", feature = "internals"))))]
#[cfg(all(feature = "procmacro", feature = "internals"))]
pub use sabry_intrnl::*;
