use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Eq, Hash)]
enum CommandId {
    SimpleDataRequest,
    ComplexDataRequest,
    UnimplementedRequest,
    MoreSimpleRequest,
}

enum CommandArgs {
    SimpleDataRequestArgs { serial_number: u32, object_type: u8 },
    ComplexDataRequestArgs { payload: Vec<u8> },
    UnimplementedRequestArgs,
    MoreSimpleRequestArgs,
}

struct Command {
    pub cmd: CommandId,
    pub args: CommandArgs,
}

trait CommandReceiver {
    fn handle_command(&mut self, event: CommandArgs) -> ();
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

    fn handle_more_simple(&mut self) -> () {
        println!("simple-receiver: That was easy!");
    }
}

unsafe impl Send for SimpleReceiver {}
unsafe impl Sync for SimpleReceiver {}

impl CommandReceiver for SimpleReceiver {
    fn handle_command(&mut self, event: CommandArgs) {
        match event {
            CommandArgs::SimpleDataRequestArgs {
                serial_number,
                object_type,
            } => self.handle_simple_data(serial_number, object_type),
            CommandArgs::MoreSimpleRequestArgs => self.handle_more_simple(),
            _ => eprintln!("simple-receiver: Received unknown event, dropping it."),
        }
    }
}

pub struct ComplexReceiver {
    pub logger_name: String,
}

impl ComplexReceiver {
    fn handle_complex_data(&mut self, payload: Vec<u8>) -> () {
        println!(
            "complex-receiver: Received payload with length {} bytes, logging to logfile: {}",
            payload.len(),
            self.logger_name
        );
    }
}

unsafe impl Send for ComplexReceiver {}
unsafe impl Sync for ComplexReceiver {}

impl CommandReceiver for ComplexReceiver {
    fn handle_command(&mut self, event: CommandArgs) {
        match event {
            CommandArgs::ComplexDataRequestArgs { payload } => self.handle_complex_data(payload),
            _ => eprintln!("complex-receiver: Received unknown event, dropping it."),
        }
    }
}

fn main() {
    // Initialize a command queue with some example events.
    let mut command_queue = VecDeque::new();
    command_queue.push_back(Command {
        cmd: CommandId::SimpleDataRequest,
        args: CommandArgs::SimpleDataRequestArgs {
            serial_number: 0xdeadbeef,
            object_type: 0x50,
        },
    });
    command_queue.push_back(Command {
        cmd: CommandId::ComplexDataRequest,
        args: CommandArgs::ComplexDataRequestArgs {
            payload: vec![0xff, 0xaa, 0xdd, 0xee],
        },
    });
    command_queue.push_back(Command {
        cmd: CommandId::SimpleDataRequest,
        args: CommandArgs::SimpleDataRequestArgs {
            serial_number: 0x12345678,
            object_type: 0x10,
        },
    });
    command_queue.push_back(Command {
        cmd: CommandId::SimpleDataRequest,
        args: CommandArgs::SimpleDataRequestArgs {
            serial_number: 0xdeadbeef,
            object_type: 0x50,
        },
    });
    command_queue.push_back(Command {
        cmd: CommandId::UnimplementedRequest,
        args: CommandArgs::UnimplementedRequestArgs,
    });
    command_queue.push_back(Command {
        cmd: CommandId::MoreSimpleRequest,
        args: CommandArgs::MoreSimpleRequestArgs,
    });

    let simple_receiver: Arc<Mutex<dyn CommandReceiver>> = Arc::new(Mutex::new(SimpleReceiver {
        data_link: "UART0".to_string(),
    }));
    let complex_receiver: Arc<Mutex<dyn CommandReceiver>> = Arc::new(Mutex::new(ComplexReceiver {
        logger_name: "/sys/log".to_string(),
    }));

    // Register the receivers for each command they handle
    let mut command_receiver_registry = HashMap::new();
    command_receiver_registry.insert(CommandId::SimpleDataRequest, Arc::clone(&simple_receiver));
    command_receiver_registry.insert(CommandId::ComplexDataRequest, Arc::clone(&complex_receiver));
    command_receiver_registry.insert(CommandId::MoreSimpleRequest, Arc::clone(&simple_receiver));

    // Process the commands
    while command_queue.len() > 0 {
        let next = command_queue.pop_front().unwrap();
        match command_receiver_registry.get(&next.cmd) {
            Some(receiver) => {
                if let Ok(ref mut receiver) = receiver.lock() {
                    receiver.handle_command(next.args);
                } else {
                    eprintln!("command-server: Unable to acquire access to receiver.")
                }
            },
            None => eprintln!("command-server: Received command with no receiver registered, dropping it on the floor."),
        }
    }
}
