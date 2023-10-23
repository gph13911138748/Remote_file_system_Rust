// 创建文件 operation,name,content
// 读取文件 operation,name,offset,number,content
// 非幂等插入 operation,name,offset,sequence
// 幂等计算长度 operation,name
// callback operation,name,offset,number
// operation = 6 时发随机信息,只有operation,name,此时name是直接打印的信息

#[derive(Debug)]
pub struct SerialMessage {
    operation:i32,
    name:String,
    offset:Option<i32>,
    number:Option<i32>,
    sequence:Option<Vec<u8>>,
    content:Option<Vec<u8>>,
    //还可以加一个叫自增编号，客户端发出的每条信息都需要有编号
    //重复发出的消息编号相同
    //两个不同的服务器编号可能相同，但是服务器ip不同
    //ip+编号唯一的区分了一条信息
}

impl Clone for SerialMessage {
    fn clone(&self) -> Self {
        SerialMessage {
            operation: self.operation,
            name: self.name.clone(),
            offset:self.offset.clone(),
            number:self.number.clone(),
            sequence:self.sequence.clone(),
            content:self.content.clone(),
        }
    }
}

//这个里面的所有函数都是与读操作有关
impl SerialMessage {

    //读操作数
    pub fn read_operation(&self)->i32 {
        self.operation
    }

    //读name
    pub fn read_name(&self)->String {
        self.name.clone()
    }

    //读offset
    pub fn read_offset(&self)->i32 {
        self.offset.unwrap()
    }

    //读number
    pub fn read_number(&self)->i32 {
        self.number.unwrap()
    }

    //读sequence
    pub fn read_sequence(&self)->Vec<u8> {
        self.sequence.clone().unwrap()
    }

    //读content
    pub fn read_content(&self)->Vec<u8> {
        self.content.clone().unwrap()
    }
}

//规则
//第一位operation，后面都是两位
//例子：
//1,11,17,18,18,19,19,20,23,24,30,G,A,O,.,t,x,t,3,4,P,E,N,G,C,o,n,t,e,n,t
//解释
//第一位操作数，第二位第三位是name后续的的位置可以读出GAO.txt，第四位到第五位是offset后续的位置
//如果没有的话第四位第五位就写为-1,-1

//这个里面的所有函数都跟加密相关
impl SerialMessage {
    pub fn new(operation:i32, name:String, offset:Option<i32>, 
        number:Option<i32>, sequence:Option<Vec<u8>>, content:Option<Vec<u8>>) -> Self {

        SerialMessage { operation, name, offset, number, sequence, content}
    }

    pub fn serialize_message(&self)->Vec<u8> {
        let mut serial_vec = Vec::new();
        let mut point = 11;
        let operation = self.serialize_operation();
        serial_vec.extend(operation);

        let name = self.serialize_name();
        serial_vec.push(point);
        serial_vec.push(point + name.len() as u8 - 1);
        point += name.len() as u8;

        let offset = self.serialize_offset();
        match offset.as_slice() {
            [0u8,0u8]=>{
                serial_vec.extend(vec![0,0]);
            },
            _=>{
                serial_vec.push(point);
                serial_vec.push(point + offset.len() as u8 - 1);
                point += offset.len() as u8;
            },
        }

        let number = self.serialize_number();
        match number.as_slice() {
            [0u8,0u8]=>{
                serial_vec.extend(vec![0,0]);
            },
            _=>{
                serial_vec.push(point);
                serial_vec.push(point + number.len() as u8 - 1);
                point += number.len() as u8;
            },
        }

        let sequence = self.serialize_sequence();
        match sequence.as_slice() {
            [0u8,0u8]=>{
                serial_vec.extend(vec![0,0]);
            },
            _=>{
                serial_vec.push(point);
                serial_vec.push(point + sequence.len() as u8 - 1);
                point += sequence.len() as u8;
            },
        }

        let content = self.serialize_content();
        match content.as_slice() {
            [0u8,0u8]=>{
                serial_vec.extend(vec![0,0]);
            },
            _=>{
                serial_vec.push(point);
                serial_vec.push(point + content.len() as u8 - 1);
            },
        }

        serial_vec.extend(name);
        if offset.as_slice()!= &[0u8,0u8] {
            serial_vec.extend(offset);
        }
        if number.as_slice()!= &[0u8,0u8] {
            serial_vec.extend(number);
        }
        if sequence.as_slice()!= &[0u8,0u8] {
            serial_vec.extend(sequence);
        }
        if content.as_slice()!= &[0u8,0u8] {
            serial_vec.extend(content);
        }

        serial_vec
    }

