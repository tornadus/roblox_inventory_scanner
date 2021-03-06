/*--------------------------------------
|       Roblox Inventory Scanner       |
|     Copyright (C) 2021 tornadus      |
|        Last Update: 2/18/2021        |
--------------------------------------*/


//Imports
use reqwest::Client;
use futures::stream::StreamExt;
use serde::{Deserialize};
use std::collections::HashMap;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, CONTENT_TYPE};
use std::sync::{Arc, Mutex};
use std::fs::{File};
use std::path::Path;
use std::io::prelude::*;
use std::time::{Instant};


//Structs
//Rolimon's API
#[derive(Deserialize)]
#[derive(Clone)]
struct RoliApi {
success: bool,
item_count: u64,
items: HashMap<u64, ItemDetails>,
}

//Rolimon's API Item Details
#[derive(Deserialize)]
#[derive(Clone)]
struct ItemDetails {
name: String,
acro: String,
rap: i64,
value: i64,
default_value: i64,
demand: i64,
trend: i64,
projected: i64,
hyped: i64,
rare: i64,
}

//Roblox Ownership API
#[derive(Deserialize)]
#[derive(Clone)]
struct OwnershipAPI {
_previous_page: Option<String>,
_next_page: Option<String>,
data: Vec<OwnedItem>,
}

//Roblox Ownership API Item Instances
#[derive(Deserialize)]
#[derive(Clone)]
struct OwnedItem {
_a_type: Option<String>,
id: u64,
name: String,
#[serde(rename(deserialize = "instanceId"))]
instance_id: u64,
}


//Roblox User API
#[derive(Deserialize)]
struct UserAPI {
#[serde(rename(deserialize = "isBanned"))]
is_banned: bool,
}


#[derive(Deserialize)]
struct UsernameAPI {
#[serde(rename(deserialize = "Id"))]
id: i64,
}

//Functions

fn construct_headers() -> HeaderMap {
    //Creates the headers needed to obtain the Rolimon's API output
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.141 Safari/537.36 Edg/87.0.664.75"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9"));
    headers
}

fn write_file(data: String, filename: String) {
    //This function writes the output to a file
    let name_file = format!("{}.txt", filename); //Add .txt to filename
    let path = Path::new(&name_file);
    let display = path.display();

    //Create file if it doesn't exist
    let mut file = match File::create(&path) {
        Err(why) => panic!("Couldn't create {}: {}...", display, why),
        Ok(file) => file,
    };

    //Write (or overwrite if file exists)
    match file.write_all(data.as_bytes()) {
        Err(why) => panic!("Couldn't write to {}: {}...", display, why),
        Ok(_) => println!("Successfully wrote list to {}!", display),
    }
}



fn spaces(num: u64) -> String {
    //Return the spaces needed for string formatting later
    let ret: String;
    if num < 10 {
        ret = "            ".to_string();
    } else if num < 100 {
        ret = "           ".to_string();
    } else if num < 1000 {
        ret = "          ".to_string();
    } else if num < 10000 {
        ret = "         ".to_string();
    } else if num < 100000 {
        ret = "        ".to_string();
    } else if num < 1000000 {
        ret = "       ".to_string();
    } else if num < 10000000 {
        ret = "      ".to_string();
    } else if num < 100000000 {
        ret = "     ".to_string();
    } else if num < 1000000000 {
        ret = "    ".to_string();
    } else if num < 10000000000 {
        ret = "   ".to_string();
    } else if num < 100000000000 {
        ret = "  ".to_string();
    } else if num < 1000000000000 {
        ret = " ".to_string();
    } else {
        ret = "".to_string()
    }
    ret
}

async fn get_user_id(ustring: String, client: reqwest::Client) -> i64 {
    //This function returns a Roblox user id from a username (or just spits back out an id if given one)
    //If already a user id, return the id
    match ustring.parse::<i64>() {
        Ok(val) => return val,
        Err(_) => println!("Username detected!"),
      }
    
    //Strip quotes from usernames, this allows the user to enter usernames consisting entirely of numbers
    let uname = ustring.replace(&['"', '\''][..], "");
    let url = format!("https://api.roblox.com/users/get-by-username?username={}", uname);

    //API request
    let req = client.get(&url)
    .headers( construct_headers() )
    .send()
    .await;
    
    match req {
        Ok(res) => {
            match res.json::<UsernameAPI>().await {
                Ok(res) => return res.id,
                Err(_) => panic!("User not found!") //Panic if the "Id" field is not present (User not found)
            }
        }
        Err(_) => panic!("HTTPS Request Failure! Check your connection"), //Panic if the HTTP request as a whole fails
    }

}

