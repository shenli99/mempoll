use mempoll;

fn main() {
    let mut process = mempoll::process::Process::new(29875);
    match process.maps() {
        Err(e) => println!("Err: {:?}", e),
        Ok(_) => {
            println!("Ok: {:#?}", process);
        }
    }
}
