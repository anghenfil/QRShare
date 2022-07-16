use chrono::NaiveDateTime;
use std::collections::HashMap;
use std::sync::RwLock;
use rand::prelude::*;
use rand::distributions::Alphanumeric;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct FileMetadata{
    //unique file id, physical file will be saved under uploads/<id>
    pub id: String,
    //Encrypted file name
    pub file_name: String,
    //Initialisation Vector for encryption
    pub file_iv: String,
    pub filename_iv: String,
    //If set to Some, file will be deleted after specified datetime
    pub delete_after: Option<NaiveDateTime>,
}

pub struct FileMetadataStorage{
    pub storage: RwLock<HashMap<String, FileMetadata>>
}

impl FileMetadataStorage{
    fn get_id(&self) -> String{
        loop{
            let id: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(8)
                .map(char::from)
                .collect();

            if !self.storage.read().unwrap().contains_key(&id){
                return id
            }
        }
    }

    pub fn add(&self, file_name: String, file_iv: String, filename_iv: String, delete_after: Option<NaiveDateTime>) -> FileMetadata{
        let id = self.get_id();
        let fm = FileMetadata{
            id: id.clone(),
            file_name,
            file_iv,
            delete_after,
            filename_iv
        };
        self.storage.write().unwrap().insert(id, fm.clone());
        fm
    }

}
