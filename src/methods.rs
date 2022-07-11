//! Expression methods implemented on the table.

use diesel::{
    dsl::{not, Filter},
    helper_types::not as Not,
    query_dsl::methods::{FilterDsl, FindDsl},
};

use super::SoftDelete;

pub trait SoftDeleteDsl: SoftDelete {
    /// The type returned by `.soft_deleted`.
    type Output;
    fn soft_deleted(self) -> Self::Output;
}

impl<T> SoftDeleteDsl for T
where
    T: SoftDelete + FilterDsl<Not<Self::Deleted>>,
{
    type Output = Filter<Self, Not<Self::Deleted>>;
    fn soft_deleted(self) -> Self::Output {
        let deleted = self.deleted_col();
        self.filter(not(deleted))
    }
}

/// The `soft_find` method
pub trait SoftFindDsl<PK>: SoftDelete {
    /// The type returned by `.soft_find`.
    type Output;
    fn soft_find(self, id: PK) -> Self::Output;
}

impl<T, PK> SoftFindDsl<PK> for T
where
    T: SoftDelete + FindDsl<PK>,
    <T as FindDsl<PK>>::Output: FilterDsl<Not<Self::Deleted>>,
{
    type Output = Filter<<T as FindDsl<PK>>::Output, Not<T::Deleted>>;

    fn soft_find(self, id: PK) -> Self::Output {
        let deleted = self.deleted_col();
        self.find(id).filter(not(deleted))
    }
}

/// The `soft_filter` method.
///
/// This trait is used to automatically add soft-delete filtering on regular `filter` in queries.
/// It only needs to be put once per query.
///
/// Be careful with it, as it is often incorrect to use it on left-joined tables. For such cases,
/// use the [`soft_left_join`](crate::query_dsl::SoftJoinDsl::soft_left_join) method to join the
/// table and don't filter on the deleted status.
pub trait SoftFilterDsl<Predicate>: SoftDelete {
    /// The type returned by `.soft_filter`.
    type Output;
    fn soft_filter(self, predicate: Predicate) -> Self::Output;
}

impl<T, Predicate> SoftFilterDsl<Predicate> for T
where
    T: SoftDelete + FilterDsl<Predicate>,
    <T as FilterDsl<Predicate>>::Output: FilterDsl<Not<Self::Deleted>>,
{
    type Output = Filter<<T as FilterDsl<Predicate>>::Output, Not<T::Deleted>>;

    fn soft_filter(self, predicate: Predicate) -> Self::Output {
        let deleted = self.deleted_col();
        self.filter(predicate).filter(not(deleted))
    }
}
