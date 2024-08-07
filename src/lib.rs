use axum::async_trait;
use bitcode::{deserialize, serialize, DecodeOwned, Encode};
use serde::{de::DeserializeOwned, Serialize};
use std::io;
use thiserror::Error;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

#[derive(Error, Debug)]
pub enum CollectionError {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("Bitcode error")]
    Bitcode(#[from] bitcode::Error),
}

pub type ID = u128;

#[async_trait]
pub trait Collection: Encode + DecodeOwned + Serialize + DeserializeOwned + Default {
    const PATH: &'static str;

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

#[macro_export]
macro_rules! collection {
    ($collection: ident, $name:ident, { $($field_name:ident : $field_type:ty),* $(,)? }, $path:expr) => {
        #[derive(bitcode::Encode, bitcode::Decode, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            pub id: ID,
            $( pub $field_name: $field_type, )*
        }
        #[derive(bitcode::Encode, bitcode::Decode, serde::Serialize, serde::Deserialize, Default)]
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
            pub fn remove(&mut self, id: ID) {
                if id == 0 { return; }
                if let Some(d) = self.data.iter_mut().filter(|d| d.id == id).next() {
                    d.id = 0;
                }
            }
            pub fn get(&self, id: ID) -> Option<&$name> {
                self.data.iter().find(|d| d.id == id)
            }
            pub fn iter(&self) -> std::slice::Iter<'_, $name> {
                self.data.iter()
            }
            pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, $name> {
                self.data.iter_mut()
            }
        }
        impl Collection for $collection {
            const PATH: &'static str = $path;
        }
    };
}
