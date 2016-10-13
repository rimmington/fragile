extern crate combine;
extern crate nix; // As in *nix
extern crate rand;
extern crate signal;

mod commander;

// Reference: https://github.com/NixOS/nixpkgs/blob/b07051ce6c3ba3039c89b6755da279002b0c3ace/nixos/modules/virtualisation/nixos-container.pl

use nix::sys::signal::{kill, SIGTERM, SIGINT};
use std::io;
use std::io::Write;
use std::process::Command;

#[derive(Debug)]
enum Error {
    Usage(String),
    NonZero(String, i32),
    ControlError(String),
    IoError(io::Error),
    NixError(nix::Error),
    Interrupted(commander::Signal)
}

use Error::*;

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self { IoError(e) }
}

impl From<nix::Error> for Error {
    fn from(e: nix::Error) -> Self { NixError(e) }
}

impl From<commander::NonZero> for Error {
    fn from(e: commander::NonZero) -> Self { match e { commander::NonZero::NonZero(s, o) => NonZero(s, o), commander::NonZero::Interrupted(n) => Interrupted(n) } }
}

type Result<T> = std::result::Result<T, Error>;

// TODO: why can't this be inferred away
fn run<T>(cmd: &mut Command) -> Result<T> where T: commander::CommandOut { commander::run(cmd) }

fn probably_unique_name(length: usize) -> String {
    use rand::distributions::{IndependentSample, Range};
    let between = Range::new(0, 36);
    let mut rng = rand::thread_rng();
    unsafe {
        std::str::from_utf8_unchecked(&(0..length).map(|_| between.ind_sample(&mut rng)).map(|v| if v < 10 { v + 48 } else { v + 87 }).collect::<Vec<u8>>()).to_string()
    }
}

// Result is of the form "10.233.x"
fn unused_ip_block() -> Result<String> {
    // Can bail early anywhere here because we've got the lock
    let mut used = std::collections::HashSet::new();
    for entry in try!(std::fs::read_dir("/etc/containers")) {
        let entry = try!(entry);
        if try!(entry.file_type()).is_file() {
            let mut str = String::new();
            use std::io::Read;
            try!(try!(std::fs::File::open(entry.path())).read_to_string(&mut str));
            for line in str.lines() {
                use combine::{string, many1, choice, digit, token, eof, Parser, ParserExt};
                let ip_component = || { string("10.233.").with(many1::<String, _>(digit())).skip(token('.')).skip(many1::<String, _>(digit())) };
                if let Ok((c, _)) = choice([string("HOST_ADDRESS="), string("LOCAL_ADDRESS=")]).with(ip_component()).skip(eof()).parse(line) {
                    used.insert(c);
                }
            }
        }
    }
    (0..255).map(|p| p.to_string()).find(|s| !used.contains(s)).map(|s| format!("10.233.{}", s)).ok_or(ControlError("Out of IP addresses".to_string()))
}

fn mkpath(mode : u32, path : &str) -> Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    match std::fs::DirBuilder::new().mode(mode).create(path) {
        Err(e) => if let std::io::ErrorKind::AlreadyExists = e.kind() { Ok(()) } else { Err(IoError(e)) },
        Ok(()) => Ok(())
    }
}

fn system_init() -> Result<()> {
    try!(mkpath(0o0755, "/etc/containers"));
    try!(mkpath(0o0700, "/var/lib/containers"));
    Ok(())
}

fn profile_dir(name : &str) -> String {
    format!("/nix/var/nix/profiles/per-container/{}", name)
}

fn container_root(name : &str) -> String {
    format!("/var/lib/containers/{}", name)
}

fn conf_file(name : &str) -> String {
    format!("/etc/containers/{}.conf", name)
}

