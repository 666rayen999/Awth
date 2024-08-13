use awth::collection;
use chrono::Utc;
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

struct Database {
    users: Users,
    posts: Posts,
}

fn main() {
    let mut posts = Posts {
        data: vec![
            Post {
                id: 1,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                caption: "1 bobaha".into(),
            },
            Post {
                id: 2,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                caption: "2 ghaly".into(),
            },
            Post {
                id: 3,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                caption: "3 khal".into(),
            },
            Post {
                id: 4,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                caption: "4 mad3ak".into(),
            },
            Post {
                id: 5,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                caption: "5 obk".into(),
            },
        ],
    };

    let users = Users::new(vec![
        User::new(
            1,
            "noice".into(),
            "hehe@gmail.com".into(),
            123,
            &vec![1, 3],
            &posts,
        ),
        User::new(
            2,
            "rayen".into(),
            "rayen@gmail.com".into(),
            12345,
            &vec![2],
            &posts,
        ),
    ]);

    // users.data.iter_mut().for_each(|user| {
    //     let ids = user.post_ids.clone();
    //     ids.iter()
    //         .for_each(|id| user.posts.push(Pointer::new(&posts.get(*id).unwrap())))
    // });

    users.iter().map(|u| &u.posts).for_each(|ps| {
        ps.iter()
            .for_each(|p| println!("{:?}", p.get().unwrap().caption))
    });

    users.iter().map(|u| &u.posts).for_each(|ps| {
        ps.iter()
            .for_each(|p| println!("{:?}", p.get().unwrap().caption))
    });

    // let users = Users::fast(
    //     UsersSer {
    //         data: vec![
    //             UserSer {
    //                 id: 1,
    //                 username: "noice".into(),
    //                 password: "123".into(),
    //                 posts: vec![1, 3, 4],
    //             },
    //             UserSer {
    //                 id: 2,
    //                 username: "nmrood".into(),
    //                 password: "RRRrrr".into(),
    //                 posts: vec![2, 5],
    //             },
    //         ],
    //     },
    //     &posts,
    // );
    // println!("{:#?}", users);
}
