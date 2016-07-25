extern crate nix;
extern crate signal;

use nix::sys::signal::{kill, SIGTERM, SIGINT, SIGCHLD};
use nix::sys::wait::{waitpid, WNOHANG};
use std::io;
use std::process::Command;

pub type Signal = i32;

pub enum NonZero {
    NonZero(String, i32),
    Interrupted(Signal)
}

pub trait CommandOut : Sized {
    fn run<E>(&mut Command) -> Result<Self, E> where E: From<io::Error> + From<NonZero>;
}

impl CommandOut for () {
    fn run<E>(cmd: &mut Command) -> Result<Self, E> where E: From<io::Error> + From<NonZero> {
        let trap = signal::trap::Trap::trap(&[SIGTERM, SIGINT, SIGCHLD]);
        let child = try!(cmd.spawn());
        let child_id = child.id() as i32;
        // There's a race here (might drop a SIGTERM), but there's no nice way to avoid it
        for sig in trap { match sig {
            SIGCHLD => {
                let status = try!(waitpid(child_id, Some(WNOHANG)).map_err(|e| match e { nix::Error::Sys(no) => io::Error::from_raw_os_error(no as i32), nix::Error::InvalidPath => unreachable!() }));
                use nix::sys::wait::WaitStatus::*;
                match status {
                    Exited(_, exit8) => return if exit8 == 0 { Ok(()) } else { Err(E::from(NonZero::NonZero(format!("{:?}", cmd), i32::from(exit8)))) },
                    Signaled(_, sig, _dunno) => return Err(E::from(NonZero::NonZero(format!("{:?}", cmd), -(sig as i32)))),
                    _ => continue
                };
            }
            e => {
                let _ = kill(child_id, SIGTERM);
                return Err(E::from(NonZero::Interrupted(e)));
            }
        }}
        unreachable!()
    }
}

impl CommandOut for String {
    fn run<E>(cmd: &mut Command) -> Result<Self, E> where E: From<io::Error> + From<NonZero> {
        use std::process::Stdio;
        use std::io::Read;
        let mut buffer = String::new();
        let mut child = try!(cmd.stderr(Stdio::null()).stdout(Stdio::piped()).spawn());
        let rdres = child.stdout.as_mut().unwrap().read_to_string(&mut buffer);
        let wtres = child.wait();
        let exit = try!(rdres.and(wtres));
        if exit.success() {
            Ok(buffer)
        } else {
            Err(E::from(NonZero::NonZero(format!("{:?}", cmd), exit.code().unwrap())))
        }
    }
}

pub fn run<T, E>(cmd: &mut Command) -> Result<T, E> where T : CommandOut, E: From<io::Error> + From<NonZero> {
    T::run(cmd)
}
