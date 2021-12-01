use std::{thread, process, fs};
use std::sync::{Arc, Mutex};
use std::net::{TcpStream, SocketAddrV4};
use std::io::{stdout, stdin, Write};
use termion::screen::AlternateScreen;
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use termion::event::Key;
use termion::{clear, cursor, color, style, terminal_size};
use serde::{Deserialize};
use bonfire_shared::{Result, Message};

fn main() -> Result {
  let config = Config::load()?;
  let mut stdout = AlternateScreen::from(stdout().into_raw_mode()?);
  let channel = Arc::new(Mutex::new(vec![]));
  let buf = Arc::new(Mutex::new(String::new()));
  let stream = TcpStream::connect(config.server)?;

  bincode::serialize_into(&stream, &config.username)?;
  watch(&stream, channel.clone())?;
  input(&stream, buf.clone())?;

  loop {
    let (w, h) = terminal_size()?;
    let status = format!("bonfire | {} | ctrl+q to exit", config.server);
    write!(
      stdout,
      "{}{}{}{}{}{}{}",
      clear::All,
      cursor::Goto(1, 1),
      color::Bg(color::LightBlack),
      status,
      " ".repeat(w as usize - status.len()),
      color::Bg(color::Reset),
      cursor::Goto(1, h - 1)
    )?;
    for (i, msg) in channel
      .lock()
      .unwrap()
      .iter()
      .rev()
      .take((h - 2).into())
      .enumerate()
    {
      write!(
        stdout,
        "{}{}:{} {}{}",
        style::Bold,
        msg.user,
        style::Reset,
        msg.content,
        cursor::Goto(0, h - i as u16 - 2)
      )?;
    }
    let buf = buf.lock().unwrap();
    write!(
      stdout,
      "{}{}{}{}{}{}",
      cursor::Goto(1, h),
      color::Bg(color::Black),
      buf,
      " ".repeat(w as usize - buf.len()),
      cursor::Goto(buf.len() as u16 + 1, h),
      color::Bg(color::Reset)
    )?;
    stdout.flush()?;
  }
}

fn watch(stream: &TcpStream, channel: Arc<Mutex<Vec<Message>>>) -> Result {
  let stream = stream.try_clone()?;
  thread::spawn(move || -> Result {
    loop {
      let msg = bincode::deserialize_from(&stream)?;
      channel.lock().unwrap().push(msg);
    }
  });
  Ok(())
}

fn input(stream: &TcpStream, buf: Arc<Mutex<String>>) -> Result {
  let stream = stream.try_clone()?;
  thread::spawn(move || -> Result {
    for key in stdin().keys() {
      match key? {
        Key::Char('\n') => {
          let buf = &mut *buf.lock().unwrap();
          bincode::serialize_into(&stream, buf)?;
          buf.clear();
        }
        Key::Char(c) => buf.lock().unwrap().push(c),
        Key::Backspace => {
          buf.lock().unwrap().pop();
        }
        Key::Ctrl('q') => process::exit(0),
        _ => {}
      }
    }
    Ok(())
  });
  Ok(())
}

#[derive(Deserialize)]
struct Config {
  pub username: String,
  pub server: SocketAddrV4,
}

impl Config {
  pub fn load() -> Result<Self> {
    let data = match fs::read_to_string("config.toml") {
      Ok(data) => data,
      Err(_) => {
        let data = include_str!("default_config.toml");
        fs::write("bonfire.toml", data)?;
        data.to_string()
      }
    };
    match toml::from_str(&data) {
      Ok(config) => Ok(config),
      Err(_) => panic!("Could not parse config"),
    }
  }
}
