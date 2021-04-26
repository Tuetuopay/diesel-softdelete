/**
 * Implement the `SoftDelete` trait on a Diesel table. By default, assumes the deleted flag name is
 * `deleted`.
 *
 * # Example
 *
 * ```rust,ignore
 * table! {
 *     user (id) {
 *         id -> Integer,
 *         deleted -> Bool,
 *     }
 * }
 * soft_delete!(user);
 * ```
 *
 * or
 *
 * ```rust,ignore
 * table! {
 *     user (id) {
 *         id -> Integer,
 *         is_deleted -> Bool,
 *     }
 * }
 * soft_delete!(user::user => (user::is_deleted));
 * ```
 */
#[macro_export]
macro_rules! soft_delete {
    ($table:path => ($deleted:path)) => {
        impl $crate::SoftDelete for $table {
            type Deleted = $deleted;
            fn deleted_col(&self) -> Self::Deleted { $deleted }
        }
    };
    ($table:ident) => { soft_delete!($table::table => ($table::deleted)); };
}
