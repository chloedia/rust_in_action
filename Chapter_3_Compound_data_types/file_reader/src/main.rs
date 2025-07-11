//! Chapter 3, simulating IO file one step at a time
#![allow(unused_variables)]
use rand::prelude::*;
use std::fmt;
use std::fmt::{Display};

#[derive(Debug, PartialEq)]
pub enum FileState{
    Open,
    Closed,
}

fn one_in(denominator: u32) -> bool {
   rand::thread_rng().gen_ratio(1, denominator)
}

/// Represents a File that can live in any file system
#[derive(Debug)]
pub struct File {
    pub name : String,
    data: Vec<u8>,
    pub state: FileState
}

impl Display for FileState{
    fn fmt(&self, f: &mut fmt::Formatter,) -> fmt::Result{
        match *self {
            FileState::Open => write!(f, "OPEN"),
            FileState::Closed => write!(f, "CLOSED"),
        }
    }
}

impl Display for File{
    fn fmt(&self, f: &mut fmt::Formatter,) -> fmt::Result{
        write!(f, "<{} ({})>", self.name, self.state)
    }
}
impl File {
    /// Creates a new, empty `File`.
    ///
    /// # Examples
    ///
    /// ```
    /// let f = File::new("f1.txt");
    /// ```
    pub fn new(name: &str) -> File {
        File{
        name: String::from(name),
        data: Vec::new(),
        state: FileState::Closed}
    }
    ///Return the len of the document
    pub fn len(&self)->usize{
        self.data.len()
    }
    
    ///Return the name of the document
    pub fn name(&self)->String{
        self.name.clone()
    }
    fn new_with_data(name: &str, data: &Vec<u8>)->File{
        let mut file = File::new(name);
        file.data = data.clone();
        file
    }

    fn read(self: &File, save_to: &mut Vec<u8>)->Result<usize, String>{
        let mut tmp = self.data.clone();
        let read_length = tmp.len();

        save_to.reserve(read_length);
        save_to.append(&mut tmp);
        
        Ok(read_length)
    }
}
fn open(mut f: File)-> Result<File, String>{
    if one_in(10000){
        let error_message = String::from("Permission Denied");
        return Err(error_message);
    }
    f.state = FileState::Open;
    Ok(f)
}

fn close(mut f: File)-> Result<File, String>{
    if one_in(10_000){
        let error_message = String::from("An error occured");
        return Err(error_message);
    }
    f.state = FileState::Closed;
    Ok(f)
}

fn main() {
    let f1_data: Vec<u8> = vec![114, 117, 115, 116, 33];
    let mut f1 = File::new_with_data("file3.txt", &f1_data);
    let mut buffer: Vec<u8> = vec![];

    //if f1.read(&mut buffer).is_err(){
    //    println!("Error Checker is Working");
    //}
    f1 = open(f1).unwrap();
    let f1_length = f1.read(&mut buffer).unwrap();
    f1 = close(f1).unwrap();
    let text = String::from_utf8_lossy(&buffer);
    println!("{:?}", f1);
    println!("{}",f1);
    println!("{} is {} bytes long", &f1.name(), &f1.len());
    println!("{}", text);
}
