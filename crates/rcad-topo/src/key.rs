slotmap::new_key_type! {
    pub(crate) struct VertexSlot;
    pub(crate) struct EdgeSlot;
    pub(crate) struct CoedgeSlot;
    pub(crate) struct WireSlot;
    pub(crate) struct FaceSlot;
    pub(crate) struct ShellSlot;
    pub(crate) struct SolidSlot;
}

/// Which model a key belongs to.
///
/// `slotmap`'s generation counter catches a key whose slot was freed and
/// refilled *in the same map*. It cannot see a key from a different model
/// reaching a live slot in this one, and that resolves to the wrong entity with
/// no crash (decisions.md §1.3).
///
/// In release builds this is a zero-sized type, so keys pay nothing for it and
/// the comparison folds away.
#[cfg(debug_assertions)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct ModelId(u64);

#[cfg(debug_assertions)]
impl ModelId {
    pub(crate) fn next() -> Self {
        use core::sync::atomic::{AtomicU64, Ordering};

        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

/// The release-build stand-in for [`ModelId`]: every model has the same one, so
/// the check costs nothing and proves nothing.
#[cfg(not(debug_assertions))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct ModelId;

#[cfg(not(debug_assertions))]
impl ModelId {
    pub(crate) fn next() -> Self {
        Self
    }
}

macro_rules! define_key {
    ($key:ident, $slot:ident, $what:literal) => {
        #[doc = concat!("A handle on one ", $what, " in a [`Model`](crate::Model).")]
        ///
        /// Every entity has its own key type, so handing a face key to
        /// something that wants an edge is a compile error rather than a
        /// silent lookup of the wrong thing.
        ///
        /// In debug builds a key also remembers which model made it. Using it
        /// against a different model panics instead of resolving to whatever
        /// now sits in that slot.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $key {
            slot: $slot,
            model: ModelId,
        }

        impl $key {
            pub(crate) fn new(slot: $slot, model: ModelId) -> Self {
                Self { slot, model }
            }

            /// The slot this key names, having checked it came from `model`.
            ///
            /// # Panics
            ///
            /// If the key was made by another model. That is a bug in the
            /// caller and the alternative is silently reading the wrong
            /// entity.
            pub(crate) fn slot_in(self, model: ModelId) -> $slot {
                assert!(
                    self.model == model,
                    concat!("this ", $what, " key was made by a different model")
                );
                self.slot
            }
        }
    };
}

define_key!(VertexKey, VertexSlot, "vertex");
define_key!(EdgeKey, EdgeSlot, "edge");
define_key!(CoedgeKey, CoedgeSlot, "coedge");
define_key!(WireKey, WireSlot, "wire");
define_key!(FaceKey, FaceSlot, "face");
define_key!(ShellKey, ShellSlot, "shell");
define_key!(SolidKey, SolidSlot, "solid");
