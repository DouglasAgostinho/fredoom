
/*

    This program is intended to be a place where .......

    #Message code table version - 000.01
    00000 - life beat message that broadcast listening port.

*/

//Modules declaration
mod net;
mod block;
mod crypt;


use std::io; 
use std::thread;
use hex::encode;
use block::{Block, Node};
use net::network::{self, VERSION};
use std::time::{Duration, SystemTime};
use std::sync::mpsc::{self, Receiver, Sender};
use tracing::{span, info, error, debug, Level};
use crypt::crypt::{generate_own_keys, generate_shared_key, encrypt, decrypt, test_keys};



//Constant to use in String based variables
const EMPTY_STRING: String = String::new();

//Time constants
const MINUTE: Duration = Duration::from_secs(10);

//Logging path constant
const LOG_PATH: &str = "/home/ares/Documentos/rust/fredoom";

//const MY_ADDRESS: &str = "xyz6886";

///Receive an item from a Vector of vector(String) if match the NODE Address
fn get_msg_from_blocks(mut block: Vec<[String; 3]>, addr: String) -> Vec<[String; 3]>{

    //Create and vector to receive index of match messages
    let mut to_remove = Vec::new();
    
    //Loop through Block messages to find desired value
    for (num, val) in block.iter().enumerate(){
    
        if val[2] == addr {            
            to_remove.push(num);
        }
    }
    
    //Loop to remove desired messages
    for n in to_remove{
        block.swap_remove(n);
    }

    //Return block without removed messages
    block
}


fn local_users(tx: Sender<String>){

    //Entering Local menu Loggin Level
    let span: span::Span = span!(Level::INFO,"Local menu");
    let _enter: span::Entered = span.enter();

    
    loop {                
        //Variable to receive user input
        let mut user_input = EMPTY_STRING;

        //Get user input
        info!("Please enter the message");
        match io::stdin().read_line(&mut user_input) {
            Ok(_) => (),
            Err(e) => println!("Error found {}", e),
        }        

        //Send user input to main thread
        if tx.send(user_input).is_err() {
            eprintln!("Failed to send input to main thread.");
            break;
        }
    }  
}

fn handle_thread_msg(message_receiver: &Receiver<String>) -> String{

    //Entering Thread message Loggin Level
    let span: span::Span = span!(Level::INFO,"Thread message");
    let _enter: span::Entered = span.enter();

    match message_receiver.try_recv() {
        Ok(msg) => {
            //Return input received
            info!("Received input: {:?}", msg);
            msg
        },
        Err(mpsc::TryRecvError::Empty) => {
            // No input received, return Empty String 
            EMPTY_STRING
        }
        Err(mpsc::TryRecvError::Disconnected) => {
            error!("Input thread has disconnected.");
            EMPTY_STRING
        }
    }
}

fn handle_net_msg(message_receiver: &Receiver<[String; 3]>) -> [String; 3]{

    //Entering Net message Loggin Level
    let span: span::Span = span!(Level::INFO,"Net message");
    let _enter: span::Entered = span.enter();

    match message_receiver.try_recv() {
        Ok(msg) => {
            //Return input received
            info!("Received input: {:?}", msg);
            msg
        },
        Err(mpsc::TryRecvError::Empty) => {
            // No input received, return Empty String 
            [EMPTY_STRING; 3]
        }
        Err(mpsc::TryRecvError::Disconnected) => {
            error!("Input thread has disconnected.");
            [EMPTY_STRING; 3]
        }
    }
}

fn main() {    

    //Instatiate the subscriber.
    //tracing_subscriber::fmt::init();
    

    let file_appender = tracing_appender::rolling::hourly(LOG_PATH, "log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .init();


    //Entering Main Loggin Level
    let span: span::Span = span!(Level::INFO,"Main");
    let _enter: span::Entered = span.enter();
    
    
    //Initial greetins (todo main menu)----------------------------------------------------------------------------------
    println!("Welcome to FREDOOM !!!");

    //Initiate Thread message channel Tx / Rx 
    let (input_message, message_receiver) = mpsc::channel();
    let (net_message, net_receiver) = mpsc::channel();

    //Spawn thread for server initialization    
    thread::spawn( move || network::net_init(net_message));

    //Instance of Block struct    
    let mut blocks: Block = Block{
        message: Vec::from([[EMPTY_STRING; 3]])
    };

    //Instance of Node struct
    let mut my_node: Node = Node{address:EMPTY_STRING};
    my_node.address = my_node.gen_address();

    //Initiate time measurement - for time triggered features
    let mut now = SystemTime::now();

    //Spawn thread for handle local user interaction
    thread::spawn(move || {local_users(input_message)});

    loop{

        //Buffer to store received messages
        let mut message_buffer: Vec<String> = Vec::new();

        //Control of time triggered features
        match now.elapsed(){

            Ok(n) => {
                info!("Tempo => {:?}", n); //Debug print - to_do change to crate tracer event
                if n >= MINUTE{
                    info!("One minute"); //to_do change to crate tracer event

                    //Propagate self IP address and port
                    let mut message = serde_json::to_string(&blocks.message).expect("Error");
                    message.push_str("00001");    //00000 - code for life beat message (check message code table)
                    message.push_str(VERSION);

                    let msg = message.clone();
                    //Spawn thread to propagate listening port to all network                  
                    thread::spawn(move || network::to_net(msg));

                    now = SystemTime::now();
                }
            },
            Err(e) => println!("Error {}", e),            
        }        

        // Check for new messages from the input thread
        message_buffer.push(handle_thread_msg(&message_receiver));

        // Check for new messages from the network thread
        let mut net_msg: [String; 3] = handle_net_msg(&net_receiver);

        loop {

            debug!("Net msg => {}", net_msg[0]);
            if net_msg[0] != EMPTY_STRING {                
    
                if !blocks.message.contains(&net_msg){

                    //Call insert function to format and store in a block section
                    blocks.insert(net_msg.clone());
                }                

                net_msg = [EMPTY_STRING; 3];
            }
            else {
                break;
            }             
        }
        

        loop{

            let mut user_msg: String = String::new();

            if let Some(_) =  message_buffer.get(0){

                user_msg = message_buffer.swap_remove(0);
            }
            
            
            if user_msg != EMPTY_STRING {
                //Organize data to fit in the message format [current time, address, message text]
                let message: [String; 3] = [my_node.get_time_ns(), my_node.address.clone(), String::from(user_msg.trim())];
                
                //Call insert function to format and store in a block section
                blocks.insert(message.clone());                
            }
            else {
                break;
            }                        
        }
        
        println!(" Blocks => {:?}", blocks.message);
        blocks.message = get_msg_from_blocks(blocks.message, "remove".to_string());
        thread::sleep(Duration::from_millis(3000));    

        
        let client_pb_key = test_keys();

        let (pv_key, _pb_key) = generate_own_keys();  

        let shared_key = generate_shared_key(pv_key, client_pb_key);

        //let msg_to_crypt = "secretamente".to_string();
        let msg_to_crypt = serde_json::to_string(&blocks.message).expect("Error");

        let crypt_msg = encrypt(shared_key, msg_to_crypt);

        debug!("Encriptada {:?}", encode(&crypt_msg));

        let decrypted_msg = decrypt(shared_key, crypt_msg);

        debug!("Decriptada {}", decrypted_msg);

    }

}
