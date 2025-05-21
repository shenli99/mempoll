
use mempoll::searcher::{self, SearchRule};

fn main() {
    let sr = searcher::SearchType::Lt(30i32);
    let data: [i32; 20] = [99,90,53,92,29,39,42,12,92,79,23,8,22,53,59,85,83,18,96,12];
    let size = std::mem::size_of::<i32>();

    let res = sr.search(unsafe {
        std::slice::from_raw_parts(data.as_ptr() as *const u8, size * data.len())
    }, data.len() * size).collect::<Vec<usize>>();

    println!("{:?}", res);

    for k in res {
        println!("{:?}({:?}): {:?}", k, k/size, data[k/size])
    }
}
