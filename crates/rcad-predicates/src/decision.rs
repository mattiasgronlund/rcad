use serde::{Deserialize, Serialize};

/// What a predicate concluded about the sign of its value.
///
/// There is no bare sign type in this crate on purpose. Every call site has to
/// say what it does when the cascade could not tell, and the only way to force
/// that is to make "could not tell" one of the answers (decisions.md §2.1).
///
/// At the predicate level `Undecidable` is a result, not a failure. At the
/// operation level it *is* a failure; the facade turns it into an error.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Decision {
    /// The value is below zero.
    Neg,
    /// The value is exactly zero.
    Zero,
    /// The value is above zero.
    Pos,
    /// The cascade could not tell which of the other three it is.
    Undecidable {
        /// How wide the interval that straddled zero was.
        enclosure: f64,
    },
}

impl Decision {
    /// Whether this answer is one the cascade proved.
    ///
    /// Only a certified answer may be used without recording provenance
    /// (decisions.md §5).
    pub fn is_certified(self) -> bool {
        match self {
            Decision::Neg | Decision::Zero | Decision::Pos => true,
            Decision::Undecidable { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Decision;

    #[test]
    fn only_undecidable_is_uncertified() {
        assert!(Decision::Neg.is_certified());
        assert!(Decision::Zero.is_certified());
        assert!(Decision::Pos.is_certified());
        assert!(!Decision::Undecidable { enclosure: 1e-30 }.is_certified());
    }

    #[test]
    fn a_decision_survives_being_written_out_and_read_back() {
        for value in [
            Decision::Neg,
            Decision::Zero,
            Decision::Pos,
            Decision::Undecidable { enclosure: 0.25 },
        ] {
            let bytes = postcard::to_allocvec(&value).unwrap();
            assert_eq!(postcard::from_bytes::<Decision>(&bytes).unwrap(), value);
        }
    }
}
