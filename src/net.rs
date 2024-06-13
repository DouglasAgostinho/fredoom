pub mod network{
    
    //use std::os::unix::net::SocketAddr;
    use std::thread;        
    use std::io::{self,Read, Write};
    use std::net::{TcpListener, TcpStream};    
    //use sha2::digest::consts::False;
    

    //----------Constants----------//

    //to use in String based variables
    //const EMPTY_STRING: String = String::new();

    //Max number of peers
    const MAX_PEERS: u8 = 3;

    //Message code size
    pub const CODE_SIZE: usize = 3;
    

    fn handle_message(message: &String, mode: &str) -> bool{
        //Function to treat incoming / outgoing messages        
        match mode {

            "send" => {false},

            "receive" => {
                let msg = message.trim();
                
                match msg {                    

                    "[!]_stream_[!]" => true, 
                                   
                    _ => {
                        println!("Received: {}", message);    
                        
                        let len = message.len();
                        let tail = &message[len - CODE_SIZE .. len];       

                        match tail {

                            "002" => println!("Received Block => {}", message),
                            _ => (),                            
                        }                                                          
                        false //to_do Will return decrypted message
                    },
                }
            },

            "test" => {
                println!("Received: {}", message);                
                false
            },

            _ => false,
        }
    }


    fn handle_client(mut stream: TcpStream) {        

        let income_addr = stream.peer_addr().expect("Error");
        //println!("Incoming connection from {}", stream.peer_addr()?);   
        println!("Incoming connection from {}", income_addr);
        let mut buf = [0; 1024];

        loop {
                        
            let bytes_read = stream.read(&mut buf).expect("Error");            
            if bytes_read == 0 {break}
            
            let received = String::from_utf8_lossy(&buf[..bytes_read]);

            if handle_message(&received.to_string(), "receive") {                                
                                                     
                stream.write_all("ready_to_receive".as_bytes()).expect("error");
                // Receive data continuously from the server
                let mut buffer = [0; 1024];                    
                loop {
                    match stream.read(&mut buffer) {
                        Ok(0) => {
                            println!("Connection closed by server");
                            break;
                        },
                        Ok(n) => {
                            let msg = String::from_utf8_lossy(&buffer[0..n]);
                            print!("{}", msg);      //Uses print! to not insert /n after each received data                                                            
                            io::stdout().flush().expect("error");  // Ensure immediate output
                        },
                        Err(e) => {
                            println!("Failed to receive message: {}", e);
                            break;
                        }
                    }
                }                    
            } 
            else {
                println!("Connection closed by server");
                break;                
            }           
        }
    }


    pub fn net_init(port: &str){
        //Composing IP address with received port
        let mut addr = String::from("0.0.0.0");
        addr.push_str(port);

        //Set system to listen
        let listener = TcpListener::bind(addr).expect("Could not bind");
        println!("Server initialized...");
        
        //thread::spawn( || to_net("!who_is_alive!"));

        for stream in listener.incoming(){
            match stream {
                Err(e) => println!("Error found {e}"),
                Ok(stream) => {
                    thread::spawn(move || {
                        //handle_client(stream).unwrap_or_else(|error| println!("Error {:?}", error));
                        handle_client(stream);
                    });
                }
            }
        }
    }        
    
    /// Broadcast message to all Network    
    pub fn to_net(send_what: &str) {                

        let mut message = serde_json::to_string(send_what).expect("Error");

        message.push_str("002");

        for n in 1..MAX_PEERS {             
            //Loop through all address
            let address = format!("192.168.191.{}:6886", n);

            //call client function to send message
            match client(send_what, &address, "simple"){
                Ok(_) => (),
                Err(e) => println!("On host {} Error found {}",address, e),
            }            
        }
    }    


    fn client(message: &str, address: &str, mode: &str)-> io::Result<()> {
        match mode {
            "simple" => {
                // Connect to the server
                let mut stream = TcpStream::connect(address)?;
                // Send data to the server
                stream.write_all(message.as_bytes())?;
                Ok(())
            },
            "serialized" => {
                // Connect to the server
                let mut stream = TcpStream::connect(address)?;

                //println!("Pre Serde {:?}", message);

                let serialized = serde_json::to_string(&message)?;
                stream.write_all(serialized.as_bytes())?;
                Ok(())
            },
            "test" => {
                // Connect to the server
                let mut stream = TcpStream::connect(address)?;
                stream.write_all(message.as_bytes())?;
                Ok(())
            },
            _ => Ok(()),
        }
    }
}