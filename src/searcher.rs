use std::simd::{ Simd, Mask, cmp::{ SimdPartialOrd, SimdPartialEq }};
use crate::process::MapRange;

#[derive(Debug)]
pub enum SearchError{
    TypeError,
    ReadError(String)
}

pub trait MemorySearcher {
    fn search<T: SearchRule, const N: usize>(&self, rule: T, filter: Option<impl Fn(&MapRange) -> bool>) -> Result<Vec<usize>, SearchError>;
}

#[derive(Debug, Clone, Copy)]
pub enum SearchType<T: Copy> {
    ///Eq(a) equal to a == value
    Eq(T),
    //Ge(a) equal to a < value
    Gt(T),
    //Ge(a) equal to a <= value
    Ge(T),
    //Le(a) equal to a >= value
    Lt(T),
    //Le(a) equal to a >= valu
    Le(T),
    //Bte(a, b) equal to [a, b]
    Bte(T, T),
    //Bter(a, b) equal to [a, b)
    Bter(T, T),
    //Btel(a, b) equal to (a, b]
    Btel(T, T)
}

pub trait SearchRule: Copy {
    fn search<'a>(self, data: &'a [u8], len: usize) -> impl Iterator<Item = usize> + 'a;
}

macro_rules! search_rule {
    { $($number:ty),* } => {
        $(
            impl SearchRule for SearchType<$number>
            {
                fn search<'a>(self, data: &'a [u8], len: usize) -> impl Iterator<Item = usize> + 'a
                {
                    #[cfg(target_feature = "avx2")]
                    const LANES_LEN: usize = 256;
                    #[cfg(target_feature = "neon")]
                    const LANES_LEN: usize = 128;
                    #[cfg(not(any(target_feature = "avx2", target_feature = "neon")))]
                    const LANES_LEN: usize = 128;
                    let mut offset = 0;
                    let rule = self;
                    const SIZE: usize = std::mem::size_of::<$number>();
                    const LANES: usize = LANES_LEN / SIZE / 8;
                    let mut pending: std::collections::VecDeque<usize> = std::collections::VecDeque::with_capacity(LANES);
                    let buff = unsafe { std::slice::from_raw_parts(
                        data.as_ptr() as *const $number,
                        len / SIZE
                    )};
                    std::iter::from_fn(move || {
                        if let Some(pos) = pending.pop_front() {
                            return Some(pos);
                        }

                        while offset + LANES <= buff.len() {
                            let chunk: Simd<$number, LANES> = Simd::from_slice(&buff[offset..offset+LANES]);
                            let mask: Mask<_, LANES> = match rule {
                                SearchType::Eq(v) => chunk.simd_eq(Simd::splat(v)),
                                SearchType::Gt(v) => chunk.simd_gt(Simd::splat(v)),
                                SearchType::Ge(v) => chunk.simd_ge(Simd::splat(v)),
                                SearchType::Lt(v) => chunk.simd_lt(Simd::splat(v)),
                                SearchType::Le(v) => chunk.simd_le(Simd::splat(v)),
                                SearchType::Bte(v0, v1) => chunk.simd_ge(Simd::splat(v0)) & chunk.simd_le(Simd::splat(v1)),
                                SearchType::Bter(v0, v1) => chunk.simd_ge(Simd::splat(v0)) & chunk.simd_lt(Simd::splat(v1)),
                                SearchType::Btel(v0, v1) => chunk.simd_gt(Simd::splat(v0)) & chunk.simd_le(Simd::splat(v1)),
                            };
                            let bits = mask.to_bitmask();
                            if bits != 0 {
                                for i in 0..LANES {
                                    if bits & (1 << i) != 0 {
                                        pending.push_back((offset + i) * SIZE);
                                    }
                                }
                                offset += LANES;
                                return pending.pop_front();
                            }
                            offset += LANES;
                        }

                        while offset < buff.len() {
                            let v = buff[offset];
                            let hit = match rule {
                                SearchType::Eq(x) => v == x,
                                SearchType::Gt(x) => v > x,
                                SearchType::Ge(x) => v >= x,
                                SearchType::Lt(x) => v < x,
                                SearchType::Le(x) => v <= x,
                                SearchType::Bte(a, b) => v >= a && v <= b,
                                SearchType::Bter(a, b) => v >= a && v < b,
                                SearchType::Btel(a, b) => v > a && v <= b,
                            };
                            if hit {
                                let pos = offset;
                                offset += 1;
                                return Some(pos * SIZE);
                            }
                            offset += 1;
                        }

                        None
                    })
                }
            }
        )*
    }
}

search_rule! { f32, f64, u8, u16, u32, u64, usize, i8, i16, i32, i64, isize }