pub mod cache;
pub mod file;
pub mod serialize;

use std::{net::UdpSocket, collections:: HashSet, fs};
//use std::str;
use serialize::SerialMessage;
use crate::cache::LRUCache;
use std::io;
use file::TFile;

#[warn(dead_code)]
struct Server {
    address:String,//服务端绑定的地址
    server_cache:LRUCache,//记录缓存中的文件
    //file_name_in_memory:HashMap<String,Vec<String>>,//记录内存中文件和文件的权限的拥有者
    //如果不在内存中，需要在磁盘上查找文件，此时需要权限
    //不在Vec中则没有权限，需要输入密码来获得权限
    //此功能还没有开发完全，所以先注释掉了
    client_address:HashSet<String>,
    socket:UdpSocket,
    //可以新加一个变量维护(信息+ip+客户端单增的编号)
    //对于幂等操作不需要操作，重复发送无所谓
    //对于非幂等的操作只能发送一次，所以根据操作数operation，需要找出需要过滤的操作
    //用ip+客户端单增的编号进行过滤
}

impl Server {

    //创建服务器
    pub fn new(address:String,capacity:i32)->Self {
        Server {
            address:address.clone(),
            server_cache: LRUCache::new(capacity),
            client_address:HashSet::new(),
            socket:UdpSocket::bind(&address).unwrap(),
        }
    }

    //服务器运行
    pub fn run(&mut self) {

    }

    //接收信息 接收的信息包括，反序列化的信息和地址
    pub fn receive_serial_message(&mut self) -> io::Result<(SerialMessage,String)>{
        // buf : UDP发送信息的容量
        let mut buf = [0u8; 1000];

        //amt：UDP具体获得的信息数, src: 客户端地址
        let (amt, src) = self.socket.recv_from(&mut buf)?;

        //把客户端地址写入服务端
        self.client_address.insert(src.to_string());
        //buf 接收的具体长度
        let buf = &mut buf[..amt];
        //self.socket.send_to(buf, &src)?;
        Ok((SerialMessage::deserialize(buf.to_vec()),src.to_string()))
    }

    //解析操作
    fn parse_operation(&mut self,serial_message:SerialMessage,address:String) -> Option<SerialMessage>{
        match serial_message.read_operation() {

            //当操作数=1的时候，是创建文件的操作
            1=>{
                //首先查找是否文件已经存在
                //先检查服务器cache中是否存在
                if self.server_cache.find(serial_message.read_name()) {
                    let reply = SerialMessage::new(6,
                        "message".to_string(),
                        None,
                        None,
                        None,
                        Some("服务器上已经存在该文件".as_bytes().to_vec()));
                    let mut res = self.send_serial_message(reply.clone(), address.clone());
                    while let Err(err) = res {
                        println!("网络阻塞，需要重试{}",err);
                        res = self.send_serial_message(reply.clone(), address.clone());
                    }
                    return None;
                }
                //再检查内存中是否存在
                else if let Ok(metadata) = fs::metadata(serial_message.read_name()) {
                    if metadata.is_file() {
                        let reply = SerialMessage::new(6,
                            "message".to_string(),
                            None,
                            None,
                            None,
                            Some("服务器上已经存在该文件".as_bytes().to_vec()));
                        let mut res = self.send_serial_message(reply.clone(), address.clone());
                        while let Err(err) = res {
                            println!("网络阻塞，需要重试{}",err);
                            res = self.send_serial_message(reply.clone(), address.clone());
                        }
                        return None;
                    }
                }
                else {
                    let file = TFile::new(serial_message.read_name(), serial_message.read_content()).unwrap();
                    self.server_cache.put(serial_message.read_name(), file);
                    println!("{:?}",self.server_cache.map.keys());
                    return Some(SerialMessage::new(6,
                        "message".to_string(),
                        None,
                        None,
                        None,
                        Some("成功创建文件".as_bytes().to_vec())))
                }
                None
            },

            2=>{None},
            3=>{None},
            4=>{None},
            5=>{None},
            6=>{None},
            _=>{None},
        }
    }
    
    //发送信息
    //参数为Serial_message和客户端的地址
    pub fn send_serial_message(&mut self,serial_message:SerialMessage,address:String) ->io::Result<()> {
        let serial_vec = serial_message.serialize_message();
        self.socket.send_to(&mut serial_vec.as_slice(), address)?;
        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    //绑定端口
    //建立服务端对象
    //包含缓存等信息
    let mut server = Server::new("127.0.0.1:8080".to_string(),2);

    loop {
        // // buf : capacity
        // let mut buf = [0u8; 100];

        // //amt：number of bytes, src: address
        // let (amt, src) = server.socket.recv_from(&mut buf)?;

        // //buf is concrete length of recv
        // let buf = &mut buf[..amt];
        // server.socket.send_to(buf, &src)?;
        // let a = SerialMessage::deserialize(buf.to_vec());
        let a = server.receive_serial_message()?;
        let b = a.clone();
        let c = server.parse_operation(b.0, b.1);
        if let Some(d) = c {
            println!("{:?}",d.clone());
            let res = server.send_serial_message(d,a.1.clone());
            if let Err(e) = res {
                println!("{:?}",e);
            }
        } 
        println!("{:?}", String::from_utf8(a.0.clone().read_content()).unwrap());
        //println!("{:?}", server.server_cache.map);
    }
}
