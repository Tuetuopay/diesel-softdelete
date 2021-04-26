use diesel::{
    dsl::{not, Filter},
    helper_types::not as Not,
    query_dsl::methods::{FilterDsl, FindDsl},
};

use super::SoftDelete;

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
