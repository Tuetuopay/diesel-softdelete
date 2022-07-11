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

table! {
    comment (id) {
        id -> Integer,
        user_id -> Integer,
        post_id -> Integer,
        content -> Text,
        deleted -> Bool,
    }
}

joinable!(post -> user (user_id));
joinable!(comment -> user (user_id));
joinable!(comment -> post (post_id));
allow_tables_to_appear_in_same_query!(user, post, comment);
soft_delete!(user);
soft_delete!(post);
soft_delete!(comment);

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

#[derive(Identifiable, Queryable, Debug, PartialEq)]
#[table_name = "comment"]
struct Comment {
    id: i32,
    user_id: i32,
    post_id: i32,
    content: String,
    deleted: bool,
}

#[derive(Insertable, Default)]
#[table_name = "comment"]
struct NewComment<'a> {
    user_id: i32,
    post_id: i32,
    content: &'a str,
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
        create table comment(
            id integer primary key,
            user_id integer not null,
            post_id integer not null,
            content text not null,
            deleted bool not null default false,
            foreign key (user_id) references user(id),
            foreign key (post_id) references post(id)
        );
    ",
    )
    .expect("Failed to create `user`, `post` or `comment` table");
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

#[test]
fn test_nested_join_ok() {
    let conn = conn();

    diesel::insert_into(user::table)
        .values(vec![NewUser { name: "Joe" }, NewUser { name: "Jack" }])
        .execute(&conn)
        .unwrap();
    let joe: User = user::table.filter(user::name.eq("Joe")).first(&conn).unwrap();
    let jack: User = user::table.filter(user::name.eq("Jack")).first(&conn).unwrap();

    diesel::insert_into(post::table)
        .values(NewPost { user_id: joe.id, title: "Some post", ..Default::default() })
        .execute(&conn)
        .unwrap();
    let post_id: i32 = post::table.select(post::id).first(&conn).unwrap();

    diesel::insert_into(comment::table)
        .values(NewComment {
            user_id: jack.id,
            post_id,
            content: "Some comment",
            ..Default::default()
        })
        .execute(&conn)
        .unwrap();

    // Comments made by Jack on Joe's posts
    let (_, post_and_comment) = user::table
        .soft_find(joe.id)
        .left_join(post::table.left_join(comment::table))
        .first::<(User, Option<(Post, Option<Comment>)>)>(&conn)
        .unwrap();
    assert!(post_and_comment.is_some());
    let (_, comment) = post_and_comment.unwrap();
    assert!(comment.is_some());

    let user_post_comment = user::table
        .soft_find(joe.id)
        .inner_join(post::table.inner_join(comment::table))
        .first::<(User, (Post, Comment))>(&conn)
        .optional()
        .unwrap();
    assert!(user_post_comment.is_some());
}

#[test]
fn test_nested_join_inner_soft_ok() {
    let conn = conn();

    diesel::insert_into(user::table)
        .values(vec![NewUser { name: "Joe" }, NewUser { name: "Jack" }])
        .execute(&conn)
        .unwrap();
    let joe: User = user::table.filter(user::name.eq("Joe")).first(&conn).unwrap();
    let jack: User = user::table.filter(user::name.eq("Jack")).first(&conn).unwrap();

    diesel::insert_into(post::table)
        .values(NewPost { user_id: joe.id, title: "Some post", ..Default::default() })
        .execute(&conn)
        .unwrap();
    let post_id: i32 = post::table.select(post::id).first(&conn).unwrap();

    diesel::insert_into(comment::table)
        .values(NewComment {
            user_id: jack.id,
            post_id,
            content: "Some comment",
            ..Default::default()
        })
        .execute(&conn)
        .unwrap();

    // Comments made by Jack on Joe's posts
    let (_, post_and_comment) = user::table
        .soft_find(joe.id)
        .left_join(post::table.soft_left_join(comment::table))
        .first::<(User, Option<(Post, Option<Comment>)>)>(&conn)
        .unwrap();
    assert!(post_and_comment.is_some());
    let (_, comment) = post_and_comment.unwrap();
    assert!(comment.is_some());
}

// does not work at the moment
//#[test]
//fn test_nested_join_outer_soft_ok() {
//    let conn = conn();
//
//    diesel::insert_into(user::table)
//        .values(vec![NewUser { name: "Joe" }, NewUser { name: "Jack" }])
//        .execute(&conn)
//        .unwrap();
//    let joe: User = user::table.filter(user::name.eq("Joe")).first(&conn).unwrap();
//    let jack: User = user::table.filter(user::name.eq("Jack")).first(&conn).unwrap();
//
//    diesel::insert_into(post::table)
//        .values(NewPost { user_id: joe.id, title: "Some post", ..Default::default() })
//        .execute(&conn)
//        .unwrap();
//    let post_id: i32 = post::table.select(post::id).first(&conn).unwrap();
//
//    diesel::insert_into(comment::table)
//        .values(NewComment {
//            user_id: jack.id,
//            post_id,
//            content: "Some comment",
//            ..Default::default()
//        })
//        .execute(&conn)
//        .unwrap();
//
//    // Comments made by Jack on Joe's posts
//    let (_, post_and_comment) = user::table
//        .soft_find(joe.id)
//        .soft_left_join(post::table.soft_left_join(comment::table))
//        .first::<(User, Option<(Post, Option<Comment>)>)>(&conn)
//        .unwrap();
//    assert!(post_and_comment.is_some());
//    let (_, comment) = post_and_comment.unwrap();
//    assert!(comment.is_some());
//}
