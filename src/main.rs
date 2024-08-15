use awth::*;
use axum::{response::IntoResponse, routing, Json, Router};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::time::Duration;
use tokio::{net::TcpListener, sync::Mutex, time::sleep};

collection!(Posts, Post, {
    caption: String
}, "test/postz.db");

collection!(Users, User, {
    username: String,
    email: String,
    hashed_password: u128,
}, [posts(post_ids): Posts], "test/userz.db");

lazy_static! {
    static ref POSTS: Mutex<Posts> = Mutex::new(Posts::empty());
    static ref USERS: Mutex<Users> = Mutex::new(Users::empty());
}

#[tokio::main]
async fn main() -> Result<(), CollectionError> {
    load().await?;

    let app = Router::new()
        .route("/", routing::get(route))
        .route("/save", routing::get(save_now))
        .route("/api/user", routing::get(login))
        .route("/api/user", routing::post(register))
        .route("/api/post", routing::post(post_share));
    tokio::spawn(save());

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn route() -> impl IntoResponse {
    "welcome!\n"
}

async fn load() -> Result<(), CollectionError> {
    println!("loading...");
    POSTS.lock().await.reload().await;
    USERS.lock().await.reload().await;
    USERS.lock().await.optimize(POSTS.lock().await.get_ref());
    println!("loaded!");
    Ok(())
}

async fn ctrl_s() -> Result<(), CollectionError> {
    println!("saving...");
    POSTS.lock().await.save().await?;
    USERS.lock().await.save().await?;
    println!("saved!");
    Ok(())
}

async fn save() -> Result<(), CollectionError> {
    loop {
        sleep(Duration::from_secs(1000)).await;
        ctrl_s().await?
    }
}

async fn save_now() -> impl IntoResponse {
    if ctrl_s().await.is_ok() {
        "\n"
    } else {
        "Err: not saved!"
    }
}

#[derive(Deserialize)]
struct CreateUser {
    username: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginUser {
    user: String,
    password: String,
}

#[derive(Deserialize)]
struct PostShare {
    user_id: u128,
    caption: String,
}

async fn register(Json(payload): Json<CreateUser>) {
    USERS.lock().await.add(User::new(
        payload.username.len() as u128,
        payload.username,
        payload.email,
        payload.password.len() as u128,
        &vec![],
        &POSTS.lock().await.get_ref(),
    ));
}

async fn login(Json(payload): Json<LoginUser>) -> impl IntoResponse {
    USERS
        .lock()
        .await
        .iter()
        .find(|u| {
            (u.email == payload.user || u.username == payload.user) && payload.password.len() > 0
        })
        .map(|u| {
            format!(
                "{}: {:?}\n",
                u.username,
                u.posts
                    .iter()
                    .filter_map(|p| p.get())
                    .map(|p| &p.caption)
                    .collect::<Vec<_>>()
            )
        })
        .unwrap_or("404\n".into())
}

async fn post_share(Json(payload): Json<PostShare>) -> impl IntoResponse {
    let id = payload.user_id + payload.caption.len() as u128;
    let p = Post::new(id, payload.caption);
    println!("adding!");
    POSTS.lock().await.add(p);
    println!("post added!");

    let user = USERS.lock().await.get(payload.user_id).cloned();
    if let Some(mut user) = user {
        user.post_ids.push(id);
        user.posts.push(Pointer::new(POSTS.lock().await.get(id)));
        USERS.lock().await.update(user);

        "Done!\n"
    } else {
        "Err!\n"
    }
}
