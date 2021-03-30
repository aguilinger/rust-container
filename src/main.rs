use tokio::process::Command;
use std::process::Output;
use std::io::Error;
use std::env;
use nix::sched::*;
use nix::Result as NixResult;
use std::process;
use std::fs;
use rtnetlink::*;
extern crate tokio;

use tokio::runtime::Runtime;

use std::thread::spawn;
use tokio::*;


fn run(command: String) -> Command {
    return Command::new("sh");
}

fn create_namespaces() -> NixResult<()> {
    let mut clone_flags = CloneFlags::empty();
    clone_flags.insert(CloneFlags::CLONE_NEWPID);
    clone_flags.insert(CloneFlags::CLONE_NEWNET);
    clone_flags.insert(CloneFlags::CLONE_THREAD);
    let result = unshare(clone_flags);
    return result;
}
fn print_namespaces(pid: u32) {
    for ns in ["pid", "ipc", "mnt", "net", "user", "uts"].iter() {
        let filename = format!("/proc/{}/ns/{}", pid, ns);
        println!("Reading {}", filename);
        let contents = fs::read_link(filename)
            .expect("Something went wrong reading the file");

        println!("Namspace {}: {}", ns, contents.to_str().unwrap());
    };
}

async fn create_veth(pid: u32) -> Result<(), String> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);
    handle
        .link()
        .add()
        .veth("veth-rs-1".into(), "veth-rs-2".into())
        .execute()
        .await
        .map_err(|e| format!("{}", e));
    handle.link().set(0).setns_by_pid(pid).execute().await.map_err(|e| format!("{}", e))
}


#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("Hello, world!");
    let args: Vec<String> = env::args().collect();
    let command = (&args[1]).to_string();
    let mut running_command = Command::new(&args[1]);
    if args.len() > 2 {
        for command_arg in &args[2..] {
            running_command.arg(command_arg);
        }
    }
    println!("Command {:?}", running_command);
    let pid: u32 = process::id();
    print_namespaces(pid);
    let r = create_namespaces();
    dbg!(r);
    match running_command.spawn() {
      Ok(child) => {
        let child_pid = child.id().unwrap();
        println!("Child's ID is {}", child_pid);
        print_namespaces(child_pid);
        create_veth(child_pid);

      },
      Err(e) => println!("Error: {}", e)
    }
}