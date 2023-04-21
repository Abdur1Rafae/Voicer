use mongodb::{Client, Collection};
use mongodb::bson::{self,oid::ObjectId, doc, Bson};
use serde::{Serialize, Deserialize};
use std::io;

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









pub async fn connect_to_mongodb() -> Collection<Users> {
    let client = Client::with_uri_str("mongodb+srv://rustuser:rustuserIBA@cluster0.3kwjnje.mongodb.net/test").await.unwrap();
    let db = client.database("Cluster0");
    let collection = db.collection::<Users>("users");
    println!("reached");
    collection
}

async fn create_user(user_collection: Collection<Users>, username: String, password: String, name: String) -> ObjectId {
    let new_user = Users {
        _id: ObjectId::new(),
        username: username,
        password: password,
        name: name,
        description: String::from(""),
        followers: Vec::new(),
        following: Vec::new(),
        voice_notes: Vec::new(),
    };
    let new_user_id = new_user.insert_one(user_collection.clone()).await;
    new_user_id
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


pub async fn sign_up(user_collection: Collection<Users>) -> ObjectId {
    println!("Please enter your username:");
    let mut username = String::new();
    io::stdin().read_line(&mut username).expect("Failed to read input.");
    username = username.trim().to_string();

    println!("Please enter your name:");
    let mut name = String::new();
    io::stdin().read_line(&mut name).expect("Failed to read input.");
    name = name.trim().to_string();

    println!("Please enter your password:");
    let mut password = String::new();
    io::stdin().read_line(&mut password).expect("Failed to read input.");
    password = password.trim().to_string();
    println!("Signup successful.");

    create_user(user_collection, username, password, name).await
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

fn main() {}

