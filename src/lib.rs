use axum::async_trait;
use bitcode::{deserialize, serialize};
use serde::{de::DeserializeOwned, Serialize};
use std::{io, ptr::NonNull};
use thiserror::Error;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

pub struct Pointer<T: Sized>(Option<NonNull<T>>);

impl<T: Sized> Pointer<T> {
    pub fn new(reference: Option<&T>) -> Self {
        match reference {
            Some(val) => Self(Some(val.into())),
            None => Self(None),
        }
    }
    pub fn get(&self) -> Option<&T> {
        match self.0 {
            Some(v) => Some(unsafe { v.as_ref() }),
            None => None,
        }
    }
}

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

    async fn save(&self) -> Result<(), CollectionError>
    where
        Self: Sized,
    {
        let encoded_data = serialize(self)?;
        let mut file = File::create(Self::PATH).await?;
        file.write_all(&encoded_data).await?;
        Ok(())
    }
}

pub trait Document: Sized {
    fn id(&self) -> u128;
}

pub trait Optimize<T: Document> {
    fn optimize(&mut self, ids: &Vec<u128>, values: &Vec<T>);
}

impl<T: Document> Optimize<T> for Vec<Pointer<T>> {
    fn optimize(&mut self, ids: &Vec<u128>, values: &Vec<T>) {
        self.iter_mut().zip(ids.iter()).for_each(|(value, id)| {
            *value = Pointer::new(values.iter().find(|v| v.id() == *id));
        });
    }
}

#[macro_export]
macro_rules! collection {
    ($collection: ident, $name:ident, { $($field_name:ident : $field_type:ty),* $(,)? }, [ $($relation_name:ident($relation_name_id:ident) : $relation_type:ty),* $(,)? ], $path:expr) => {
        #[derive(serde::Serialize, serde::Deserialize)]
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
                    id: 1,
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
        }
        impl awth::Document for $name {
            fn id(&self) -> u128 {
                self.id
            }
        }
        #[derive(serde::Serialize, serde::Deserialize, Default)]
        pub struct $collection {
            data: Vec<$name>
        }
        impl $collection {
            pub const fn new(data: Vec<$name>) -> Self {
                Self { data }
            }
            pub fn add(&mut self, data: $name) {
                if data.id == 0 { return; }
                if !self.data.iter().any(|d| d.id == data.id) {
                    if let Some(d) = self.data.iter_mut().filter(|d| d.id == 0).next() {
                        *d = data;
                    } else {
                        self.data.push(data);
                    }
                }
            }
            pub fn update(&mut self, data: $name) {
                if data.id == 0 { return; }
                if let Some(d) = self.data.iter_mut().filter(|d| d.id == data.id).next() {
                    *d = data;
                }
            }
            pub fn remove(&mut self, id: u128) {
                if id == 0 { return; }
                if let Some(d) = self.data.iter_mut().filter(|d| d.id == id).next() {
                    d.id = 0;
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
        impl awth::Collection for $collection {
            const PATH: &'static str = $path;
            type TYPE = $name;
        }
    };
    ($collection: ident, $name:ident, { $($field_name:ident : $field_type:ty),* $(,)? }, $path:expr) => {
        #[derive(serde::Serialize, serde::Deserialize)]
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
                    id: 1,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    $( $field_name , )*
                }
            }
        }
        #[derive(serde::Serialize, serde::Deserialize, Default)]
        pub struct $collection {
            data: Vec<$name>
        }
        impl $collection {
            pub fn add(&mut self, data: $name) {
                if data.id == 0 { return; }
                if !self.data.iter().any(|d| d.id == data.id) {
                    if let Some(d) = self.data.iter_mut().filter(|d| d.id == 0).next() {
                        *d = data;
                    } else {
                        self.data.push(data);
                    }
                }
            }
            pub fn update(&mut self, data: $name) {
                if data.id == 0 { return; }
                if let Some(d) = self.data.iter_mut().filter(|d| d.id == data.id).next() {
                    *d = data;
                }
            }
            pub fn remove(&mut self, id: u128) {
                if id == 0 { return; }
                if let Some(d) = self.data.iter_mut().filter(|d| d.id == id).next() {
                    d.id = 0;
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
        impl awth::Collection for $collection {
            const PATH: &'static str = $path;
            type TYPE = $name;
        }
    };
}
