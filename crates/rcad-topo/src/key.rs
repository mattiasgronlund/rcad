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
/// # It is a real number in every build, and it used to not be
///
/// This was `struct ModelId;` — a zero-sized type, one value, every comparison
/// true — whenever `debug_assertions` were off, on the grounds that keys should
/// pay nothing for a check nobody had priced. The price is now measured and it
/// is not there: nothing a [`Model`](crate::Model) stores holds a key at all.
/// Every arena payload points at its neighbours by `*Slot`, so growing the id
/// grows the model by zero bytes; only keys in flight and the transient maps a
/// mesh or a fuse builds get wider. On the biggest model in the STEP corpus
/// that reads — 20 312 entities out of `PinHeader_2x39_P2.54mm_Horizontal.step`
/// — parse, import, validate and mesh moved inside their own run-to-run spread
/// and peak resident memory moved 0.1%.
///
/// What the zero-sized version bought instead was a release build in which
/// `slot_in` waved every foreign key through, silently and against the doc
/// comment below saying it panics. Two models, each holding one block as its
/// first solid: `theirs.solid(mine)` measured 1000 mm³ for a key naming a 1 mm³
/// cube, and `theirs.fuse(&[mine], &[block], Union, …)` answered **`Ok`**
/// naming no solids at all — the block fused against itself, the caller's cube
/// never looked at (decisions.md §82.10).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct ModelId(u64);

impl ModelId {
    pub(crate) fn next() -> Self {
        use core::sync::atomic::{AtomicU64, Ordering};

        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
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
        /// A key also remembers which model made it, in every build. Using it
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