fn create(config_file : &str) -> Result<String> {
    fn init_container_unsynced(name : &str) -> Result<()> {
        let ip_block = try!(unused_ip_block());
        // Use create_new to avoid clobbering existing container
        let mut f = try!(std::fs::OpenOptions::new().create_new(true).write(true).open(conf_file(name)));
        write!(&mut f, "PRIVATE_NETWORK=1\nHOST_ADDRESS={0}.1\nLOCAL_ADDRESS={0}.2\nAUTO_START=0\n", ip_block).map_err(IoError)
    }

    fn populate_container(name : &str, config_file : &str) -> Result<()> {
        use std::os::unix::fs::DirBuilderExt;
        // The per-container directory is restricted to prevent users on
        // the host from messing with guest users who happen to have the
        // same uid.
        try!(mkpath(0o0700, "/nix/var/nix/profiles/per-container"));
        let profile_dir = profile_dir(name);
        let mut dir_builder = std::fs::DirBuilder::new();
        try!(dir_builder.mode(0o0755).create(&profile_dir));

        let container_root = container_root(name);
        dir_builder.recursive(true);
        try!(dir_builder.mode(0o0755).create(format!("{}/etc/nixos", container_root)));

        // Write configuration.nix into the container fs
        let configuration_nix = format!("{}/etc/nixos/configuration.nix", container_root);
        {
            let mut f = try!(std::fs::File::create(format!("{}/etc/nixos/configuration.nix", container_root)));
            try!(write!(&mut f, "
    {{ config, lib, pkgs, ... }}:
    with lib;

    {{
        boot.isContainer = true;
        networking.hostName = mkDefault \"{}\";
        networking.useDHCP = false;
        imports = [ {} ];
    }}", name, config_file));
            // TODO: guaranteed sync at end of block?
        }

        // Build the container config
        run(Command::new("nix-env")
            .arg("-p").arg(format!("{}/system", profile_dir))
            .arg("-I").arg(format!("nixos-config={}", configuration_nix))
            .arg("-f").arg("<nixpkgs/nixos>")
            .arg("--set")
            .arg("-A").arg("system")
            .arg("--show-trace"))
    }

    let name = format!("fr{}", probably_unique_name(9));

    // Acquire lock
    use std::os::unix::io::IntoRawFd;
    let lock_fd = try!(std::fs::OpenOptions::new().create(true).append(true).open("/run/lock/nixos-container")).into_raw_fd();
    try!(nix::fcntl::flock(lock_fd, nix::fcntl::FlockArg::LockExclusive));

    let res = init_container_unsynced(&name);

    // Drop lock
    unsafe {
        let _ = nix::libc::close(lock_fd);  // If closing fails, ¯\_(ツ)_/¯
    }

    match res.and_then(|()| populate_container(&name, config_file)) {
        Ok(()) => Ok(name),
        // Don't leave a half-baked container behind
        Err(e) => { let _ = destroy(&name); Err(e) }
    }
}

fn run_test(container_name : &str, args : &[String]) -> Result<i32> {
    try!(run(Command::new("systemctl").arg("start").arg(format!("container@{}", container_name))));
    let leader_pid = try!(run(Command::new("machinectl").arg("show").arg(container_name).arg("-p").arg("Leader")).and_then(|o : String| {
        use combine::{string, many1, digit, spaces, eof, Parser, ParserExt};
        string("Leader=").with(many1::<String, _>(digit())).skip(spaces()).skip(eof()).parse(o.as_str())
            .map(|(ds, _)| ds)
            .map_err(|_| ControlError(format!("Bad machinectl output {}", o)))
    }));
    let su_cmd = args.iter().map(|s| format!("'{}'", s.replace("'", "'\\''"))).collect::<Vec<String>>().join(" ");
    match run(Command::new("nsenter")
            .arg("-t").arg(leader_pid)
            .arg("-m").arg("-u").arg("-i").arg("-n").arg("-p")
            .arg("--").arg("/var/setuid-wrappers/su").arg("root").arg("-l").arg("-c").arg(format!("exec {}", su_cmd))) {
        Ok(()) => Ok(0),
        Err(NonZero(_, code)) => Ok(code),
        Err(e) => Err(e)
    }
}

// Remove a directory while recursively unmounting all mounted filesystems within
// that directory and unmounting/removing that directory afterwards as well.
//
// Specified path shouldn't be a mountpoint.
fn safe_remove_tree(path : &str) -> Result<()> {
    if !std::path::Path::new(path).is_dir() {
        return Ok(());
    }
    try!(run(Command::new("find").arg(path)
        .arg("-mindepth").arg("1")
        .arg("-xdev")
        .arg("(")
            .arg("-type").arg("d")
            .arg("-exec").arg("mountpoint").arg("-q").arg("{}").arg(";")
        .arg(")")
        .arg("-exec").arg("umount").arg("-fR").arg("{}").arg("+")));
    try!(run(Command::new("rm").arg("--one-file-system").arg("-rf").arg(path)));

    Ok(())
}

fn stop(container_name : &str) -> Result<()> {
    match run(Command::new("systemctl").arg("stop").arg(format!("container@{}", container_name))) {
        Ok(()) => Ok(()),
        Err(Interrupted(_)) => {
            return run(Command::new("systemctl").arg("kill").arg(format!("container@{}", container_name)))
        },
        Err(e) => Err(e)
    }
}

fn destroy(container_name : &str) -> Result<()> {
    fn log_err(e: Error) {
        println!("Error while destroying container: {:?}", e);
    }

    safe_remove_tree(&profile_dir(container_name)).unwrap_or_else(log_err);
    safe_remove_tree(&format!("/nix/var/nix/gcroots/per-container/{}", container_name)).unwrap_or_else(log_err);
    safe_remove_tree(&container_root(container_name)).unwrap_or_else(log_err);

    std::fs::remove_file(conf_file(container_name)).or_else(|e| match e.kind() {
        std::io::ErrorKind::NotFound => Ok(()),
        _ => Err(IoError(e))
    })
}

fn go() -> Result<i32> {
    // Skip program name
    let mut args = std::env::args().skip(1).peekable();

    let no_destroy = args.peek().map_or(false, |arg| arg == "--no-destroy");
    if no_destroy { args.next(); }

    let config_file = try!(args.next().ok_or(Error::Usage("Missing config file argument".to_string())));

    let test_args : Vec<String> = args.collect();
    if test_args.len() == 0 {
        return Err(Usage("Missing test args".to_string()));
    }

    let config_file = {
        let p = try!(std::fs::canonicalize(config_file).map_err(|e| Usage(format!("For config file: {}", e))));
        try!(p.to_str().ok_or(Usage("Config file path is invalid UTF-8".to_string()))).to_string()
    };

    try!(system_init());
    // We suppress signals in this process and allow the commander module to deal with child processes
    let trap = signal::trap::Trap::trap(&[SIGTERM, SIGINT]);
    let container_name = try!(create(&config_file));
    let res = match trap.wait(std::time::Instant::now()) {
        None => run_test(&container_name, &test_args),
        Some(n) => Err(Interrupted(n))
    };

    let _ = stop(&container_name);
    if !no_destroy {
        try!(destroy(&container_name));
    }

    res
}

fn main() {
    std::process::exit(go().unwrap_or_else(|e| match e {
        NonZero(cmd, code) => { writeln!(io::stderr(), "Command {:?} failed with code {:?}", cmd, code).unwrap(); 2 },
        Usage(msg) => { writeln!(io::stderr(), "Usage error: {}", msg).unwrap(); 1 },
        IoError(err) => { writeln!(io::stderr(), "{}", err).unwrap(); 2 },
        NixError(err) => { writeln!(io::stderr(), "{}", err).unwrap(); 2 },
        ControlError(msg) => { writeln!(io::stderr(), "{}", msg).unwrap(); 2 },
        Interrupted(SIGINT) => { kill(nix::unistd::getpid(), SIGINT).unwrap(); loop {} }
        Interrupted(n) => 128 + n
    }));
}
