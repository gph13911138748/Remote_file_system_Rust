pub mod cache;
pub mod file;
pub mod serialize;

use std::net::UdpSocket;
use std::io;
use serialize::SerialMessage;
use file::TFile;
use cache::LRUCache;

pub struct Client {
    client_addr: String,
    server_addr: String,
    client_cache:LRUCache,
    socket:UdpSocket,
}

impl Client {
    
    //创建Client实例
    pub fn new(client_addr: String, server_addr: String, capacity:i32) -> Self {
        Client {
            client_addr: client_addr.clone(),
            server_addr: server_addr,
            client_cache: LRUCache::new(capacity),
            socket: UdpSocket::bind(&client_addr).unwrap(),
        }
    }

    //把客户端和服务端连接
    pub fn connect_server(&self) -> io::Result<()> {
        self.socket.connect(&self.server_addr)?;
        Ok(())
    }

    pub fn receive_serial_message(&mut self) -> io::Result<(SerialMessage,String)>{
        // buf : UDP发送信息的容量
        let mut buf = [0u8; 1000];

        //amt：UDP具体获得的信息数, src: 客户端地址
        let (amt, src) = self.socket.recv_from(&mut buf)?;

        //buf 接收的具体长度
        let buf = &mut buf[..amt];
        //self.socket.send_to(buf, &src)?;
        Ok((SerialMessage::deserialize(buf.to_vec()),src.to_string()))
    }

    pub fn send_serial_message(&mut self,serial_message:SerialMessage,address:String) ->io::Result<()> {
        let serial_vec = serial_message.serialize_message();
        self.socket.send_to(&mut serial_vec.as_slice(), &address)?;
        Ok(())
    }

    //远程创建文件
    pub fn remote_create_file(&mut self, name: String,content:Option<String>) -> io::Result<String> {
        //先从自己的缓存上寻找是否创建过文件
        if self.client_cache.find(name.clone()) {
            return Ok("已经创建过文件".to_string());
        }

        //没有则寻求远程创建
        //创建编码并发送
        let send_serial_message = SerialMessage::new(1, 
                                                        name.clone(), 
                                                        None, 
                                                        None, 
                                                        None, 
                                                        if content.is_none() {Some("".as_bytes().to_vec())} 
                                                        else {Some(content.unwrap().as_bytes().to_vec())},);
                                                        //{Option("".to_string().to_vec::<u8>())} if content.is_none() else {Option(content.to_vec::<u8>())});
        let mut res = self.send_serial_message(send_serial_message.clone(), self.server_addr.clone());
        while let Err(err) = res {
            println!("网络阻塞，需要重试{}",err);
            res = self.send_serial_message(send_serial_message.clone(), self.server_addr.clone());
        }
        // if let Err(e) = res {
        //     return Err(e);
        // }
        
        //接收信息
        let serial_message = self.receive_serial_message()?;
        if &String::from_utf8(serial_message.0.clone().read_content()).unwrap() == "服务器上已经存在该文件" {
            return Ok("已经创建过文件".to_string());
        }

        //如果之前没有创建过，就需要在客户端缓存中创建
        //该操作会把之前最久没使用过的文件挤掉
        let file = TFile::new_in_client(send_serial_message.read_name(), send_serial_message.read_content()).unwrap();
        self.client_cache.put(send_serial_message.read_name(), file);    
        Ok("已经成功创建文件".to_string())
    }

}

fn main() -> std::io::Result<()> {
    let mut client1 = Client::new("127.0.0.1:8081".to_string(),
                                          "127.0.0.1:8080".to_string(),
                                          2);
    //client1.connect_server()?;
    //client1.connect_server()?;

    let res = client1.remote_create_file("gph.txt".to_string(), Some("gphlzy".to_string()))?;
    println!("{}",res);
    let res = client1.remote_create_file("gp.txt".to_string(), Some("gphlzy".to_string()))?;
    println!("{}",res);
    let res = client1.remote_create_file("g.txt".to_string(), Some("gphlzy".to_string()))?;
    println!("{}",res);
    println!("{:?}",client1.client_cache.map.keys());
    
    loop {
        let mut input = String::new();
        //从键盘输入要发送的信息
        io::stdin().read_line(&mut input).expect("Failed to read");
        
        //发送消息
        //socket.send(input.as_bytes()).expect("Failed to send");
        //let ser1 = SerialMessage::new(1,"test.txt".to_string(),Some(20),Some(8),Some(vec![8,10,45,2]),Some("123".to_string().into_bytes()));
        //client1.socket.send(ser1.serialize_message().as_slice())?;
        let res = client1.remote_create_file("gph.txt".to_string(), Some("gphlzy".to_string()))?;
        println!("{}",res);
        let res = client1.remote_create_file("gp.txt".to_string(), Some("gphlzy".to_string()))?;
        println!("{}",res);
        let res = client1.remote_create_file("g.txt".to_string(), Some("gphlzy".to_string()))?;
        println!("{}",res);
        //接受消息
        // let mut buf = [0u8;1000];
        // let (amt, _) = client1.socket.recv_from(&mut buf).unwrap();
        // let buf = &mut buf[..amt];
        // println!("{:?}",String::from_utf8(SerialMessage::deserialize(buf.to_vec()).read_content()));
    }
}
