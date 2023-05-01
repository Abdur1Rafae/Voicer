use mongodb::{Client, Collection  , Database,options::{ClientOptions, GridFsBucketOptions},};
use mongodb_gridfs::{options::GridFSBucketOptions, GridFSBucket,  GridFSError};
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
    pub async fn insert_one(&self, collection: Collection<Users>) {
        let new_user = self.clone();
        collection.insert_one(new_user, None).await.unwrap();
    }
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

#[derive(Serialize, Deserialize, Debug)]
pub struct VoiceNote{
    v_id:bson::oid::ObjectId,
    user_id:ObjectId,
    is_post:bool,
    replies:Vec<ObjectId>,
    reactions:Vec<Reaction>,
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

async fn create_user(user_collection: Collection<Users>, username: String, password: String, name: String) -> ObjectId {
    let mut user_id = ObjectId::new();
    let new_user = Users {
        _id: user_id,
        username: username,
        password: password,
        name: name,
        description: String::from(""),
        followers: Vec::new(),
        following: Vec::new(),
        voice_notes: Vec::new(),
    };
    new_user.insert_one(user_collection.clone()).await;
    user_id
}

pub async fn create_post(voice_collection: Collection<VoiceNote>, user_collection: Collection<Users>, user_id: ObjectId) -> ObjectId {
    let mut voice_id = ObjectId::new();
    let new_voice_note = VoiceNote {
        v_id: voice_id,
        user_id: user_id,
        is_post: true,
        replies: Vec::new(),
        reactions: Vec::new(),
    };
    new_voice_note.insert_one(voice_collection.clone()).await;
    save_voice_note(user_collection, user_id, voice_id).await;
    voice_id
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

async fn save_voice_note(collection: Collection<Users> ,userid: ObjectId, v_id: ObjectId) {
    // Create a filter to match the user with the given ID
    let filter = doc! { "_id": userid };

    // Create an update document to append the voice note ID to the `voice_notes` array
    let update = doc! { "$push": { "voice_notes": v_id.clone().to_hex()} };

    // Create an UpdateOptions instance with default options
    let options = UpdateOptions::builder().build();

    // Call the update_one method on the collection with the filter, update, and options
    let result = collection.update_one(filter, update, options).await;

    println!("Audio file saved in MongoDB using GridFS!");
}

// pub async fn save_voice_note(userid: ObjectId) -> PyResult<()>{

//     Python::with_gil(|py| {
//         let fun: Py<PyAny> = PyModule::from_code(
//             py,
//             "import pymongo\nimport gridfs\n\
//              def save_sound(user_id):
//                 client = pymongo.MongoClient(f'mongodb+srv://RustUser:RUSTIBA@cluster0.btmwmdh.mongodb.net/test')

//                 # Access a database
//                 db = client['Cluster0']
//                 fs = gridfs.GridFS(db, collection='fs')
//                 with open('hello.wav', 'rb') as f:
//                     contents = f.read()
            
//                 # Create a new GridFS file and write the contents to it
//                 grid_in = fs.new_file(filename='new.wav')
//                 grid_in.write(contents)
//                 grid_in.close()
            
//                 # Get the _id of the newly uploaded file
//                 file_id = grid_in._id
//                 users_collection = db['users']
//                 users_collection.update_one(
//                 {'_id': user_id},
//                 {'$push': {'voice_notes': {'id': file_id}}}
//             )
            
//                 print('Voicenote ID added to user document successfully.')
//         ",        


//             "",
//             "",
//         )
//         .expect("function should be called")
//         .getattr("save_sound")?
//         .into();
    
//         // call object without any arguments
//         let args = PyTuple::new(py, &[userid.to_string()]);

//         fun.call1(py , args);
//         Ok(())
//     })
 
// }


// pub async fn play_audio(v_id : &str) -> PyResult<()> {
//     Python::with_gil(|py| {
//         let fun: Py<PyAny> = PyModule::from_code(
//             py,
//             "import pymongo\nimport gridfs\nimport playsound\n\
//             def download_audio_file( file_id):
            
//                 client = pymongo.MongoClient(f'mongodb+srv://RustUser:RUSTIBA@cluster0.btmwmdh.mongodb.net/test')
            
//                 # Access a database
//                 db = client['Cluster0']
//                 fs = gridfs.GridFS(db, collection='fs')
//                 # Find the audio file in GridFS
//                 grid_out = fs.find_one({'_id': file_id})
            
//                 # Read the audio file data into a variable
//                 audio_data = grid_out.read()
            
//                 # Write the audio file data to a local file with a .wav extension
//                 with open('downloaded.wav', 'wb') as f:
//                     f.write(audio_data)
//                 playsound('downloaded.wav')
        
//         ",        


//             "",
//             "",
//         )
//         .expect("function should be called")
//         .getattr("download_audio_file")?
//         .into();
    
//         // call object without any arguments
//         let args = PyTuple::new(py, &[v_id.to_string()]);

//         fun.call1(py , args);

//         Ok(())
//     })
     
// }

fn main() {}
