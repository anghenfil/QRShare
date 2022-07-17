#[macro_use] extern crate rocket;
use crate::storage::FileMetadataStorage;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::ErrorKind::NotFound;
use std::sync::RwLock;
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
}

#[post("/upload", data = "<upload>")]
async fn upload(upload: Form<Upload<'_>>, fms: &State<FileMetadataStorage>) -> Result<String, Status> {
    let mut upload = upload.into_inner();
    let file_metadata = fms.add(upload.encrypted_filename, upload.file_iv, upload.filename_iv, None);

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
fn get_metadata(id: String, fmt: &State<FileMetadataStorage>) -> Result<Json<FileMetadata>, Status> {
    match fmt.storage.read().unwrap().get(&id){
        Some(fm) => Ok(Json(fm.clone())),
        None => Err(Status::NotFound)
    }
}


#[get("/d/<id>")]
fn decrypt(id: String, fmt: &State<FileMetadataStorage>) -> Result<Template, Status> {
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
    fs::remove_dir_all(path).unwrap();
    fs::create_dir(path).unwrap();

    rocket::build().mount("/", routes![index, upload, download, decrypt, get_metadata]).mount("/css/", FileServer::from("res/css")).mount("/js/", FileServer::from("res/js")).attach(Template::fairing()).manage(storage)
}