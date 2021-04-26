use crate::prelude::*;
use diesel::{connection::SimpleConnection, prelude::*, sqlite::SqliteConnection};

table! {
    user (id) {
        id -> Integer,
        name -> Text,
        deleted -> Bool,
    }
}

table! {
    post (id) {
        id -> Integer,
        user_id -> Integer,
        title -> Text,
        deleted -> Bool,
    }
}

joinable!(post -> user (user_id));
allow_tables_to_appear_in_same_query!(user, post);
soft_delete!(user);
soft_delete!(post);

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

#[derive(Identifiable, Queryable, Debug, PartialEq)]
#[table_name = "post"]
struct Post {
    id: i32,
    user_id: i32,
    title: String,
    deleted: bool,
}

#[derive(Insertable, Default)]
#[table_name = "post"]
struct NewPost<'a> {
    user_id: i32,
    title: &'a str,
    deleted: Option<bool>,
}

fn conn() -> SqliteConnection {
    let conn = SqliteConnection::establish(":memory:").expect("Failed to open `:memory:` database");
    conn.batch_execute(
        "
        create table user(
            id integer primary key,
            name text not null,
            deleted bool not null default false
        );
        create table post(
            id integer primary key,
            user_id integer not null,
            title text not null,
            deleted bool not null default false,
            foreign key (user_id) references user(id)
        );
    ",
    )
    .expect("Failed to create `user` and/or `post` table");
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

#[test]
fn test_join_ok() {
    let conn = conn();

    diesel::insert_into(user::table).values(NewUser { name: "Joe" }).execute(&conn).unwrap();
    let joe_id: i32 = user::table.select(user::id).first(&conn).unwrap();

    diesel::insert_into(post::table)
        .values(vec![
            NewPost { user_id: joe_id, title: "My first post", ..Default::default() },
            NewPost { user_id: joe_id, title: "Failed post", deleted: Some(true) },
        ])
        .execute(&conn)
        .unwrap();

    let user_posts = user::table
        .soft_find(joe_id)
        .left_join(post::table)
        .load::<(User, Option<Post>)>(&conn)
        .unwrap();

    assert_eq!(user_posts.len(), 2);
}

#[test]
fn test_soft_left_join_ok() {
    let conn = conn();

    diesel::insert_into(user::table).values(NewUser { name: "Joe" }).execute(&conn).unwrap();
    let joe_id: i32 = user::table.select(user::id).first(&conn).unwrap();

    diesel::insert_into(post::table)
        .values(NewPost { user_id: joe_id, title: "Some post", ..Default::default() })
        .execute(&conn)
        .unwrap();

    let (_, post) = user::table
        .soft_find(joe_id)
        .soft_left_join(post::table)
        .first::<(User, Option<Post>)>(&conn)
        .unwrap();
    assert!(post.is_some());
    let post = post.unwrap();
    assert!(!post.deleted);
    assert_eq!(post.title, "Some post");

    diesel::update(&post).set(post::deleted.eq(true)).execute(&conn).unwrap();

    let (_, post) = user::table
        .soft_find(joe_id)
        .soft_left_join(post::table)
        .first::<(User, Option<Post>)>(&conn)
        .unwrap();
    assert!(post.is_none());
}

#[test]
fn test_soft_inner_join_ok() {
    let conn = conn();

    diesel::insert_into(user::table).values(NewUser { name: "Joe" }).execute(&conn).unwrap();
    let joe_id: i32 = user::table.select(user::id).first(&conn).unwrap();

    diesel::insert_into(post::table)
        .values(NewPost { user_id: joe_id, title: "Some post", ..Default::default() })
        .execute(&conn)
        .unwrap();

    let user_and_post = user::table
        .soft_find(joe_id)
        .soft_inner_join(post::table)
        .first::<(User, Post)>(&conn)
        .optional()
        .unwrap();
    assert!(user_and_post.is_some());
    let (_, post) = user_and_post.unwrap();
    assert!(!post.deleted);
    assert_eq!(post.title, "Some post");

    diesel::update(&post).set(post::deleted.eq(true)).execute(&conn).unwrap();

    let user_and_post = user::table
        .soft_find(joe_id)
        .soft_inner_join(post::table)
        .first::<(User, Post)>(&conn)
        .optional()
        .unwrap();
    assert!(user_and_post.is_none());
}
