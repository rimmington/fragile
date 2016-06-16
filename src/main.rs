extern crate rand;

use std::io;
use std::io::Write;
use std::process::Command;

enum Error {
    Usage(&'static str),
    NonZero(String, Option<i32>),
    IoError(io::Error)
}

use Error::*;

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self { IoError(e) }
}

type Result<T> = std::result::Result<T, Error>;

trait CommandOut : Sized {
    fn run(&mut Command) -> Result<Self>;
}

impl CommandOut for () {
    fn run(cmd: &mut Command) -> Result<Self> {
        let exit = try!(cmd.spawn().and_then(|mut c| c.wait()));
        if exit.success() {
            Ok(())
        } else {
            Err(NonZero(format!("{:?}", cmd), exit.code()))
        }
    }
}

impl CommandOut for String {
    fn run(cmd: &mut Command) -> Result<Self> {
        use std::process::Stdio;
        use std::io::Read;
        let mut buffer = String::new();
        let mut child = try!(cmd.stderr(Stdio::null()).stdout(Stdio::piped()).spawn());
        let rdres = child.stdout.as_mut().unwrap().read_to_string(&mut buffer);
        let wtres = child.wait();
        let exit = try!(rdres.and(wtres).map_err(IoError));
        if exit.success() {
            Ok(buffer)
        } else {
            Err(NonZero(format!("{:?}", cmd), exit.code()))
        }
    }
}

fn run<T>(cmd: &mut Command) -> Result<T> where T : CommandOut {
    T::run(cmd)
}

fn probably_unique_name(length: u64) -> String {
    use rand::distributions::{IndependentSample, Range};
    let between = Range::new(0, 36);
    let mut rng = rand::thread_rng();
    unsafe {
        std::str::from_utf8_unchecked(&(0..length).map(|_| between.ind_sample(&mut rng)).map(|v| if v < 10 { v + 48 } else { v + 87 }).collect::<Vec<u8>>()).to_string()
    }
}

fn create(nixpkgs : String, config_file : String) -> Result<String> {
    let name = format!("fr{}", probably_unique_name(9));
    match run(Command::new("nixos-container").arg("create").arg(&name).arg("--config").arg(format!("imports = [ {} ];", config_file)).env("NIX_PATH", format!("nixpkgs={}", nixpkgs))) {
        Ok(()) => Ok(name),
        Err(e) => { let _ = destroy(&name); Err(e) }
    }
}

fn destroy(container_name : &str) -> Result<()> {
    run(Command::new("nixos-container").arg("destroy").arg(container_name))
}

fn go() -> Result<()> {
    let mut args = std::env::args();
    args.next(); // Discard program name
    let nixpkgs = try!(args.next().ok_or(Error::Usage("Missing nixpkgs argument")));
    let config_file = try!(args.next().ok_or(Error::Usage("Missing config file argument")));

    let container_name = try!(create(nixpkgs, config_file));
    println!("{}", container_name);
    try!(destroy(&container_name));
    Ok(())
}

fn main() {
    go().unwrap_or_else(|e| std::process::exit(match e {
        NonZero(cmd, code) => { writeln!(io::stderr(), "Command {:?} failed with code {:?}", cmd, code).unwrap(); 2 },
        Usage(msg) => { writeln!(io::stderr(), "Usage error: {}", msg).unwrap(); 1 },
        IoError(err) => { writeln!(io::stderr(), "{}", err).unwrap(); 2 }
    }));
}
