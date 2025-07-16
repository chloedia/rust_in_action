#! [allow(unused_variables)]
#[derive(Debug)]
enum StatusMessage{
    Ok,
}

fn check_status(sat : CubeSat) -> CubeSat{
    println!("{:?}: {:?}", sat, StatusMessage::Ok);
    sat
}

fn fetch_sat_ids()->Vec<u64>{
    vec![1,2,3]
}

#[derive(Debug)]
struct CubeSat {
    id: u64,
    mailbox: Mailbox,
}

#[derive(Debug)]
struct Mailbox {
    messages: Vec<Message>,
}

#[derive(Debug)]
struct Message {
    to: u64,
    content: String
}
struct GroundStation;

impl GroundStation {
    fn connect(&self, sat_id: u64) -> CubeSat{
        CubeSat {id : sat_id, mailbox: Mailbox {messages: vec![]}}
    }
    fn send(&self, mailbox: &mut Mailbox, msg: Message) {
        mailbox.post(msg);
    }
}

impl CubeSat{
    fn recv(&mut self, mailbox: &mut Mailbox) -> Option<Message> {
        mailbox.deliver(&self)
    }
}

impl Mailbox {
    fn post(&mut self, message: Message){
        self.messages.push(message);
    }

    fn deliver(&mut self, recipient: &CubeSat) -> Option<Message>{
        for i in 0..self.messages.len() {
            if self.messages[i].to == recipient.id {
                let msg = self.messages.remove(i);
                return Some(msg);
            }
        }
        None
    }
}

fn main(){
    let sat_ids = fetch_sat_ids();
    let mut mail = Mailbox { messages: vec![]};
    let base = GroundStation {};

    for sat_id in sat_ids {
        let sat = base.connect(sat_id);
        let msg = Message {to : sat_id, content: String::from("hello you")};
        base.send(&mut mail, msg);
    }
    
    let sat_ids = fetch_sat_ids();
    for sat_id in sat_ids {
        let mut sat = base.connect(sat_id);
        let msg = sat.recv(&mut mail);
        println!("{:?}:{:?}", sat, msg);
    }


}
