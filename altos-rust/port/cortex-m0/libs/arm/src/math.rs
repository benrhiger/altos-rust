/*
 * Copyright (C) 2017 AltOS-Rust Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

#[no_mangle]
pub extern "C" fn __aeabi_lmul(a: u64, b: u64) -> u64 {
    let half_bits: u32 = 64 / 4;
    let lower_mask = !0 >> half_bits;
    let mut low = ((a as u32) & lower_mask).wrapping_mul((b as u32) & lower_mask);
    let mut t = low >> half_bits;
    low &= lower_mask;
    t += ((a as u32) >> half_bits).wrapping_mul((b as u32) & lower_mask);
    low += (t & lower_mask) << half_bits;
    let mut high = t >> half_bits;
    t = low >> half_bits;
    low &= lower_mask;
    t += ((b as u32) >> half_bits).wrapping_mul((a as u32) & lower_mask);
    low += (t & lower_mask) << half_bits;
    high += t >> half_bits;
    high += ((a as u32) >> half_bits).wrapping_mul((b as u32) >> half_bits);
    high = high.wrapping_add(((a >> 32) as u32).wrapping_mul((b as u32)).wrapping_add((a as u32).wrapping_mul(((b >> 32) as u32))));
    low as u64 | ((high as u64) << 32)
}

#[no_mangle]
pub extern "C" fn __aeabi_uidiv(num: u32, den: u32) -> u32 {
    __aeabi_uidivbase(num, den, false)
}

#[no_mangle]
pub extern "C" fn __udivmodsi4(mut num: u32, mut den: u32, rem_p: Option<&mut u32>) -> u32 {
    let mut quot = 0;
    let mut qbit = 1;

    if den == 0 {
        return 0;
    }

    /*
     * left-justify denominator and count shift
     */
    while den as i32 >= 0 {
        den <<= 1;
        qbit <<= 1;
    }

    while qbit != 0 {
        if den <= num {
            num -= den;
            quot += qbit;
        }
        den >>= 1;
        qbit >>= 1;
    }

    if let Some(rem) = rem_p {
        *rem = num;
    }

    return quot;
}

#[no_mangle]
#[naked]
pub unsafe fn __aeabi_uidivmod() {
    asm!("push {lr}
          sub sp, sp, #4
          mov r2, sp
          bl __udivmodsi4
          ldr r1, [sp]
          add sp, sp, #4
          pop {pc}"
    );
    ::core::intrinsics::unreachable();
}

fn __aeabi_uidivbase(mut num: u32, mut den: u32, modwanted: bool) -> u32 {
    let mut bit: u32 = 1;
    let mut res: u32 = 0;

    while den < num && bit != 0 && (den & (1<<31)) == 0 {
        den <<= 1;
        bit <<= 1;
    }
    while bit != 0 {
        if num >= den {
            num -= den;
            res |= bit;
        }
        bit >>= 1;
        den >>= 1;
    }
    if modwanted { num } else { res }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divide_even() {
        assert_eq!(10, __aeabi_uidiv(100, 10));
    }

    #[test]
    fn divide_uneven() {
        assert_eq!(10, __aeabi_uidiv(105, 10));
    }

    #[test]
    fn divide_denominator_bigger() {
        assert_eq!(0, __aeabi_uidiv(5, 10));
    }

    #[test]
    fn mod_even() {
        assert_eq!(0, __aeabi_uidivmod(100, 10));
    }

    #[test]
    fn mod_uneven() {
        assert_eq!(5, __aeabi_uidivmod(105, 10));
    }

    #[test]
    fn mod_denominator_bigger() {
        assert_eq!(5, __aeabi_uidivmod(5, 10));
    }

    #[test]
    fn multiply_bigger_first() {
        assert_eq!(100, __aeabi_lmul(20, 5));
    }

    #[test]
    fn multiply_bigger_second() {
        assert_eq!(100, __aeabi_lmul(5, 20));
    }
}
