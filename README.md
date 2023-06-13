# Built as university project for CSE 4xx Rust Programming.
Project members: 
Muhammad Abdur Rafae (22828)
Saifullah Khan (22877)
Hassan Yahya (22965)

Project Idea: A twitter-like social media application built purely on rust libraries, with the change of tweets/quotes being recorded instead of being typed. 

Database used: MongoDB
Collections: Users and VoiceNotes

The project is a working social media app with simple and straight-forward UI built on egui and eframe. Users will be able to perform all of the following tasks:
1) Sign Up / Login
2) Record / Delete their Quotes
3) Follow / Unfollow people using their unique usernames
4) Listen to Quotes recorded by people they follow
5) React to Quote (Shut Up / Speak Up)
6) Reply to a Quote and react to other replies on a post

Quotes when recorded get converted into a vector which is then uploaded. Voice Notes are downloaded on runtime, and deleted as user logout. 
Utlised tokio's library to perform asynchronous tasks within closures, by creating new Runtime that blocks current execution until its code is fully executed.
Automatic directory cleaning: All downloaded quotes in a user session gets deleted when user logout.

Utilised Structs and vectors of those Structs to post and fetch data from mongoDB. 
Created Enums to restrict options in scenarios like reactions to a tweet and switching between pages on frontend.
Managed Error Handling using Rust's enums: Option and Result.
