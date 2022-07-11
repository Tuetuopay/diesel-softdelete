/*!
 * # Diesel softdelete
 *
 * Utilities and traits to handle model soft delete in diesel.
 */

#[cfg(test)]
#[macro_use]
extern crate diesel;

use diesel::{expression::NonAggregate, sql_types::Bool, Expression, SelectableExpression};

mod macros;
pub mod methods;
pub mod query_dsl;
mod query_source;

pub mod prelude {
    pub use crate::soft_delete;
    pub use crate::{methods::*, query_dsl::*};
}

#[cfg(test)]
mod tests;

/// A SQL database table that makes use of Soft Delete
pub trait SoftDelete: Sized {
    /// The type returned by `deleted_col`
    type Deleted: SelectableExpression<Self> + NonAggregate + Expression<SqlType = Bool>;

    fn deleted_col(&self) -> Self::Deleted;
}

impl<F, S, D, W, O, L, Of, G> SoftDelete
    for diesel::query_builder::SelectStatement<F, S, D, W, O, L, Of, G>
where
    F: SoftDelete + diesel::associations::HasTable<Table = F>,
    F::Deleted: SelectableExpression<Self>,
{
    type Deleted = F::Deleted;

    fn deleted_col(&self) -> Self::Deleted {
        F::deleted_col(&F::table())
    }
}
