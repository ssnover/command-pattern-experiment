enum Command {
    ExampleSimpleDataRequest { serial_number: u32, object_type: u8 },
    ExampleComplexDataRequest { payload: Vec<u8> },
}

trait CommandReceiver {
    fn handle_command(&self, event: Command) -> ();
}

pub struct SimpleReceiver {
    data_link: String,
}

impl SimpleReceiver {
    fn handle_simple_data(&self, serial_number: u32, object_type: u8) -> () {
        println!(
            "simple-receiver: Registered new object with Serial Number {}, and Type {}",
            serial_number, object_type
        );
        println!(
            "simple-receiver: Forwarding new object data on link: {}",
            self.data_link
        );
    }
}

impl CommandReceiver for SimpleReceiver {
    fn handle_command(&self, event: Command) {
        match event {
            Command::ExampleSimpleDataRequest {
                serial_number,
                object_type,
            } => self.handle_simple_data(serial_number, object_type),
            _ => (),
        }
    }
}

pub struct ComplexReceiver {
    pub logger_name: String,
}

impl ComplexReceiver {
    fn handle_complex_data(&self, payload: Vec<u8>) -> () {
        println!(
            "complex-receiver: Received payload with length {} bytes, logging to logfile: {}",
            payload.len(),
            self.logger_name
        );
    }
}

impl CommandReceiver for ComplexReceiver {
    fn handle_command(&self, event: Command) {
        match event {
            Command::ExampleComplexDataRequest { payload } => self.handle_complex_data(payload),
            _ => (),
        }
    }
}

fn main() {
    // Initialize a command queue with some example events.
    let mut command_queue = std::collections::VecDeque::new();
    command_queue.push_back(Command::ExampleSimpleDataRequest {
        serial_number: 0xdeadbeef,
        object_type: 0x50,
    });
    command_queue.push_back(Command::ExampleComplexDataRequest {
        payload: vec![0xff, 0xaa, 0xdd, 0xee],
    });
    command_queue.push_back(Command::ExampleSimpleDataRequest {
        serial_number: 0x12345678,
        object_type: 0x10,
    });
    command_queue.push_back(Command::ExampleSimpleDataRequest {
        serial_number: 0xdeadbeef,
        object_type: 0x50,
    });

    // Generate a receiver collection to handle incoming events.
    let receiver_collection: Vec<Box<dyn CommandReceiver>> = vec![
        Box::new(SimpleReceiver {
            data_link: "UART0".to_string(),
        }),
        Box::new(ComplexReceiver {
            logger_name: "/sys/log".to_string(),
        }),
    ];

    // Handle events in the command queue. Would normally be done in another thread asynchronously.
    while command_queue.len() > 0 {
        for receiver in &receiver_collection {
            receiver.handle_command(command_queue.pop_front().unwrap());
        }
    }
}