async fn normal_scan(ids: Vec<u64>, client: reqwest::Client, uid: i64, connections: usize) -> Vec<OwnershipAPI> {
    //Scans non-terminated users
    let found_ids: Vec<OwnershipAPI> = Vec::new(); //Used below
    let found = Arc::new(Mutex::new(found_ids)); //Gathering owned items from the asynchronous HTTP calls
    let fetches = futures::stream::iter(
        ids.into_iter().map(|ids| {
            let path = format!("https://inventory.roblox.com/v1/users/{}/items/Asset/{}", uid, ids); //Create URL
            let send_fut = client.get(&path).send(); //Send HTTP request
            let cloned_found = found.clone(); //Clone of found needed to store info from within the async move block
            async move {
                match send_fut.await {
                    Ok(resp) => {
                        match resp.text().await {
                            Ok(text) => {
                                let api: OwnershipAPI = serde_json::from_str(&text).unwrap(); //Create instance of OwnershipAPI from JSON
                                if api.data.len() == 0 { //Check if the item is NOT owned
                                } else {
                                    cloned_found.lock().unwrap().push(api); //If owned, push OwnershipAPI instance into vector
                                }
                                
                                }
                            
                            Err(_) => panic!("Failed to decode JSON. Try lowering concurrent connections! Item ID: {} ", ids),
                        }
                    }
                    Err(_) => panic!("Failed to retrieve request. Try lowering concurrent connections! Item ID: {}", ids),
                }
            }
        })
        ).buffer_unordered(connections).collect::<Vec<()>>(); //50 concurrent requests, this number can be raised at risk of HTTP request failure
    fetches.await;

    let item_vector = &*found.lock().unwrap(); //Unwrap to obtain vector
    item_vector.to_vec() //Return vector
}

async fn banned_scan(ids: Vec<u64>, client: reqwest::Client, uid: i64, connections: usize) -> Vec<u64> {
    //Scans terminated users
    let found_ids: Vec<u64> = Vec::new(); //Used below
    let found = Arc::new(Mutex::new(found_ids)); //Gather owned item ids from the async block

    let fetches = futures::stream::iter(
    ids.into_iter().map(|ids| {
        let path = format!("https://api.roblox.com/ownership/hasasset?userId={}&assetId={}", uid, ids);
        let send_fut = client.get(&path).send(); //Send HTTP request
        let cloned_found = found.clone(); //Clone of found needed for async block
        async move {
            match send_fut.await { //Await request
                Ok(resp) => {
                    match resp.text().await { //Get text from request
                        Ok(text) => {
                            if text == "true"{ //If text is "true"...
                                cloned_found.lock().unwrap().push(ids); //... then append the ID to the vector
                            }
                        }
                        Err(_) => println!("[WARNING] Failed to load response. Try lowering concurrent connections! Item ID: {} ", ids),
                    }
                }
                Err(_) => println!("[WARNING] Failed to retrieve request. Try lowering concurrent connections! Item ID: {}", ids),
            }
        }
    })
    ).buffer_unordered(connections).collect::<Vec<()>>();
    fetches.await;

    let item_vector = &*found.lock().unwrap(); //Unwrap to obtain vector
    item_vector.to_vec() //Return vector
}

