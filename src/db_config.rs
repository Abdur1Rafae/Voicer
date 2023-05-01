use mongodb::{Client, Collection  , Database,options::{ClientOptions, GridFsBucketOptions},};
use mongodb_gridfs::{options::GridFSBucketOptions, GridFSBucket,  GridFSError};
use mongodb::options::FindOneOptions;
use mongodb::options::FindOneAndUpdateOptions;
use futures::stream::StreamExt;
use futures_util::io::AsyncReadExt;
use mongodb::bson::{self,oid::ObjectId, doc, Bson};
use serde::{Serialize, Deserialize};
use mongodb::options::UpdateOptions;
use std::io;
use std::fs::File;
use std::env;
use std::io::Read;
use std::str::FromStr;
use std::io::prelude::*;
use rodio::Sink;
use pyo3::prelude::*;
use pyo3::types::PyTuple;


#[derive(Serialize, Deserialize, Debug)]
pub struct Users {
    pub _id: bson::oid::ObjectId,
    pub username:String,
    pub password: String,
    pub name: String,
    pub description: String,
    pub followers:Vec<ObjectId>,
    pub following:Vec<ObjectId>,
    pub voice_notes:Vec<ObjectId>
}

impl Users {
    pub async fn insert_one(&self, collection: Collection<Users>) -> ObjectId {
        let new_user = self.clone();
        let new_user_id = ObjectId::new();
        collection.insert_one(new_user, None).await.unwrap();
        new_user_id
    }
}

pub enum ReactionType{
    SpeakUp,
    ShutUp,
}

pub struct Reaction{
    user_id:ObjectId,
    reaction: ReactionType
}

pub struct VoiceNote{
    v_id:bson::oid::ObjectId,
    user_id:ObjectId,
    is_post:bool,
    replies:Vec<ObjectId>,
    reactions:Vec<Reaction>,
}

pub async fn connect_to_mongodb() -> (Collection<Users>, Database, Client) {
    let client = Client::with_uri_str("mongodb+srv://RustUser:RUSTIBA@cluster0.btmwmdh.mongodb.net/test").await.unwrap();
    let db = client.database("Cluster0");
    let collection = db.collection::<Users>("users");
    println!("reached");
    (collection , db , client)
}

async fn create_user(user_collection: Collection<Users>, username: String, password: String, name: String) -> ObjectId {
    let new_user = Users {
        _id: ObjectId::new(),
        username: username.clone(),
        password: password,
        name: name,
        description: String::from(""),
        followers: Vec::new(),
        following: Vec::new(),
        voice_notes: Vec::new(),
    };
    
    // Check if a user with the given username exists in the collection
    let filter = doc! { "username": username };
    let result = user_collection.find_one(filter, None).await;
    let user:ObjectId = match result.expect("Error finding user") {
        Some(user) => { 
            println!("User with email already exists");
            ObjectId::parse_str("f0f0f0f0f0f0f0f0f0f0f0f0").unwrap()},
        None => {
            println!("Creating new user");
            new_user.insert_one(user_collection.clone()).await
   
        }
    };
        
    user
}

async fn get_user_by_username(collection: Collection<Users>, username: String, password: String) -> Option<Users> {
    let filter = doc! { "username": username };

    let mut user;

    match collection.find_one(filter, None).await {
        Ok(result) => match result {
            Some(doc) => {
                if doc.password==password {
                    user=Some(doc)
                }
                else{
                    println!("Wrong password");
                    user= None
                }
            }
            None => user= None,
        },
        Err(e) => {
            println!("Failed to get user: {}", e);
            user = None
        }
    };
    println!("{:?}", user);
    user
}

pub async fn update_user_name_by_username(user_collection: Collection<Users>, username: &str, new_name: &str) -> bool {
    let filter = doc! { "username": username };
    let update = doc! { "$set": { "name": new_name } };
    let options = FindOneAndUpdateOptions::builder().return_document(mongodb::options::ReturnDocument::After).build();
    if let Ok(updated_user) = user_collection.find_one_and_update(filter, update, options).await {
        return true;
    }
    false
}
pub async fn update_password_by_username(user_collection: Collection<Users>, username: &str, new_password: &str) -> bool {
    let filter = doc! { "username": username };
    let update = doc! { "$set": { "password": new_password } };
    let options = FindOneAndUpdateOptions::builder().return_document(mongodb::options::ReturnDocument::After).build();
    if let Ok(updated_user) = user_collection.find_one_and_update(filter, update, options).await {
        return true;
    }
    false
}
pub async fn update_description_by_username(user_collection: Collection<Users>, username: &str, new_desc: &str) -> bool {
    let filter = doc! { "username": username };
    let update = doc! { "$set": { "description": new_desc } };
    let options = FindOneAndUpdateOptions::builder().return_document(mongodb::options::ReturnDocument::After).build();
    if let Ok(updated_user) = user_collection.find_one_and_update(filter, update, options).await {
        return true;
    }
    false
}

pub async fn sign_up(user_collection: Collection<Users>) -> ObjectId {
    println!("Please enter your email:");
    let mut email = String::new();
    io::stdin().read_line(&mut email).expect("Failed to read input.");
    email = email.trim().to_string();

    println!("Please enter your name:");
    let mut name = String::new();
    io::stdin().read_line(&mut name).expect("Failed to read input.");
    name = name.trim().to_string();

    println!("Please enter your password:");
    let mut password = String::new();
    io::stdin().read_line(&mut password).expect("Failed to read input.");
    password = password.trim().to_string();
    

    let mut new_user_id = create_user(user_collection, email, password, name).await;
    
    new_user_id
}

pub async fn login(user_collection: Collection<Users>) -> Option<Users> {
    println!("Please enter your username:");
    let mut username = String::new();
    io::stdin().read_line(&mut username).expect("Failed to read input.");
    username = username.trim().to_string();

    println!("Please enter your password:");
    let mut password = String::new();
    io::stdin().read_line(&mut password).expect("Failed to read input.");
    password = password.trim().to_string();

    get_user_by_username(user_collection, username, password).await
}

pub async fn save_voice_note(userid: ObjectId) -> PyResult<()>{

    
        Ok(())
     
}


pub async fn play_audio(v_id : &str) -> PyResult<()> {
        Ok(())
     
}

fn main() {}
