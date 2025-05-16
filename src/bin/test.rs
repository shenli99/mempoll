
use mempoll::searcher;

fn main() {
    let sr = searcher::SearchRule::Lt(12i16);
    let data: [i16; 20] = [99,90,53,92,29,39,42,12,92,79,23,8,22,53,59,85,83,18,96,12];
    let size = std::mem::size_of::<i16>();

    let res = sr.search::<16>(unsafe {
        std::slice::from_raw_parts(data.as_ptr() as *const u8, size * data.len())
    }).collect::<Vec<usize>>();
    for k in res {
        println!("{:?}({:?}): {:?}", k, k/size, data[k/size])
    }
}
