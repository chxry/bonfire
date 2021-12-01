use std::thread;
use std::sync::{Arc, Mutex};
use std::net::{TcpListener, TcpStream};
use bonfire_shared::{Result, Message};

fn main() -> Result {
  let listener = TcpListener::bind("0.0.0.0:5555")?;
  let clients: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(vec![]));
  println!("listening on 5555");

  for stream in listener.incoming() {
    let stream = stream?;
    let clients = clients.clone();
    clients.lock().unwrap().push(stream.try_clone()?);
    thread::spawn(|| handle_client(stream, clients));
  }
  Ok(())
}

fn handle_client(stream: TcpStream, clients: Arc<Mutex<Vec<TcpStream>>>) -> Result {
  let user: String = bincode::deserialize_from(&stream)?;
  broadcast_msg(
    Message {
      user: "server".to_string(),
      content: format!("{} connected", user),
    },
    &clients,
  )?;
  loop {
    let msg: String = bincode::deserialize_from(&stream)?;
    broadcast_msg(
      Message {
        user: user.clone(),
        content: msg,
      },
      &clients,
    )?;
  }
}

fn broadcast_msg(msg: Message, clients: &Arc<Mutex<Vec<TcpStream>>>) -> Result {
  println!("{}: {}", msg.user, msg.content);
  for client in clients.lock().unwrap().iter() {
    bincode::serialize_into(client, &msg)?;
  }
  Ok(())
}
