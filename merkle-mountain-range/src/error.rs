pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    AncestorRootNotPredecessor,
    GetRootOnEmpty,
    InconsistentStore,
    StoreError(crate::string::String),
    /// proof items is not enough to build a tree
    CorruptedProof,
    /// The leaves is an empty list, or beyond the mmr range
    GenProofForInvalidLeaves,
    /// The nodes are an empty list, or beyond the mmr range
    GenProofForInvalidNodes,

    /// The two nodes couldn't merge into one.
    MergeError(crate::string::String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        use Error::*;
        match self {
            AncestorRootNotPredecessor => write!(f, "Ancestor mmr size exceeds current mmr size")?,
            GetRootOnEmpty => write!(f, "Get root on an empty MMR")?,
            InconsistentStore => write!(f, "Inconsistent store")?,
            StoreError(msg) => write!(f, "Store error {}", msg)?,
            CorruptedProof => write!(f, "Corrupted proof")?,
            GenProofForInvalidLeaves => write!(f, "Generate proof for invalid leaves")?,
            GenProofForInvalidNodes => write!(f, "Generate proof for invalid nodes")?,
            MergeError(msg) => write!(f, "Merge error {}", msg)?,
        }
        Ok(())
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        impl ::std::error::Error for Error {}
    }
}
