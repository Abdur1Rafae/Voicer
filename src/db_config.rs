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
    #[serde(rename = "ReactionType")]
    reaction: ReactionType
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VoiceNote {
    pub _id: ObjectId,
    pub user_id: ObjectId,
    pub is_post: bool,
    pub data: Vec<i16>,
    pub replies: Vec<ObjectId>,
    pub reactions: Vec<Reaction>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
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


// impl Ord for VoiceNote {
//     fn cmp(&self, other: &Self) -> Ordering {
//         self.timestamp.cmp(&other.timestamp)
//     }
// }

pub struct replies{
    pub _id: ObjectId,
    pub user_id: ObjectId,
}

pub struct conversation{
    pub v_id: ObjectId,
    pub v_user_id: ObjectId,
    pub reactions: Vec<Reaction>,
    pub replies: Vec<replies>
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


pub async fn create_user(user_collection: Collection<Users>, username: String, password: String, name: String) -> ObjectId {
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


pub async fn react_to_quote(voice_collection: Collection<VoiceNote>, v_id: ObjectId, user_id: ObjectId, reaction: ReactionType) {
    let filter = doc!{"_id": v_id};

    let user_reaction = Reaction{
        user_id: user_id,
        reaction: reaction,
    };

    let reaction_doc = bson::to_document(&user_reaction)
    .expect("Failed to serialize reaction");

    let update = doc! { "$push": { "reactions": reaction_doc} };

    let options = UpdateOptions::builder().build();

    let result = voice_collection.update_one(filter, update, options).await; 
}

pub async fn create_post(voice_collection: Collection<VoiceNote>, user_collection: Collection<Users>, user_id: ObjectId, data: Vec<i16>, voice_id: ObjectId) {
    let new_voice_note = VoiceNote {
        _id: voice_id,
        user_id: user_id,
        is_post: true,
        data: data,
        replies: Vec::new(),
        reactions: Vec::new(),
        timestamp: Utc::now()
    };
    new_voice_note.insert_one(voice_collection.clone()).await;
    save_voice_note(user_collection, user_id, voice_id).await;
}

pub async fn convert_audio_to_vec(filename: &str) -> Vec<i16> {
    let mut reader = hound::WavReader::open(filename).unwrap();
    
    let samples: Vec<i16> = reader.samples::<f32>().map(|x| (x.unwrap() * i16::MAX as f32) as i16).collect();

    samples
}

pub async fn convert_vec_to_audio(filename:&str, data: Vec<i16>) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(filename, spec).unwrap();

    for sample in data {
        writer.write_sample(sample).unwrap();
    }

    writer.finalize().unwrap();
}

pub async fn create_comment(voice_collection: Collection<VoiceNote>, user_collection: Collection<Users>, user_id: ObjectId, voice_id: String, comment_id: ObjectId, data: Vec<i16>) {
    let new_voice_note = VoiceNote {
        _id: comment_id,
        user_id: user_id,
        is_post: false,
        data: data,
        replies: Vec::new(),
        reactions: Vec::new(),
        timestamp: Utc::now()
    };
    new_voice_note.insert_one(voice_collection.clone()).await;
    save_voice_note(user_collection, user_id, comment_id).await;
    add_reply(voice_collection, voice_id, comment_id).await;
}

pub async fn add_reply(voice_collection: Collection<VoiceNote>, voice_id: String, comment_id: ObjectId) {
    let filter = doc! {"_id": ObjectId::parse_str(voice_id).unwrap()};

    let update = doc! { "$push": { "replies": comment_id.to_hex()} };

    let options = UpdateOptions::builder().build();

    let result = voice_collection.update_one(filter, update, options).await; 
}

pub async fn create_conversation (voice_collection: Collection<VoiceNote>, v_id: ObjectId,) -> conversation {
    let filter = doc! { "_id": v_id };

    let mut post= VoiceNote { _id: ObjectId::new(), user_id: ObjectId::new(), is_post: false, replies: vec![],data: vec![], reactions: vec![], timestamp: Utc::now() };

    match voice_collection.find_one(filter, None).await {
        Ok(result) => match result {
            Some(doc) => {
                    post=doc;
                }
            None => {}
        },
        Err(e) => {
            println!("Failed to get user: {}", e);
        }
    };

    let mut con_replies = Vec::new();

    for item in post.replies{
        let reply = replies {
            _id: item,
            user_id: get_user_of_vn(voice_collection.clone(), item).await.unwrap(),
        };
        con_replies.push(reply);
    };

    let result = conversation {
        v_id: post._id,
        v_user_id: post.user_id,
        reactions: post.reactions,
        replies: con_replies, 
    };

    result

}

async fn get_user_of_vn(voice_collection: Collection<VoiceNote>, v_id: ObjectId) -> Option<ObjectId> {
    let filter = doc! {"_id": v_id};

    let mut user:Option<ObjectId>;

    match voice_collection.find_one(filter, None).await {
        Ok(result) => match result{
            Some(doc)=>{
                user = Some(doc.user_id);
            }
            None => user=None,
        }
        Err(e) => {
            println!("Failed to get user: {}", e);
            user = None
        }
    }

    user
}

pub async fn get_user_by_username(collection: Collection<Users>, username: String, password: String) -> Option<Users> {
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


pub async fn get_all_voice_ids_from_following(user_collection:Collection<Users> , voice_collection:Collection<VoiceNote> , user_id:ObjectId) -> Vec<VoiceNote>{
    let following = get_all_following(user_collection.clone(), user_id).await;
    println!("you follow {:?}", following);
        let mut voice_ids = Vec::new();
        for i in following {
            let filter = doc! {"_id": i};
            let mut cursor = user_collection.find(filter, None).await.expect("Failed to execute find.");
            while let Some(result) = cursor.next().await {
                if let Ok(user) = result {
                    for k in user.voice_notes {
                        let filter = doc! {"_id": k};
                        let mut cursor = voice_collection.find(filter, None).await.expect("Failed to execute find.");            
                        while let Some(result) = cursor.next().await {
                            if let Ok(voice) = result {
                                voice_ids.push(voice);
                            }
                        }
                    }
                }        
            }
        }
    voice_ids
}

pub fn sort_voice_notes_by_timestamp_desc(notes : &mut Vec<VoiceNote>) {
    
    notes.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // notes.sort();
    // notes.reverse();
    // println!("{:?}",notes);
}

pub async fn download_voice_notes(voice_collection : Collection<VoiceNote> , v_id : ObjectId){
    let filter = doc! {"_id": v_id};
    let result = voice_collection.find_one(filter, None).await;
    let mut voice:Vec<i16> = Vec::new();
    voice = match result.expect("Error finding voice") {
        Some(result) => { 
            result.data
        },
        None => {
            println!("No voice data found");
            Vec::new()
    
        }
    };

    convert_vec_to_audio("downloaded.wav" , voice).await;   
}


fn main() {}
