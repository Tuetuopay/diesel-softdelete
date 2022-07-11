//! Expression methods implemented on the table.

use diesel::{
    dsl::{not, Filter, Find},
    helper_types::not as Not,
    query_dsl::methods::{FilterDsl, FindDsl},
};

use super::SoftDelete;

pub trait SoftDeleteDsl: SoftDelete {
    /// The type returned by `.soft_deleted`.
    type Output;
    fn soft_deleted(self) -> Self::Output;
}

pub type SoftDeleted<Source> = Filter<Source, Not<<Source as SoftDelete>::Deleted>>;

impl<T> SoftDeleteDsl for T
where
    Self: SoftDelete + FilterDsl<Not<Self::Deleted>>,
{
    type Output = SoftDeleted<Self>;
    fn soft_deleted(self) -> Self::Output {
        let deleted = self.deleted_col();
        self.filter(not(deleted))
    }
}

pub type SoftFind<Source, PK> = Filter<Find<Source, PK>, Not<<Source as SoftDelete>::Deleted>>;

/// The `soft_find` method
pub trait SoftFindDsl<PK>: SoftDelete {
    /// The type returned by `.soft_find`.
    type Output;
    fn soft_find(self, id: PK) -> Self::Output;
}

impl<T, PK> SoftFindDsl<PK> for T
where
    Self: SoftDelete + FindDsl<PK>,
    Find<Self, PK>: FilterDsl<Not<Self::Deleted>>,
{
    type Output = SoftFind<Self, PK>;

    fn soft_find(self, id: PK) -> Self::Output {
        let deleted = self.deleted_col();
        self.find(id).filter(not(deleted))
    }
}

pub type SoftFilter<Source, Predicate> =
    Filter<Filter<Source, Predicate>, Not<<Source as SoftDelete>::Deleted>>;

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
    Self: SoftDelete + FilterDsl<Predicate>,
    Filter<Self, Predicate>: FilterDsl<Not<Self::Deleted>>,
{
    type Output = SoftFilter<Self, Predicate>;

    fn soft_filter(self, predicate: Predicate) -> Self::Output {
        let deleted = self.deleted_col();
        self.filter(predicate).filter(not(deleted))
    }
}
