use std::net::{TcpListener,TcpStream};
use std::thread;
use std::io::BufReader;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::mpsc;

fn fn main() {

	let (customer, barber) = mpsc::sync_channel(3);

    for i in 0..{

    	let customer = customer.clone();

    	thread::spawn( move || {
    		
    		println!("customer {} Comes", i);
    		match customer.try_send(i).unwrap() {
    			Ok(_) => println!("customer {} begins waiting", i)
    			Err(_) =>
    		}
    	})

    }
}


