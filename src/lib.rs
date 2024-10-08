use axum::async_trait;
use bitcode::{deserialize, serialize};
use serde::{de::DeserializeOwned, Serialize};
use std::{io, ptr::NonNull};
use thiserror::Error;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

#[derive(Copy, Clone, Debug)]
pub struct Pointer<T: Sized>(Option<NonNull<T>>);

impl<T: Document> Pointer<T> {
    pub fn new(reference: Option<&T>) -> Self {
        match reference {
            Some(val) => match val.id() {
                0 => Self(None),
                _ => Self(Some(val.into())),
            },
            None => Self(None),
        }
    }
    pub fn get(&self) -> Option<&T> {
        match self.0 {
            Some(v) => {
                let v = unsafe { v.as_ref() };
                match v.id() {
                    0 => None,
                    _ => Some(v),
                }
            }
            None => None,
        }
    }
}

unsafe impl<T: Document> Send for Pointer<T> {}
unsafe impl<T: Document> Sync for Pointer<T> {}

#[derive(Error, Debug)]
pub enum CollectionError {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("Bitcode error")]
    Bitcode(#[from] bitcode::Error),
}

#[async_trait]
pub trait Collection: Serialize + DeserializeOwned + Default {
    const PATH: &'static str;
    type TYPE;

    fn changed(&self) -> bool;
    fn filter(&self) -> Self;

    fn get_ref(&self) -> &Self {
        self
    }

    async fn load() -> Result<Self, CollectionError>
    where
        Self: Sized,
    {
        let mut file = File::open(Self::PATH).await?;
        let mut encoded_data = Vec::new();
        file.read_to_end(&mut encoded_data).await?;
        let data = deserialize(&encoded_data)?;
        Ok(data)
    }

    async fn reload(&mut self)
    where
        Self: Sized,
    {
        *self = Self::load().await.unwrap_or_default();
    }

    async fn save(&self) -> Result<(), CollectionError>
    where
        Self: Sized,
    {
        let encoded_data = if self.changed() {
            serialize(&self.filter())?
        } else {
            serialize(self)?
        };
        let mut file = File::create(Self::PATH).await?;
        file.write_all(&encoded_data).await?;
        Ok(())
    }
}

pub trait Document: Sized + Send {
    fn id(&self) -> u128;
}

// pub trait Optimize<T: Document> {
//     fn optimize(&mut self, ids: &Vec<u128>, values: &Vec<T>);
// }

// impl<T: Document + std::fmt::Debug> Optimize<T> for Vec<Pointer<T>> {
//     fn optimize(&mut self, ids: &Vec<u128>, values: &Vec<T>) {
//         self.iter_mut().zip(ids.iter()).for_each(|(value, id)| {
//             *value = Pointer::new(values.iter().find(|v| v.id() == *id));
//             println!("=== {:?}", value);
//         });
//     }
// }

