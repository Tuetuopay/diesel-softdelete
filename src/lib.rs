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

pub mod prelude {
    pub use crate::soft_delete;
}

#[cfg(test)]
mod tests;

/// A SQL database table that makes use of Soft Delete
pub trait SoftDelete: Sized {
    /// The type returned by `deleted_col`
    type Deleted: SelectableExpression<Self> + NonAggregate + Expression<SqlType = Bool>;

    fn deleted_col(&self) -> Self::Deleted;
}
