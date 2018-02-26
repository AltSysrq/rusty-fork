//-
// Copyright 2018 Jason Lingle
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// Produce a hashable identifier unique to the particular macro invocation
/// which is stable across processes of the same executable.
///
/// This is usually the best thing to pass for the `fork_id` argument of
/// [`fork`](fn.fork.html).
#[macro_export]
macro_rules! fork_test_id { () => { {
    struct _ForkTestId;
    $crate::ForkTestId::of(::std::any::TypeId::of::<_ForkTestId>())
} } }

/// The type of the value produced by
/// [`fork_test_id!`](macro.fork_test_id.html).
#[derive(Clone, Hash, PartialEq, Debug)]
pub struct ForkTestId(::std::any::TypeId);
impl ForkTestId {
    #[allow(missing_docs)]
    #[doc(hidden)]
    pub fn of(id: ::std::any::TypeId) -> Self {
        ForkTestId(id)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn ids_are_actually_distinct() {
        assert_ne!(fork_test_id!(), fork_test_id!());
    }
}
