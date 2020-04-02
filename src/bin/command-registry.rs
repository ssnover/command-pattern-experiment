use std::collections::{HashMap, VecDeque};

#[derive(PartialEq, Eq, Hash)]
enum CommandId {
    SimpleDataRequest,
    ComplexDataRequest,
    UnimplementedRequest,
}

enum CommandArgs {
    SimpleDataRequestArgs { serial_number: u32, object_type: u8 },
    ComplexDataRequestArgs { payload: Vec<u8> },
    UnimplementedRequestArgs,
}

struct Command {
    pub cmd: CommandId,
    pub args: CommandArgs,
}

trait CommandReceiver {
    fn handle_command(&self, event: CommandArgs) -> ();
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
    fn handle_command(&self, event: CommandArgs) {
        match event {
            CommandArgs::SimpleDataRequestArgs {
                serial_number,
                object_type,
            } => self.handle_simple_data(serial_number, object_type),
            _ => eprintln!("simple-receiver: Received unknown event, dropping it."),
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
    fn handle_command(&self, event: CommandArgs) {
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

    // Register the receivers for each command they handle
    let mut command_receiver_registry: HashMap<CommandId, Box<dyn CommandReceiver>> =
        HashMap::new();
    command_receiver_registry.insert(
        CommandId::SimpleDataRequest,
        Box::new(SimpleReceiver {
            data_link: "UART0".to_string(),
        }),
    );
    command_receiver_registry.insert(
        CommandId::ComplexDataRequest,
        Box::new(ComplexReceiver {
            logger_name: "/sys/log".to_string(),
        }),
    );

    // Process the commands
    while command_queue.len() > 0 {
        let next = command_queue.pop_front().unwrap();
        match command_receiver_registry.get(&next.cmd) {
            Some(receiver) => receiver.handle_command(next.args),
            None => eprintln!("command-server: Received command with no receiver registered, dropping it on the floor."),
        }
    }
}
