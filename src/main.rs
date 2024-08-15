use awth::*;
use axum::{extract::Path, response::IntoResponse, routing::get, Router};
use lazy_static::lazy_static;
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
        .route("/", get(route))
        .route("/save", get(save_now))
        .route("/login/:email/:password", get(login))
        .route("/register/:username/:email/:password", get(register))
        .route("/post/:id/:caption", get(post));
    tokio::spawn(save());

    let listener = TcpListener::bind("0.0.0.0:6666").await?;
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

async fn register(
    Path((email, username, password)): Path<(String, String, String)>,
) -> impl IntoResponse {
    USERS.lock().await.add(User::new(
        username.len() as u128,
        username,
        email,
        password.len() as u128,
        &vec![],
        &POSTS.lock().await.get_ref(),
    ));

    format!(
        "{}\n",
        USERS
            .lock()
            .await
            .iter()
            .last()
            .map(|u| u.id)
            .unwrap_or(404)
    )
}

async fn login(Path((email, password)): Path<(String, String)>) -> impl IntoResponse {
    USERS
        .lock()
        .await
        .iter()
        .find(|u| u.email == email && password.len() > 0)
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

async fn post(Path((id, caption)): Path<(u128, String)>) -> impl IntoResponse {
    let id_ = id + caption.len() as u128;
    let p = Post::new(id_, caption);
    println!("adding!");
    POSTS.lock().await.add(p);
    println!("post added!");

    let user = USERS.lock().await.get(id).cloned();
    if let Some(mut user) = user {
        user.post_ids.push(id_);
        user.posts.push(Pointer::new(POSTS.lock().await.get(id_)));
        USERS.lock().await.update(user);

        "Done!\n"
    } else {
        "Err!\n"
    }
}
