use awth::{collection, Collection, Optimize};
// #[derive(Clone, Encode, Serialize)]
// struct UsersSer {
//     data: Vec<UserSer>,
// }

// #[derive(Debug)]
// struct Users<'a> {
//     data: Vec<User<'a>>,
// }

// #[derive(Clone, Encode, Serialize)]
// struct UserSer {
//     id: u128,
//     posts: Vec<u128>,
//     username: String,
//     password: String,
// }

// #[derive(Debug)]
// struct User<'a> {
//     id: u128,
//     posts: Vec<&'a Post>,
//     username: String,
//     password: String,
// }

// #[derive(Debug, Encode, Serialize)]
// struct Post {
//     id: u128,
//     caption: String,
// }
// #[derive(Encode, Serialize)]
// struct Posts {
//     data: Vec<Post>,
// }

// impl<'a> Users<'a> {
//     fn fast(from: UsersSer, posts: &'a Posts) -> Self {
//         Self {
//             data: from
//                 .to_owned()
//                 .data
//                 .iter()
//                 .map(|user| User {
//                     id: user.id,
//                     username: user.username.to_owned(),
//                     password: user.password.to_owned(),
//                     posts: user
//                         .posts
//                         .iter()
//                         .filter_map(|post_id| {
//                             posts.data.iter().filter(|post| post.id == *post_id).next()
//                         })
//                         .collect::<Vec<&Post>>(),
//                 })
//                 .collect::<Vec<_>>(),
//         }
//     }
// }

collection!(Posts, Post, {
    caption: String
}, "test/postz.db");

collection!(Users, User, {
    username: String,
    email: String,
    hashed_password: u128,
}, [posts(post_ids): Posts], "test/userz.db");

#[tokio::main]
async fn main() {
    let posts = Posts::load().await.unwrap_or_default();
    // posts.add(Post::new(1, "1 bobaha".into()));
    // posts.add(Post::new(2, "2 ghaly".into()));
    // posts.add(Post::new(3, "3 khal".into()));
    // posts.add(Post::new(4, "4 mad3ak".into()));
    // posts.add(Post::new(5, "5 obk".into()));

    let mut users = Users::load().await.unwrap_or_default();
    users
        .iter_mut()
        .for_each(|user| user.posts.optimize(&user.post_ids, &posts.data));
    // users.add(User::new(
    //     1,
    //     "noice".into(),
    //     "hehe@gmail.com".into(),
    //     123,
    //     &vec![1, 3, 4],
    //     &posts,
    // ));
    // users.add(User::new(
    //     2,
    //     "rayen".into(),
    //     "rayen@gmail.com".into(),
    //     12345,
    //     &vec![2],
    //     &posts,
    // ));

    println!("{:#?}", posts);
    println!("{:#?}", users);

    println!("=========================");
    users.iter().map(|u| &u.posts).for_each(|ps| {
        ps.iter()
            .filter_map(|p| p.get())
            .for_each(|p| println!("{:?}", p.caption));
    });

    // users.save().await.unwrap();
    // posts.save().await.unwrap();
}