    fn serialize_operation(&self)->Vec<u8> {
        let mut serial_vec = Vec::with_capacity(16);
        serial_vec.push(self.operation as u8);
        serial_vec
    }

    fn serialize_name(&self)->Vec<u8> {
        let mut serial_vec = Vec::with_capacity(16);
        serial_vec.extend(self.name.clone().into_bytes());//.into_iter().map(|x| serial_vec.push(x));
        serial_vec
    }

    fn serialize_offset(&self)->Vec<u8> {
        let mut serial_vec = Vec::with_capacity(16);
        match self.offset {
            Some(offset) => serial_vec.push(offset as u8),
            _=>{serial_vec.extend(vec![0,0])},
        }
        serial_vec
    }
    
    fn serialize_number(&self)->Vec<u8> {
        let mut serial_vec = Vec::with_capacity(16);
        match self.number {
            Some(number) => serial_vec.push(number as u8),
            _=>{serial_vec.extend(vec![0,0])},
        }
        serial_vec
    }

    fn serialize_sequence(&self)->Vec<u8> {
        let mut serial_vec = Vec::with_capacity(16);
        match self.sequence.clone() {
            Some(sequence) => serial_vec.extend(sequence),
            _=>{serial_vec.extend(vec![0,0])},
        }
        serial_vec
    }

    fn serialize_content(&self)->Vec<u8> {
        let mut serial_vec = Vec::with_capacity(16);
        match self.content.clone() {
            Some(content) => serial_vec.extend(content),
            _=>{serial_vec.extend(vec![0,0])},
        }
        serial_vec
    }
    // pub fn print_serialization(&self) {
    //     println!("{}", self.serialization);
    // }
}

//这个里面与反序列化相关
impl SerialMessage {
    pub fn deserialize(serial_vec:Vec<u8>)->Self {
        let operation = serial_vec[0] as i32;
        let name = serial_vec[serial_vec[1] as usize..=serial_vec[2] as usize].to_vec();

        let offset = if serial_vec[3]==0 {None} 
        else {Some(serial_vec[serial_vec[3] as usize] as i32)};

        let number = if serial_vec[5]==0 {None} 
        else {Some(serial_vec[serial_vec[5] as usize] as i32)};

        let sequence = if serial_vec[7]==0 {None} 
        else {Some(serial_vec[serial_vec[7] as usize..=serial_vec[8] as usize].to_vec())};

        let content = if serial_vec[9]==0 {None} 
        else {Some(serial_vec[serial_vec[9] as usize..=serial_vec[10] as usize].to_vec())};

        SerialMessage {operation, 
            name: String::from_utf8(name).unwrap(), 
            offset, number, sequence, content,}
    }
}

#[test]
fn test_serialization() {
    let mut v1 = Vec::with_capacity(16);
    v1.push(1);
    let mut v2 = Vec::with_capacity(16);
    v2.push(2);
    v1.extend(v2);
    //println!("{}",v1.len());

    let ser = SerialMessage::new(1,"test.txt".to_string(),
                    None,Some(8),Some(vec![8,10,45,2]),None);
    println!("{:?}",ser.serialize_message());
    let de = SerialMessage::deserialize(ser.serialize_message());
    println!("{:?}",de);
}