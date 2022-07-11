use super::SoftDelete;
use diesel::{
    associations::HasTable,
    dsl::{not, And},
    helper_types::not as Not,
    query_builder::AsQuery,
    query_dsl::InternalJoinDsl,
    BoolExpressionMethods, Expression, JoinTo,
};

/// Indicates that two tables can be joined without an explicit `ON` clause while respecting
/// soft-delete.
pub trait SoftJoinTo<T>: JoinTo<T> {
    type SoftOnClause;
    fn soft_join_target(rhs: T) -> (<Self as JoinTo<T>>::FromClause, Self::SoftOnClause);
}

impl<Lhs, Rhs> SoftJoinTo<Rhs> for Lhs
where
    Lhs: JoinTo<Rhs>,
    Rhs: SoftDelete,
    <Lhs as JoinTo<Rhs>>::OnClause: Expression + BoolExpressionMethods,
{
    type SoftOnClause = And<Lhs::OnClause, Not<Rhs::Deleted>>;

    fn soft_join_target(rhs: Rhs) -> (Self::FromClause, Self::SoftOnClause) {
        let deleted = Rhs::deleted_col(&rhs);
        let (from_clause, on_clause) = Self::join_target(rhs);
        (from_clause, on_clause.and(not(deleted)))
    }
}

pub trait SoftJoin<Rhs, Kind> {
    type Output: AsQuery;
    fn soft_join(self, rhs: Rhs, kind: Kind) -> Self::Output;
}

impl<Lhs, Rhs, Kind> SoftJoin<Rhs, Kind> for Lhs
where
    Lhs: SoftJoinTo<Rhs>,
    Lhs: InternalJoinDsl<
        <Lhs as JoinTo<Rhs>>::FromClause,
        Kind,
        <Lhs as SoftJoinTo<Rhs>>::SoftOnClause,
    >,
{
    type Output = <Lhs as InternalJoinDsl<Lhs::FromClause, Kind, Lhs::SoftOnClause>>::Output;
    fn soft_join(self, rhs: Rhs, kind: Kind) -> Self::Output {
        let (from, on) = Lhs::soft_join_target(rhs);
        self.join(from, kind, on)
    }
}
