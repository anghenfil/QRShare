#[macro_use] extern crate rocket;
use crate::storage::FileMetadataStorage;
use std::collections::HashMap;
use std::{fs, thread, time};
use std::fs::File;
use std::io::ErrorKind::NotFound;
use std::sync::{Arc, RwLock};
use chrono::{Duration, NaiveDateTime, Utc};
use crate::storage::FileMetadata;
use rocket::serde::json::Json;
use rocket::form::Form;
use rocket::fs::{FileServer, TempFile};
use rocket::http::Status;
use rocket::State;
use rocket_dyn_templates::{context, Template};

pub mod storage;

#[get("/")]
fn index() -> Template {
    Template::render("index", context!{})
}

#[derive(FromForm)]
struct Upload<'r> {
    encrypted_filename: String,
    filename_iv: String,
    file: TempFile<'r>,
    file_iv: String,
    auto_delete: Option<String>
}

#[post("/upload", data = "<upload>")]
async fn upload(upload: Form<Upload<'_>>, fms: &State<Arc<FileMetadataStorage>>) -> Result<String, Status> {
    let mut upload = upload.into_inner();

    let delete_after : NaiveDateTime = match upload.auto_delete{
        Some(ad) => {
            match ad.parse(){
                Ok(mut ad) => {
                    if ad > 1440{ //Do not allow values greater than 24 hours!
                        ad = 1440;
                    }
                    match Utc::now().naive_utc().checked_add_signed(Duration::minutes(ad)){
                        None => {
                            eprintln!("Submitted auto_delete number is to big!");
                            return Err(Status::BadRequest);
                        }
                        Some(ad) => ad
                    }
                },
                Err(e) => {
                    eprintln!("Couldn't parse submitted auto_delete string as number: {}", e);
                    return Err(Status::BadRequest);
                }
            }
        },
        None => Utc::now().naive_utc().checked_add_signed(Duration::minutes(1440)).unwrap() //Setting default auto delete to 24 hours
    };

    let file_metadata = fms.add(upload.encrypted_filename, upload.file_iv, upload.filename_iv, Some(delete_after));
    println!("Autodelete file at {}.", delete_after.to_string());
    let id = file_metadata.id;
    match upload.file.move_copy_to(format!("uploads/{}", id)).await{
        Ok(_) => {
            println!("uploaded to: uploads/{}", id);
            Ok(id)
        },
        Err(e) => {
            eprintln!("Couldn't move uploaded file: {}", e);
            Err(Status::InternalServerError)
        }
    }
}

#[get("/download/<filename>")]
fn download(filename: String) -> Result<File, Status>{
    let f = match File::open(format!("uploads/{}", filename)){
        Ok(f) => f,
        Err(e) => {
            if e.kind() == NotFound{
                eprintln!("Couldn't requested find file: {}", e);
                return Err(Status::NotFound)
            }else{
                eprintln!("Couldn't load file: {}", e);
                return Err(Status::InternalServerError)
            }
        }
    };
    Ok(f)
}

#[get("/metadata/<id>")]
fn get_metadata(id: String, fmt: &State<Arc<FileMetadataStorage>>) -> Result<Json<FileMetadata>, Status> {
    match fmt.storage.read().unwrap().get(&id){
        Some(fm) => Ok(Json(fm.clone())),
        None => Err(Status::NotFound)
    }
}


#[get("/d/<id>")]
fn decrypt(id: String, fmt: &State<Arc<FileMetadataStorage>>) -> Result<Template, Status> {
    if !fmt.storage.read().unwrap().contains_key(&id){
        return Err(Status::NotFound)
    }else{
        Ok(Template::render("decrypt", context! {}))
    }
}

#[launch]
fn rocket() -> _ {
    let storage = FileMetadataStorage{
        storage: RwLock::new(HashMap::new()),
    };

    //Clear upload directory
    let path = "uploads";
    fs::remove_dir_all(path);
    fs::create_dir(path).unwrap();

    let storage_copy = Arc::from(storage);
    let storage_copy1 = storage_copy.clone();
    //Spawn thread to monitor and execute auto deletion
    thread::spawn(move ||{
        let storage_copy2 = storage_copy1.clone();
        loop{
            let mut deletion_list = vec![];
            for (id, fm) in storage_copy2.storage.read().unwrap().iter(){
                if let Some(delete_after) = fm.delete_after{
                    if delete_after.le(&Utc::now().naive_utc()){ //File is due for deletion
                        deletion_list.push(id.clone());
                        if let Err(e) = fs::remove_file(format!("{}/{}", path, id)){
                            eprintln!("Couldn't delete physical file: {}", e);
                        }
                    }
                }
            }
            for id in deletion_list{
                storage_copy2.storage.write().unwrap().remove(&id);
            }
            thread::sleep(time::Duration::from_secs(60));
        }
    });

    rocket::build().mount("/", routes![index, upload, download, decrypt, get_metadata]).mount("/css/", FileServer::from("res/css")).mount("/js/", FileServer::from("res/js")).attach(Template::fairing()).manage(storage_copy)
}