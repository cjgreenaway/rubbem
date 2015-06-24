use byteorder::BigEndian;
use byteorder::WriteBytesExt;
use known_nodes::KnownNode;
use std::io::Cursor;
use message::{Message,ParseError,MAX_NODES_COUNT};
use super::check_no_more_data;

pub struct AddrMessage {
    addr_list: Vec<KnownNode>
}

impl AddrMessage {
    pub fn new(addr_list: Vec<KnownNode>) -> AddrMessage {
        assert!(addr_list.len() <= MAX_NODES_COUNT);
        AddrMessage {
            addr_list: addr_list
        }
    }

    pub fn read(payload: Vec<u8>) -> Result<Box<AddrMessage>,ParseError> {
        let mut cursor = Cursor::new(payload);

        let count = try!(super::read_var_int_usize(&mut cursor, MAX_NODES_COUNT));

        let mut known_nodes: Vec<KnownNode> = Vec::with_capacity(count);
        for _ in 0..count {
            let timestamp = try!(super::read_timestamp(&mut cursor));
            let stream = try!(super::read_u32(&mut cursor));
            let services = try!(super::read_u64(&mut cursor));
            let addr = try!(super::read_address_and_port(&mut cursor));

            if let Ok(known_node) = KnownNode::new(timestamp, stream, services, addr) {
                known_nodes.push(known_node);
            }
        }

        try!(check_no_more_data(&mut cursor));

        Ok(Box::new(AddrMessage::new(known_nodes)))
    }

    pub fn addr_list(&self) -> &Vec<KnownNode> {
        &self.addr_list
    }
}

impl Message for AddrMessage {
    fn command(&self) -> String {
        "addr".to_string()
    }

    fn payload(&self) -> Vec<u8> {
        let mut payload = vec![];
        super::write_var_int_16(&mut payload, self.addr_list.len() as u16);
        for addr in self.addr_list.iter() {
            payload.write_i64::<BigEndian>(addr.last_seen().sec).unwrap();
            payload.write_u32::<BigEndian>(addr.stream()).unwrap();
            payload.write_u64::<BigEndian>(addr.services()).unwrap();
            super::write_address_and_port(&mut payload, &addr.socket_addr());
        }

        payload
    }
}

#[cfg(test)]
mod tests {
    use known_nodes::KnownNode;
    use message::Message;
    use message::addr::AddrMessage;
    use std::io::{Cursor,Read};
    use time::Timespec;

    #[test]
    fn test_addr_message_payload() {
        let node1 = KnownNode::new(Timespec::new(1, 0), 2, 3, "12.13.14.15:1617").unwrap();
        let node2 = KnownNode::new(Timespec::new(4, 0), 5, 6, "22.23.24.25:2627").unwrap();
        let message = AddrMessage::new(vec![node1, node2]);
        let payload = message.payload();

        assert_eq!("addr".to_string(), message.command());

        let expected = vec![
            2,
            0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0, 2,
            0, 0, 0, 0, 0, 0, 0, 3,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, 12, 13, 14, 15,
            6, 81,
            0, 0, 0, 0, 0, 0, 0, 4,
            0, 0, 0, 5,
            0, 0, 0, 0, 0, 0, 0, 6,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, 22, 23, 24, 25,
            10, 67
        ];
        assert_eq!(expected, payload);

        let roundtrip = AddrMessage::read(payload).unwrap();

        assert_eq!("addr".to_string(), roundtrip.command());
        assert_eq!(&vec![node1, node2], roundtrip.addr_list());
    }
}