#[macro_export]
macro_rules! collection {
    ($collection: ident, $name:ident, { $($field_name:ident : $field_type:ty),* $(,)? }, [ $($relation_name:ident($relation_name_id:ident) : $relation_type:ty),* $(,)? ], $path:expr) => {
        #[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
        pub struct $name {
            pub id: u128,
            #[serde(with = "chrono::serde::ts_seconds")]
            pub created_at: chrono::DateTime<chrono::Utc>,
            #[serde(with = "chrono::serde::ts_seconds")]
            pub updated_at: chrono::DateTime<chrono::Utc>,
            $( pub $field_name: $field_type, )*
            $(
                #[serde(skip)]
                pub $relation_name: Vec<awth::Pointer<<$relation_type as awth::Collection>::TYPE>>,
                pub $relation_name_id: Vec<u128>,
            )*
        }
        impl $name {
            pub fn new(id: u128, $( $field_name: $field_type , )* $( $relation_name_id: &Vec<u128> , )* $( $relation_name: &$relation_type , )*) -> Self {
                let mut ret = Self {
                    id,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    $( $field_name , )*
                    $(
                        $relation_name: Vec::new(),
                        $relation_name_id: $relation_name_id.clone(),
                    )*
                };
                $(
                    let mut x = Vec::with_capacity($relation_name_id.len());
                    $relation_name_id
                        .iter()
                        .for_each(|&id| x.push(awth::Pointer::new($relation_name.get(id))));
                    ret.$relation_name = x;
                )*
                ret
            }
            pub fn optimize(&mut self, $( $relation_name: &$relation_type, )*) {
                $(
                    self.$relation_name_id
                        .iter()
                        .for_each(|&id| {
                            self.$relation_name.push(awth::Pointer::new($relation_name.get(id)));
                        });
                )*
            }
        }
        impl awth::Document for $name {
            fn id(&self) -> u128 {
                self.id
            }
        }
        #[derive(serde::Serialize, serde::Deserialize, Clone, Default, Debug)]
        pub struct $collection {
            data: Vec<$name>,
            changed: bool
        }
        impl $collection {
            pub const fn empty() -> Self {
                Self { data: Vec::new(), changed: false }
            }
            pub const fn new(data: Vec<$name>) -> Self {
                Self { data, changed: false }
            }
            pub fn add(&mut self, data: $name) {
                if data.id == 0 { return; }
                if !self.data.iter().any(|d| d.id == data.id) {
                    self.data.push(data);
                    self.changed = true;
                }
            }
            pub fn update(&mut self, data: $name) {
                if data.id == 0 { return; }
                if let Some(d) = self.data.iter_mut().find(|d| d.id == data.id) {
                    *d = data;
                    self.changed = true;
                }
            }
            pub fn remove(&mut self, id: u128) {
                if id == 0 { return; }
                if let Some(d) = self.data.iter_mut().filter(|d| d.id == id).next() {
                    d.id = 0;
                    self.changed = true;
                }
            }
            pub fn get(&self, id: u128) -> Option<&$name> {
                self.data.iter().find(|d| d.id == id)
            }
            pub fn iter(&self) -> std::slice::Iter<'_, $name> {
                self.data.iter()
            }
            pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, $name> {
                self.data.iter_mut()
            }
            pub fn optimize(&mut self, $( $relation_name: &$relation_type, )*) -> Self {
                self.iter_mut().for_each(|x| x.optimize($($relation_name,)*));
                self.to_owned()
            }
        }
        impl awth::Collection for $collection {
            const PATH: &'static str = $path;
            type TYPE = $name;
            fn changed(&self) -> bool {
                self.changed
            }
            fn filter(&self) -> Self {
                Self {
                    data: self.data.clone().into_iter()
                            .filter(|data| data.id != 0)
                            .collect::<Vec<_>>(),
                    changed: false,
                }
            }
        }
    };
    ($collection: ident, $name:ident, { $($field_name:ident : $field_type:ty),* $(,)? }, $path:expr) => {
        #[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
        pub struct $name {
            pub id: u128,
            #[serde(with = "chrono::serde::ts_seconds")]
            pub created_at: chrono::DateTime<chrono::Utc>,
            #[serde(with = "chrono::serde::ts_seconds")]
            pub updated_at: chrono::DateTime<chrono::Utc>,
            $( pub $field_name: $field_type, )*
        }
        impl $name {
            pub fn new(id: u128, $( $field_name: $field_type, )*) -> Self {
                Self {
                    id,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    $( $field_name , )*
                }
            }
        }
        #[derive(serde::Serialize, serde::Deserialize, Clone, Default, Debug)]
        pub struct $collection {
            data: Vec<$name>,
            changed: bool
        }
        impl $collection {
            pub const fn empty() -> Self {
                Self { data: Vec::new(), changed: false }
            }
            pub const fn new(data: Vec<$name>) -> Self {
                Self { data, changed: false }
            }
            pub fn add(&mut self, data: $name) {
                if data.id == 0 { return; }
                if !self.data.iter().any(|d| d.id == data.id) {
                    self.data.push(data);
                    self.changed = true;
                }
            }
            pub fn update(&mut self, data: $name) {
                if data.id == 0 { return; }
                if let Some(d) = self.data.iter_mut().filter(|d| d.id == data.id).next() {
                    *d = data;
                    self.changed = true;
                }
            }
            pub fn remove(&mut self, id: u128) {
                if id == 0 { return; }
                if let Some(d) = self.data.iter_mut().filter(|d| d.id == id).next() {
                    d.id = 0;
                    self.changed = true;
                }
            }
            pub fn get(&self, id: u128) -> Option<&$name> {
                self.data.iter().find(|d| d.id == id)
            }
            pub fn iter(&self) -> std::slice::Iter<'_, $name> {
                self.data.iter()
            }
            pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, $name> {
                self.data.iter_mut()
            }
        }
        impl awth::Document for $name {
            fn id(&self) -> u128 {
                self.id
            }
        }
        impl awth::Collection for $collection {
            const PATH: &'static str = $path;
            type TYPE = $name;
            fn changed(&self) -> bool {
                self.changed
            }
            fn filter(&self) -> Self {
                Self {
                    data: self.data.clone().into_iter()
                            .filter(|data| data.id != 0)
                            .collect::<Vec<_>>(),
                    changed: false,
                }
            }
        }
    };
}
