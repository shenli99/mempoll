use std::{simd::{cmp::{SimdPartialEq, SimdPartialOrd}, LaneCount, Mask, Simd, SupportedLaneCount}, usize};

pub enum SearchError{

}

pub trait MemorySearcher {
    fn search<T: Eq + Sized, const N: usize>(&self, rule: SearchRule<T>) -> Result<Vec<usize>, SearchError>;
}

#[derive(Debug, Clone, Copy)]
pub enum SearchRule<T: Sized> {
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

macro_rules! search_rule {
    { $($number:ty),* } => {
        $(
            impl SearchRule<$number>
            {
                pub fn search<'a, const LANES: usize>(self, data: &'a [u8]) -> impl Iterator<Item = usize> + 'a 
                where
                LaneCount<LANES>: SupportedLaneCount,
                {
                    let mut offset = 0;
                    let mut pending = Vec::new();
                    let rule = self;
                    let size = std::mem::size_of::<$number>();
                    let buff = unsafe { std::slice::from_raw_parts(
                        data.as_ptr() as *const $number,
                        data.len() / std::mem::size_of::<$number>()
                    )};
                    std::iter::from_fn(move || {
                        if let Some(pos) = pending.pop() {
                            return Some(pos);
                        }

                        let simd_val_low;
                        let simd_val_high;
                        let is_range = matches!(rule, SearchRule::Bte(_, _) | SearchRule::Bter(_, _) | SearchRule::Btel(_, _));
                        if is_range {
                            if let SearchRule::Bte(lo, hi)
                            | SearchRule::Bter(lo, hi)
                            | SearchRule::Btel(lo, hi) = rule
                            {
                                simd_val_low = Simd::splat(lo);
                                simd_val_high = Simd::splat(hi);
                            } else {
                                unreachable!();
                            }
                        } else if let SearchRule::Eq(v) = rule {
                            simd_val_low = Simd::splat(v);
                            simd_val_high = Simd::splat(v);
                        } else if let SearchRule::Gt(v) = rule {
                            simd_val_low = Simd::splat(v);
                            simd_val_high = Simd::splat(<$number>::default());
                        } else if let SearchRule::Ge(v) = rule {
                            simd_val_low = Simd::splat(v);
                            simd_val_high = Simd::splat(<$number>::default());
                        } else if let SearchRule::Lt(v) = rule {
                            simd_val_low = Simd::splat(<$number>::default());
                            simd_val_high = Simd::splat(v);
                        } else if let SearchRule::Le(v) = rule {
                            simd_val_low = Simd::splat(<$number>::default());
                            simd_val_high = Simd::splat(v);
                        } else {
                            simd_val_low = Simd::splat(<$number>::default());
                            simd_val_high = Simd::splat(<$number>::default());
                        }

                        while offset + LANES <= buff.len() {
                            let chunk: Simd<$number, LANES> = Simd::from_slice(&buff[offset..]);
                            let mask: Mask<_, LANES> = match rule {
                                SearchRule::Eq(_) => chunk.simd_eq(simd_val_low),
                                SearchRule::Gt(_) => chunk.simd_gt(simd_val_low),
                                SearchRule::Ge(_) => chunk.simd_ge(simd_val_low),
                                SearchRule::Lt(_) => chunk.simd_lt(simd_val_high),
                                SearchRule::Le(_) => chunk.simd_le(simd_val_high),
                                SearchRule::Bte(_, _) => chunk.simd_ge(simd_val_low) & chunk.simd_le(simd_val_high),
                                SearchRule::Bter(_, _) => chunk.simd_ge(simd_val_low) & chunk.simd_lt(simd_val_high),
                                SearchRule::Btel(_, _) => chunk.simd_gt(simd_val_low) & chunk.simd_le(simd_val_high),
                            };
                            let bits = mask.to_bitmask();
                            if bits != 0 {
                                for i in 0..LANES {
                                    if bits & (1 << i) != 0 {
                                        pending.push((offset + i) * size);
                                    }
                                }
                                offset += LANES;
                                return pending.pop();
                            }
                            offset += LANES;
                        }

                        while offset < buff.len() {
                            let v = buff[offset];
                            let hit = match rule {
                                SearchRule::Eq(x) => v == x,
                                SearchRule::Gt(x) => v > x,
                                SearchRule::Ge(x) => v >= x,
                                SearchRule::Lt(x) => v < x,
                                SearchRule::Le(x) => v <= x,
                                SearchRule::Bte(a, b) => v >= a && v <= b,
                                SearchRule::Bter(a, b) => v >= a && v < b,
                                SearchRule::Btel(a, b) => v > a && v <= b,
                            };
                            if hit {
                                let pos = offset;
                                offset += 1;
                                return Some(pos * size);
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