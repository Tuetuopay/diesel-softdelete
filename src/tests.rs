use crate::prelude::*;
use diesel::{connection::SimpleConnection, prelude::*, sqlite::SqliteConnection};

table! {
    user (id) {
        id -> Integer,
        name -> Text,
        deleted -> Bool,
    }
}

#[derive(Identifiable, Queryable, Debug, PartialEq)]
#[table_name = "user"]
struct User {
    id: i32,
    name: String,
    deleted: bool,
}

#[derive(Insertable)]
#[table_name = "user"]
struct NewUser<'a> {
    name: &'a str,
}

soft_delete!(user);

fn conn() -> SqliteConnection {
    let conn = SqliteConnection::establish(":memory:").expect("Failed to open `:memory:` database");
    conn.batch_execute(
        "
        create table user(
            id integer primary key,
            name text not null,
            deleted bool not null default false
        );
    ",
    )
    .expect("Failed to create `user` table");
    conn
}

#[test]
fn test_find_ok() {
    let conn = conn();

    diesel::insert_into(user::table).values(NewUser { name: "Joe" }).execute(&conn).unwrap();
    let joe_id: i32 = user::table.select(user::id).first(&conn).unwrap();
    let joe: User = user::table.find(joe_id).first(&conn).unwrap();

    assert_eq!(joe, User { id: joe_id, name: "Joe".to_owned(), deleted: false });
}

#[test]
fn test_soft_find_ok() {
    let conn = conn();

    diesel::insert_into(user::table).values(NewUser { name: "Joe" }).execute(&conn).unwrap();
    let joe_id: i32 = user::table.select(user::id).first(&conn).unwrap();

    let joe: Option<User> = user::table.soft_find(joe_id).first(&conn).optional().unwrap();
    assert_eq!(joe, Some(User { id: joe_id, name: "Joe".to_owned(), deleted: false }));

    diesel::update(user::table).set(user::deleted.eq(true)).execute(&conn).unwrap();

    let joe: Option<User> = user::table.soft_find(joe_id).first(&conn).optional().unwrap();
    assert_eq!(joe, None);
}
