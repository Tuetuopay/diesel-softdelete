# diesel-softdelete

Add soft-delete integration to the Diesel query builder.

Soft deletion is a practice where a database entry is not actually deleted, but flagged as
deleted. This often takes the form of a `deleted` boolean flag. However this implies always
filtering on this flag in your queries, which is both cumbersome and error prone: always
filtering on the flag is repetitive, can be forgotten and can lead to incorrect queries in e.g.
joins.

The main additions of this library are to the query builder, with new operations:

- [`soft_find`](methods::SoftFindDsl::soft_find) which is analogous to
  [`find`](diesel::query_dsl::QueryDsl::find), but with the soft-delete filter applied.
- [`soft_inner_join`](query_dsl::SoftJoinDsl::soft_inner_join) /
  [`soft_left_join`](query_dsl::SoftJoinDsl::soft_left_join) which are analogous to
  [`inner_join`](diesel::query_dsl::QueryDsl::inner_join) /
  [`left_join`](diesel::query_dsl::QueryDsl::left_join),
  but with the soft-delete filter applied to the `ON` clause, not the `WHERE` clause.

## Usage

Your model needs to have a `deleted` boolean column. Then, use the [`soft_delete`](soft_delete)
macro to implement the [`SoftDelete`](SoftDelete) trait on the table. And that's it! The
`soft_find` and other functions are ready to be used in place of the regular `find` etc macros
once the prelude is imported.

## Example

```rust
use diesel_softdelete::prelude::*;

table! {
    user (id) {
        id -> Integer,
        name -> Text,
        deleted -> Bool,
    }
}
soft_delete!(user);

conn.batch_execute("
    create table user(
        id integer primary key,
        name text not null,
        deleted bool not null default false
    );
    insert into user(id, name, deleted) values (1, 'Alice', false), (2, 'Bob', true);
")?;

let name = user::table.soft_find(1).select(user::name).first::<String>(&conn).optional()?;
assert_eq!(name, Some("Alice".to_owned()));
let name = user::table.soft_find(2).select(user::name).first::<String>(&conn).optional()?;
assert_eq!(name, None);
```

License: MIT OR Apache-2.0
