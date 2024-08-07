# Awth

### Overview
`Awth` is a Rust library designed to simplify database management. With macros, you can easily create databases and collections, and perform operations like load, save, update, and delete. Optimized with async IO functionality for efficient performance.

### Features
- Easy creation of collections using macros
- Simple load, save, update, and delete operations
- Tiny file size for efficient storage
- Asynchronous IO for non-blocking operations
- Blazingly FAST

### Goals (TODO)
- [ ] Add Post/Get functionality (axum)
- [ ] JWT Authentication
- [ ] Relations between collections
- [ ] More Optimizations (Compressing files, Faster functions: Adding, Removing, Updating, ...)

### Usage
```rust
// Creation
//    collection_name
//            | document_name
//            |     |
//            v     v
collection!(Users, User, {
    // id added by default
    username: String,
    email: String,
    hashed_password: String,
}, "test/users.db");
//         ^
//         |
//    path/file_name

#[tokio::main]
async fn main() {
    // Loading
    //                     ( create empty collection if file doesnt exist )
    //                                          |
    //                                          v
    let mut users = Users::load().await.unwrap_or_default();

    // Add Document
    users.add(User {
      id: 1, // must be unique to add
      username: "username".into(),
      email: "email@email.com".into(),
      hashed_password: "hcuiehiyiudezuiicjsdkvjnsnqjkcqsc".into(),
    });

    // Updating
    users.update(User {
      id: 1, // ID of the user you want to update
      username: "username_2".into(),
      email: "email_2@email.com".into(),
      hashed_password: "ioduiaejhckjsdccnjazejhzkeac".into(),
    });

    // Removing by ID
    // id with type ID (u128)
    users.remove(1);

    // Getting Data
    // 1- By ID
    let Some(user) = users.get(1) {
      // user exists
    }
    // 2- All (iter / iter_mut)
    users
        .iter()
        .filter(|u| u.username.len() > 5)
        .for_each(|u| println!("{}", u.id));

    // Saving
    users.save().await.expect("ERROR: couldn't save");
}
```

### Contributing
Contributions are welcome!

### License
`Awth` is licensed under the MIT License.

