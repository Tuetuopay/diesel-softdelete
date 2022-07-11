//! Methods to use on the query builder

use crate::query_source::SoftJoin;
use diesel::query_source::joins::{Inner, LeftOuter};

/// The `soft_left_join` and `soft_inner_join` methods.
pub trait SoftJoinDsl: Sized {
    fn soft_inner_join<Rhs>(self, rhs: Rhs) -> Self::Output
    where
        Self: SoftJoin<Rhs, Inner>,
    {
        self.soft_join(rhs, Inner)
    }

    fn soft_left_join<Rhs>(self, rhs: Rhs) -> Self::Output
    where
        Self: SoftJoin<Rhs, LeftOuter>,
    {
        self.soft_join(rhs, LeftOuter)
    }
}

impl<Lhs> SoftJoinDsl for Lhs where Lhs: Sized {}
