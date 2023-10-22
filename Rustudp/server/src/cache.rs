//假设缓存的大小是有限的，利用LRU算法来保证缓存访问的质量
use crate::file::{self, TFile};

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

//LRU缓存由哈希表和链表构成
//哈希表负责访问为O(1)
//链表负责访问后的调整顺序和普通的加入操作是O(1)
//在服务端没有callout的时候不需要更新数据
#[derive(Debug)]
pub struct LRUCache {
    capacity:i32,
    pub map:HashMap<String,Rc<RefCell<ListNode>>>,
    first:Option<Rc<RefCell<ListNode>>>,
    last:Option<Rc<RefCell<ListNode>>>,
}

//链表节点由key=String,value=TFile构成
//TFile包含了名字，长度和储存的信息，是客户端的缓存，便于客户端重复访问数据
#[derive(Debug)]
pub struct ListNode {
    key:String,
    value:file::TFile,
    next:Option<Rc<RefCell<ListNode>>>,
    prev:Option<Rc<RefCell<ListNode>>>,
}

impl ListNode {

    fn new(key:String,value:TFile) -> Self{
        ListNode {
            key,
            value,
            next:None,
            prev:None,
        }
    }
}


//相关的操作的实现
//其中new,get,put是可以被外界访问的API，删除等操作为了安全是禁止外界访问
impl LRUCache {

    //创建缓存区操作
    pub fn new(capacity: i32) -> Self {
        LRUCache {
            capacity,
            map:HashMap::new(),
            first:None,
            last:None,
        }
    }
    
    //寻找缓存区是否有文件，有的话进行访问
    pub fn get(&mut self, key: String) -> Result<TFile,&'static str> {
        if let Some(node) = self.map.get(&key) {
            let node = node.clone();
            let value = node.borrow().value.clone();
            self.remove(&node);
            self.offer_last(&node);
            Ok(value)
        }else {
            Err("Don't find in cache")
        }
    }
    
    //查找文件是否在缓存区存在
    pub fn find(&self,name:String) -> bool {
        if self.map.contains_key(&name) {
            true
        }else {
            false
        }
    }

    //向缓存区添加数据，如果满了则移除最久没访问的文件
    //同时也可以更新数据，如果已经存在该文件，则直接更新
    pub fn put(&mut self, key: String, value: TFile) {
        let node = Rc::new(RefCell::new(ListNode::new(key.clone(), value)));
        if let Some(node) = self.map.get(&key) {
            let node = node.clone();
            self.remove(&node);
        }else {
            if self.map.len() == self.capacity as usize {
                self.poll_first();
            }
        }
        self.offer_last(&node);
    }

    //移除文件操作
    //访问不是最新的文件，需要从链表中间删除并加入到链表末尾（最新访问区）
    fn remove(&mut self,node:&Rc<RefCell<ListNode>>) {
        match (node.clone().borrow().prev.as_ref(), node.clone().borrow().next.as_ref()) {
            (Some(pnode),Some(nnode))=> {
                pnode.borrow_mut().next = node.clone().borrow().next.clone();
                nnode.borrow_mut().prev = node.clone().borrow().prev.clone();
            },
            (Some(pnode),None)=> {
                pnode.borrow_mut().next = None;
                self.last = node.clone().borrow().prev.clone();
            },
            (None,Some(nnode))=> {
                nnode.borrow_mut().prev = None;
                self.first = node.clone().borrow().next.clone();
            },
            (None,None)=> {
                self.first = None;
                self.last = None;
            }
        }
        let key = node.borrow().key.clone();
        self.map.remove(&key);
    }

    //添加文件到链表的末尾（最新访问区）
    fn offer_last(&mut self,node:&Rc<RefCell<ListNode>>) {
        let node = node.clone();
        node.borrow_mut().next = None;
        node.borrow_mut().prev = None;
        if self.first.is_none() {
            self.first = Some(node.clone());
            self.last = Some(node.clone());
        }else {
            if let Some(ref n) = self.last {
                node.borrow_mut().prev = Some(n.clone());
                n.borrow_mut().next = Some(node.clone());
                self.last = Some(node.clone());
            }
        }
        let key = node.borrow().key.clone();
        self.map.insert(key,node.clone());
    }

    //移除链表的第一个元素（满了就需要移除）
    fn poll_first(&mut self) {
        if let Some(node) = self.first.take() {
            let key = node.borrow().key.clone();
            self.map.remove(&key);
            match node.borrow().next.as_ref() {
                Some(next)=>{
                    next.borrow_mut().prev = None;
                    self.first = Some(next.clone());
                },
                None=> {
                    self.first = None;
                    self.last = None;
                },
            }
        }
    }
}


//测试区代码
#[test]
fn test() -> std::io::Result<()> {
    let mut lru = LRUCache::new(2);
    let my_string = String::from("Hello, Rust!");
    let my_string2 = String::from("hello,rust!");
    let my_string3 = String::from("lzy");
    // 将字符串转换为 Vec<u8>
    let bytes = my_string.into_bytes();
    let bytes2 = my_string2.into_bytes();
    let bytes3 = my_string3.into_bytes();

    let t = TFile::new("test1.txt".to_string(),bytes)?;
    let t2 = TFile::new("test2.txt".to_string(),bytes2)?;
    let t3 = TFile::new("test3.txt".to_string(),bytes3)?;

    lru.put("1".to_string(),t);

    //理论上来讲，此时cache中不能含有2
    let b = lru.get("2".to_string());
    lru.put("2".to_string(),t2);
    let a = lru.get("1".to_string()).unwrap();

    //由于只有2个容量，所以cache保存的是最近访问的1和3，其中还是没有2，所以访问2还是找不到
    lru.put("3".to_string(),t3);
    let c = lru.get("2".to_string());
    println!("{:?}",a.read_tfile());
    println!("{:?}",b);
    println!("{:?}",c);
    Ok(())
}