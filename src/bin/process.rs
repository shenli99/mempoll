use mempoll::{memory::proc_memory, process::{MapRange, MemoryType}, searcher::{self, MemorySearcher, SearchType}};

fn main() {
    let mut proc = proc_memory::ProcMemory::new(22671);
    match proc.open() {
        Ok(_) => {},
        Err(e) => println!("{:#?}", e),
    }
    match proc.process.maps() {
        Ok(_) => {
            let rule = searcher::SearchType::Eq(0xDEADBEEFu32);
            match proc.search::<SearchType<u32>, 4096>(rule, Some(|m: &MapRange|m.memory_type == MemoryType::Ch)) {
                Ok(v) => {
                    println!("{:#X}", v.first().unwrap());
                },
                Err(e) => {
                    println!("{:#?}", e);
                }
            }
            // println!("{:#?}", proc.process.maps);
        },
        Err(e) => {
            println!("{:#?}", e);
        }
    }
}