use mongodb::{Client, Collection  , Database};
use mongodb::options::FindOneAndUpdateOptions;
use mongodb::bson::{self,oid::ObjectId, doc};
use mongodb::options::UpdateOptions;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io;
use futures_util::StreamExt;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct Users {
    pub _id: bson::oid::ObjectId,
    pub username:String,
    pub password: String,
    pub name: String,
    pub description: String,
    // pub verified: bool,
    pub followers:Vec<ObjectId>,
    pub following:Vec<ObjectId>,
    pub voice_notes:Vec<ObjectId>
}

impl Users {
    pub async fn insert_one(&self, collection: Collection<Users>) {
        let new_user = self.clone();
        collection.insert_one(new_user, None).await.unwrap();
    }

    // pub fn new(name: &str, age: u8, email: &str) -> Self {
    //     Self {
    //         name: name.to_string(),
    //         age,
    //         email: email.to_string(),
    //     }
    // }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ReactionType{
    SpeakUp,
    ShutUp,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Reaction{
    user_id:ObjectId,
    reaction: ReactionType
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VoiceNote {
    pub _id: ObjectId,
    pub user_id: ObjectId,
    pub is_post: bool,
    pub replies: Vec<ObjectId>,
    pub reactions: Vec<Reaction>,
    #[serde(with = "chrono::serde::ts_seconds")]
    timestamp: DateTime<Utc>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct publicUser{
    pub refNo: i32,
    pub _id: ObjectId,
    pub username:String,
    pub name: String,
    pub description: String,
    // pub verified: bool,
    pub followers:Vec<ObjectId>,
    pub following:Vec<ObjectId>,
    pub voice_notes:Vec<ObjectId>
}

impl VoiceNote{
    pub async fn insert_one(&self, collection: Collection<VoiceNote>) {
        let new_vn = self.clone();
        collection.insert_one(new_vn, None).await.unwrap();
    }
}

pub async fn connect_to_mongodb() -> (Collection<Users>, Collection<VoiceNote>, Database, Client) {
    let client = Client::with_uri_str("mongodb+srv://RustUser:RUSTIBA@cluster0.btmwmdh.mongodb.net/test").await.unwrap();
    let db = client.database("Cluster0");
    let collection = db.collection::<Users>("users");
    let vcollection: Collection<VoiceNote>= db.collection::<VoiceNote>("Voice Notes");
    println!("Connected to MongoDB");
    (collection, vcollection, db , client)
}

pub async fn find_users_by_names(user_collection: Collection<Users> , name: &str, user_id: ObjectId) -> Vec<publicUser> {
    let filter = doc! {"name": name};
    let mut cursor = user_collection.find(filter, None).await.expect("Failed to execute find.");
    let mut users = Vec::new();
    let mut var=0;
    while let Some(result) = cursor.next().await {
        if let Ok(user) = result {
            if user._id != user_id {
                let pub_user = publicUser {
                    refNo: var,
                    _id: user._id,
                    username: user.username,
                    name: user.name,
                    description: user.description,
                    // verified: user.verified,
                    followers: user.followers,
                    following: user.following,
                    voice_notes: user.voice_notes,
                };
                users.push(pub_user);
                var= var+1;
            }
        }
        
    }

    users
}


async fn create_user(user_collection: Collection<Users>, username: String, password: String, name: String) -> ObjectId {
    let user_id = ObjectId::new();
    let new_user = Users {
        _id: user_id,
        username: username.clone(),
        password: password,
        name: name,
        // verified: false,
        description: String::from(""),
        followers: Vec::new(),
        following: Vec::new(),
        voice_notes: Vec::new(),
    };
    
    // Check if a user with the given username exists in the collection
    let filter = doc! { "username": username };
    let result = user_collection.find_one(filter, None).await;
    let user:ObjectId = match result.expect("Error finding user") {
        Some(_) => { 
            println!("User with email already exists");
            ObjectId::parse_str("f0f0f0f0f0f0f0f0f0f0f0f0").unwrap()},
        None => {
            println!("Creating new user");
            new_user.insert_one(user_collection.clone()).await;
            user_id
        }
    };
        
    user
}

pub async fn create_post(voice_collection: Collection<VoiceNote>, user_collection: Collection<Users>, user_id: ObjectId) -> ObjectId {
    let voice_id = ObjectId::new();
    let new_voice_note = VoiceNote {
        _id: voice_id,
        user_id: user_id,
        is_post: true,
        replies: Vec::new(),
        reactions: Vec::new(),
        timestamp: Utc::now()
    };
    new_voice_note.insert_one(voice_collection.clone()).await;
    save_voice_note(user_collection, user_id, voice_id).await;
    voice_id
}

pub async fn create_comment(voice_collection: Collection<VoiceNote>, user_collection: Collection<Users>, user_id: ObjectId, voice_id: String)->ObjectId {
    let comment_id = ObjectId::new();
    let new_voice_note = VoiceNote {
        _id: comment_id,
        user_id: user_id,
        is_post: false,
        replies: Vec::new(),
        reactions: Vec::new(),
        timestamp: Utc::now()
    };
    new_voice_note.insert_one(voice_collection.clone()).await;
    save_voice_note(user_collection, user_id, comment_id).await;
    add_reply(voice_collection, voice_id, comment_id).await;
    comment_id
}

pub async fn add_reply(voice_collection: Collection<VoiceNote>, voice_id: String, comment_id: ObjectId) {
    let filter = doc! {"_id": ObjectId::parse_str(voice_id).unwrap()};

    let update = doc! { "$push": { "replies": comment_id.to_hex()} };

    let options = UpdateOptions::builder().build();

    let result = voice_collection.update_one(filter, update, options).await; 
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

pub async fn follow(user_collection: Collection<Users>, user_id: ObjectId, fuser_id:ObjectId) {
    let filter = doc! {"_id": user_id};

    let update = doc! { "$push": { "following": fuser_id.to_hex()} };

    let options = UpdateOptions::builder().build();

    let result = user_collection.update_one(filter, update, options).await; 

    let filter2 = doc!{"_id": fuser_id};

    let update2= doc! {"$push" : {"followers": user_id.to_hex()}};

    let options2 = UpdateOptions::builder().build();

    let result2 = user_collection.update_one(filter2, update2, options2).await;
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

async fn save_voice_note(collection: Collection<Users> ,userid: ObjectId, v_id: ObjectId) {

    let filter = doc! { "_id": userid };

    let update = doc! { "$push": { "voice_notes": v_id.to_hex()} };

    let options = UpdateOptions::builder().build();

    let result = collection.update_one(filter, update, options).await;
}

async fn get_all_following(user_collection: Collection<Users> , user_id: ObjectId) -> Vec<ObjectId> {
    let filter = doc! {"_id": user_id};
    let mut cursor = user_collection.find(filter, None).await.expect("Failed to execute find.");
    let mut following = Vec::new();
    while let Some(result) = cursor.next().await {
        if let Ok(user) = result {
            for i in user.following {
                following.push(i);
            }
        }        
    }
    following
}



pub async fn make_paths(voice_collection: Collection<VoiceNote>, voice_id: ObjectId) -> String {
    let filter = doc! {"_id": voice_id};
    let result = voice_collection.find_one(filter, None).await;
    let mut user:String = String::new();
    user = match result.expect("Error finding user") {
        Some(result) => { 
            result.user_id.to_hex() + "\\" + &voice_id.to_hex() + ".wav"
        },
        None => {
            println!("No user found");
            String::new()
        }

    };
        
    user    
}

pub async fn get_all_voice_ids_from_following(user_collection:Collection<Users> , voice_collection:Collection<VoiceNote> , user_id:ObjectId) -> Vec<VoiceNote>{
    let following = get_all_following(user_collection.clone(), user_id).await;
        let mut voice_ids = Vec::new();
        for i in following {
            let filter = doc! {"_id": i};
            let mut cursor = user_collection.find(filter, None).await.expect("Failed to execute find.");
            while let Some(result) = cursor.next().await {
                if let Ok(user) = result {
                    for k in user.voice_notes {
                        let filter = doc! {"_id": k};
                        let mut cursor = voice_collection.find(filter, None).await.expect("Failed to execute find.");            
                        while let Some(vresult) = cursor.next().await {
                            if let Ok(voice) = vresult {
                                voice_ids.push(voice);
                            }
                        }
                    }
                }        
            }
        }
    voice_ids
}

pub fn sort_voice_notes_by_timestamp_desc(notes: &mut Vec<VoiceNote>) {
    println ! ("{:?}" , notes[0].timestamp.timestamp());
    println ! ("{:?}" , notes[1].timestamp.timestamp());
    println ! ("{:?}" , notes[2].timestamp.timestamp());
    notes.sort_by(|x, y| y.timestamp.partial_cmp(&x.timestamp).unwrap());
}



fn main() {}
