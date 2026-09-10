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
/// — parse, import, validate and mesh each moved inside their own run-to-run
/// spread over five release runs a side, and peak resident memory went from a
/// median 51 400 kB to 51 552 kB with the whole second range, 51 440–51 596,
/// inside the first, 51 240–51 624. Parse touches no key at all and moved by as
/// much as the rest, which is what says the numbers are the machine and not the
/// change.
///
/// What the zero-sized version bought instead was a release build in which
/// `slot_in` waved every foreign key through, silently and against the doc
/// comment below, which said then that it panics. Two models, each holding one block as its
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
        /// against a different model is refused —
        /// [`Error::KeyFromAnotherModel`](crate::Error::KeyFromAnotherModel) —
        /// instead of resolving to whatever now sits in that slot.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $key {
            slot: $slot,
            model: ModelId,
        }

        impl $key {
            pub(crate) fn new(slot: $slot, model: ModelId) -> Self {
                Self { slot, model }
            }

            /// The slot this key names, for a key that came from a caller.
            ///
            /// # Errors
            ///
            /// [`Error::KeyFromAnotherModel`](crate::Error::KeyFromAnotherModel)
            /// if the key was made by another model. Which model a caller's key
            /// came from is data this crate did not choose, so it is bad caller
            /// input and not a broken invariant (decisions.md §4.1, §86).
            pub(crate) fn try_slot_in(self, model: ModelId) -> $crate::Result<$slot> {
                if self.model == model {
                    Ok(self.slot)
                } else {
                    Err($crate::Error::KeyFromAnotherModel { entity: $what })
                }
            }

            /// The slot this key names, for a key this crate derived itself.
            ///
            /// # Panics
            ///
            /// If the key was made by another model. Every caller of this got
            /// its key from a view of `model` moments earlier, so a mismatch
            /// here is a bug in this crate and not in anybody's input. A key a
            /// caller handed over goes through
            #[doc = concat!("[`", stringify!($key), "::try_slot_in`] instead.")]
            ///
            /// Allowed to be dead because this macro writes seven of these and
            /// only some kinds of entity are ever reached by key from inside
            /// the crate. A wire, a shell and a coedge are reached through the
            /// face or solid that owns them, so nothing here resolves one.
            #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A key carries its model's number, and carries it in this build too.
    ///
    /// The two tests that measure what a foreign key *does* live in
    /// `tests/what_only_a_release_build_does.rs`, and only the release arm runs
    /// them. This one runs everywhere, and it is here because the failure it
    /// guards against is a build in which [`ModelId`] is zero-sized again: the
    /// key loses eight bytes, `slot_in`'s comparison becomes true whichever
    /// models the two ids came from, and nothing in the ordinary suite notices
    /// (decisions.md §84).
    ///
    /// **The second line is the guard; the first only draws the shape.** A
    /// zero-sized field has alignment one, so it can never add padding — with
    /// the id back to `struct ModelId;` the first line reads 8 == 8 + 0 and
    /// passes just as happily as it reads 16 == 8 + 8 today.
    #[test]
    fn a_key_is_wide_enough_to_hold_a_model_number() {
        assert_eq!(
            size_of::<SolidKey>(),
            size_of::<SolidSlot>() + size_of::<ModelId>(),
            "a key is its slot and its model's number, and neither is free"
        );
        assert_ne!(
            size_of::<ModelId>(),
            0,
            "a zero-sized id compares equal to every other id"
        );
    }
}