//Main function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let roclient = Client::builder().build()?; //HTTP client
    println!("Starting tornadus' inventory scanner v0.4");
    println!("Please enter a username or a user id:");
    let ustr: String = text_io::read!(); //Get user input for username/id
    let uid = get_user_id(ustr, roclient).await; //Get user id from the user's input

    println!("Scanning user id {}", uid);
    println!("Please enter amount of concurrent connections (Recommended 50, choose lower if you experience errors):");
    let connections: usize = text_io::read!(); //Get connection amount
        let client = Client::builder().build()?; //HTTP client
    
    //Rolimon's API request
    let req = client.get("https://www.rolimons.com/itemapi/itemdetails")
        .headers( construct_headers() )
        .send()
        .await?;



    let roli_api: RoliApi = req.json().await?; //Imports the Rolimon's API json as a RoliApi instance
    let mut ids: Vec<u64> = Vec::new(); //Holds item ids gathered from the Rolimon's API. Used for creating the HTTP request URLs.

    let roli_items = roli_api.items.clone(); //Used later on when we need a copy of the Rolimon's item DB

    //Iterate through items and push ids to vector
    for (id, _value) in roli_api.items {
        ids.push(id);
    }
    
    //User API request
    let user_url = format!("https://users.roblox.com/v1/users/{}", uid);
    let user_check = client.get(&user_url)
        .headers( construct_headers() )
        .send()
        .await?;

    let user_text = user_check.text().await?;
    let user_api: UserAPI = serde_json::from_str(&user_text).unwrap(); //User API instance used for ban check

    if user_api.is_banned == false { //If the user is not banned
        let start = Instant::now();
        let item_vector = normal_scan(ids, client, uid, connections).await;
        let duration = start.elapsed();
        println!("Scan took {:?}", duration);
        println!("");
        let mut item_str = String::from("UAID         ||| Name"); //Empty string used to print the list of items in one go
        let mut item_count = 0; //Total item count
        let mut total_value = 0; //Total value
        let mut total_rap = 0; //Total RAP
    
        for api in item_vector { //Iterate through OwnershipAPI objects
            for item in &api.data { //Iterate through item instance objects
                let mut value = roli_items[&item.id].value; //Mutable variable for value
                let rap = roli_items[&item.id].rap; //Item RAP
                if value == -1 {
                    value = rap //If value is -1, set value to RAP (Just like on the Rolimon's website)
                }
                total_value = total_value + value; //Add item value to total
                total_rap = total_rap + rap; //Add item RAP to total
                item_count = item_count + 1; //Add item count to total
                let addstr = format!("\n{}{}||| {}", item.instance_id, spaces(item.instance_id), item.name); //Create string to add to item_str
                item_str.push_str(&addstr) //Adds addstr to item_str
            }
        }
        //String formatting
        item_str.push_str("\n");
        let count_str = format!("{} item(s) found.", item_count);
        item_str.push_str(&count_str);
        let total_str = format!("\nTotal value is R${}\nTotal RAP is R${}\n", total_value, total_rap);
        item_str.push_str(&total_str);
        print!("{}", item_str); //Prints the item list

        //Ask about saving output to file
        println!("Save output to file?");
        let answer = loop{
            let ask: String = text_io::read!();
            if ask == "yes" || ask == "y" {
                break true;
            } else if ask == "no" || ask == "n" {
                break false;
            } else {
                println!("Please enter yes or no!")
            }

        };

        //Save output to file, exit if answer is no
        if answer == true {
            println!("Attempting save to file...");
            let filename = format!("{}_scan", uid);
            write_file(item_str, filename);
        } else {
            println!();
        }
    
        dont_disappear::any_key_to_continue::default();


    } else if user_api.is_banned == true { //If the user is banned
        println!("");
        let mut item_str = String::from("ID           ||| Name"); //Empty string used to print the list of items in one go
        let mut item_count = 0; //Total item count
        let mut total_value = 0; //Total value
        let mut total_rap = 0; //Total RAP
        let start = Instant::now();
        let item_vector = banned_scan(ids, client, uid, connections).await;
        let duration = start.elapsed();
        println!("Scan took {:?}", duration);

        for item in item_vector {
            let mut value = roli_items[&item].value; //Mutable variable for value
            let rap = roli_items[&item].rap; //Item RAP
            if value == -1 {
                value = rap //If value is -1, set value to RAP (Just like on the Rolimon's website)
            }
            total_value = total_value + value; //Add item value to total
            total_rap = total_rap + rap; //Add item RAP to total
            item_count = item_count + 1; //Add item count to total
            let addstr = format!("\n{}{}||| {}", item, spaces(item), roli_items[&item].name); //Create string to add to item_str
            item_str.push_str(&addstr) //Adds addstr to item_str
        }

        //String formatting
        item_str.push_str("\n");
        let count_str = format!("{} item(s) found.", item_count);
        item_str.push_str(&count_str);
        let total_str = format!("\nTotal value is R${}\nTotal RAP is R${}\n", total_value, total_rap);
        item_str.push_str(&total_str);
        item_str.push_str("Due to Roblox API limitations, this data does not include multiple copies of owned items.\n");
        print!("{}", item_str); //Prints the item list
        
        //Ask about saving output to file
        println!("Save output to file?");
        let answer = loop{
            let ask: String = text_io::read!();
            if ask == "yes" || ask == "y" {
                break true;
            } else if ask == "no" || ask == "n" {
                break false;
            } else {
                println!("Please enter yes or no!")
            }

        };

        //Save output to file, exit if answer is no
        if answer == true {
            println!("Attempting save to file...");
            let filename = format!("{}_scan", uid);
            write_file(item_str, filename);
        } else {
            println!();
        }
        dont_disappear::any_key_to_continue::default();

    } else {
        println!("Unknown error, try again!");
        dont_disappear::any_key_to_continue::default();
    }

    Ok(())
}