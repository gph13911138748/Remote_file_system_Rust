//use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io;

#[derive(Debug)]
pub struct TFile {
    name: String,
    length: usize,
    content: Vec<u8>,
    //access:HashSet<String>,还没有开发完全的功能，设置权限和密码
    //cipher: Vec<u8>,
}

impl Clone for TFile {
    fn clone(&self) -> Self {
        TFile { name:self.name.clone(), 
                length: self.length.clone(), 
                content: self.content.clone(), 
                //access: self.access.clone(),
                //cipher: self.cipher.clone(),
        }
    }
}

impl TFile {
    pub fn new(name:String,content: Vec<u8>) -> io::Result<TFile> {
    //pub fn new(name:String,content: Vec<u8>,access:String,cipher:Vec<u8>) -> io::Result<TFile> {
        let mut file = File::create(name.clone())?;
        file.write_all(&content)?;
        Ok(TFile { name,
                    length: content.len(),
                    content,
                    // access:{let mut hashset = HashSet::new();//还没有开发完全的功能，设置权限和密码
                    //         hashset.insert(access.clone());
                    //         hashset
                    // },
                    // cipher,
                })
    }

    pub fn new_in_client(name:String,content: Vec<u8>) -> io::Result<TFile> {
        //pub fn new(name:String,content: Vec<u8>,access:String,cipher:Vec<u8>) -> io::Result<TFile> {
            Ok(TFile { name,
                        length: content.len(),
                        content,
                        // access:{let mut hashset = HashSet::new();//还没有开发完全的功能，设置权限和密码
                        //         hashset.insert(access.clone());
                        //         hashset
                        // },
                        // cipher,
                    })
        }

    pub fn read_tfile(&self) -> io::Result<String> {
        let mut file = File::open(&self.name)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    }

    //还要相应增加TFile的长度
    pub fn write_tfile(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn test() -> io::Result<()>{
    let my_string = String::from("Hello, Rust!");

    // 将字符串转换为 Vec<u8>
    let bytes = my_string.into_bytes();

    let t = TFile::new("test1".to_string(),bytes)?;
    let s = t.read_tfile()?;
    println!("{:?}",s);
    Ok(())
}