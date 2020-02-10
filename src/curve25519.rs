use std::ptr::null;

const KEY_SIZE: u32 = 32;
const P25: i64 = 33554431;
const P26: i64 = 67108863;
const PRIME: [u8; 32] = [237, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 127];
const ORDER: [u8; 32] = [237, 211, 245, 92, 26, 99, 18, 88, 214, 156, 247, 162, 222, 249, 222, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16];
const ORDER_TIMES_8: [u8; 32] = [104, 159, 174, 231, 210, 24, 147, 192, 178, 230, 188, 23, 245, 206, 247, 166, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128];
type Long10 = [i64; 10];
const BASE_R2Y: Long10 = [5744, 8160848, 4790893, 13779497, 35730846, 12541209, 49101323, 30047407, 40071253, 6226132];
const BASE_2Y: Long10 = [39999547, 18689728, 59995525, 1648697, 57546132, 24010086, 19059592, 5425144, 63499247, 16420658];

fn clamp(k: &[u8]) {
    k[31] &= 0x7F;
    k[31] |= 0x40;
    k[0] &= 0xF8;
}

pub fn keygen(p: &mut [u8], s: Option<&mut [u8]>, k: &[u8]) {
    clamp(k);
    core(p, s, k, None);
}

fn mulaSmall(p: &mut [u8], q: &[u8], m: i64, x: &[u8], n: i64, z: i64) -> i64 {
    let mut v = 0;
    for i in 0..n {
        v += (q[(i+m) as usize] as i64) + z*(x[i as usize] as i64);
        p[(i+m) as usize] = v as u8;
        v >>= 8
    }
    return v
}

fn mula32(p: &mut [u8], x: &[u8], y: &[u8], t: i64, z: i64) {
    let n: i64 = 31;
    let mut w: i64 = 0;
    for i in 0..t {
        let zy = z * (y[i as usize] as i64);
        w += mulaSmall(p, p, i, x, n, zy) + (p[(i+n) as usize] as i64) + zy*(x[n as usize] as i64);
        p[(i+n) as usize] = (w) as u8;
        w >>= 8;
    }
    p[(t+n-1) as usize] = (w + (p[(t+n-1) as usize] as i64)) as u8;
}

fn divmod(q: &mut [u8], r: &mut [u8], n: i64, d: &[u8], t: i64) {
    let mut rn = 0;
    let dt = (d[(t-1) as usize] as i64) << 8;
    if t > 1 {
        dt |= d[(t-2) as usize] as i64;
    }
    n -= 1;
    while n >= t-1 {
        let mut z = (rn << 16) | ((r[n as usize] as i64) << 8);
        if n > 0 {
            z |= r[(n-1) as usize] as i64;
        }
        z /= dt;
        rn += mulaSmall(r, r, n-t+1, d, t, -z);
        q[(n-t+1) as usize] = (z + rn) as u8;
        mulaSmall(r, r, n-t+1, d, t, -rn);
        rn = (r[n as usize] as i64);
        r[n as usize] = 0;
        n -= 1;
    }
    r[(t-1) as usize] = rn as u8;
}

fn unpack(x: &mut Long10, m: &[u8]) {
    x[0] = (m[0] as i64) | ((m[1] as i64) << 8) | ((m[2] as i64) << 16) | (((m[3] as i64) & 3) << 24);
    x[1] = (((m[3] as i64) & !3) >> 2) | ((m[4] as i64) << 6) | ((m[5] as i64) << 14) | ((((m[6] as i64) & 0xFF) & 7) << 22);
    x[2] = (((m[6] as i64) & !7) >> 3) | ((m[7] as i64) << 5) | ((m[8] as i64) << 13) | (((m[9] as i64) & 31) << 21);
    x[3] = (((m[9] as i64) & !31) >> 5) | ((m[10] as i64) << 3) | (((m[11] as i64) & 0xFF) << 11) | (((m[12] as i64) & 63) << 19);
    x[4] = ((((m[12] as i64) & 0xFF) & !63) >> 6) | ((m[13] as i64) << 2) | ((m[14] as i64) << 10) | ((m[15] as i64) << 18);
    x[5] = (m[16] as i64) | ((m[17] as i64) << 8) | ((m[18] as i64) << 16) | (((m[19] as i64) & 1) << 24);
    x[6] = ((((m[19] as i64) & 0xFF) & !1) >> 1) | ((m[20] as i64) << 7) | ((m[21] as i64) << 15) | (((m[22] as i64) & 7) << 23);
    x[7] = (((m[22] as i64) & !7) >> 3) | ((m[23] as i64) << 5) | ((m[24] as i64) << 13) | (((m[25] as i64) & 15) << 21);
    x[8] = (((m[25] as i64) & !15) >> 4) | ((m[26] as i64) << 4) | ((m[27] as i64) << 12) | (((m[28] as i64) & 63) << 20);
    x[9] = (((m[28] as i64) & !63) >> 6) | ((m[29] as i64) << 2) | ((m[30] as i64) << 10) | ((m[31] as i64) << 18);
}

fn isOverflow(x: &Long10) -> bool {
    return ((x[0] > P26 -19) && ((x[1] & x[3] & x[5] & x[7] & x[9]) == P25) && ((x[2] & x[4] & x[6] & x[8]) == P26)) || (x[9] > P25);
}

fn pack(x: &Long10, m: &mut [u8]) {
    let mut ld: i64 = 0;
    let mut ud: i64 = 0;
    let mut t: i64 = 0;

    if isOverflow(x) {
        ld = 1;
    }
    if x[9] < 0 {
        ld -= 1;
    }

    ud = ld * -(P25 + 1);
    ld *= 19;
    t = (ld as i64) + x[0] + (x[1] << 26);
    m[0] = (t) as u8;
    m[1] = (t >> 8) as u8;
    m[2] = (t >> 16) as u8;
    m[3] = (t >> 24) as u8;
    t = (t >> 32) + (x[2] << 19);
    m[4] = (t) as u8;
    m[5] = (t >> 8) as u8;
    m[6] = (t >> 16) as u8;
    m[7] = (t >> 24) as u8;
    t = (t >> 32) + (x[3] << 13);
    m[8] = (t) as u8;
    m[9] = (t >> 8) as u8;
    m[10] = (t >> 16) as u8;
    m[11] = (t >> 24) as u8;
    t = (t >> 32) + (x[4] << 6);
    m[12] = (t) as u8;
    m[13] = (t >> 8) as u8;
    m[14] = (t >> 16) as u8;
    m[15] = (t >> 24) as u8;
    t = (t >> 32) + x[5] + (x[6] << 25);
    m[16] = (t) as u8;
    m[17] = (t >> 8) as u8;
    m[18] = (t >> 16) as u8;
    m[19] = (t >> 24) as u8;
    t = (t >> 32) + (x[7] << 19);
    m[20] = (t) as u8;
    m[21] = (t >> 8) as u8;
    m[22] = (t >> 16) as u8;
    m[23] = (t >> 24) as u8;
    t = (t >> 32) + (x[8] << 12);
    m[24] = (t) as u8;
    m[25] = (t >> 8) as u8;
    m[26] = (t >> 16) as u8;
    m[27] = (t >> 24) as u8;
    t = (t >> 32) + ((x[9] + (ud as i64)) << 6);
    m[28] = (t) as u8;
    m[29] = (t >> 8) as u8;
    m[30] = (t >> 16) as u8;
    m[31] = (t >> 24) as u8;
}

fn cpy(out: &mut Long10, input: &Long10) { // todo memcpy
    out[0] = input[0];
    out[1] = input[1];
    out[2] = input[2];
    out[3] = input[3];
    out[4] = input[4];
    out[5] = input[5];
    out[6] = input[6];
    out[7] = input[7];
    out[8] = input[8];
    out[9] = input[9];
}

fn set(out: &mut Long10, input: i64) {
    out[0] = input;
    out[1] = 0;
    out[2] = 0;
    out[3] = 0;
    out[4] = 0;
    out[5] = 0;
    out[6] = 0;
    out[7] = 0;
    out[8] = 0;
    out[9] = 0;
}

fn add(xy: &mut Long10, x: &Long10, y: &Long10) {
    xy[0] = x[0] + y[0];
    xy[1] = x[1] + y[1];
    xy[2] = x[2] + y[2];
    xy[3] = x[3] + y[3];
    xy[4] = x[4] + y[4];
    xy[5] = x[5] + y[5];
    xy[6] = x[6] + y[6];
    xy[7] = x[7] + y[7];
    xy[8] = x[8] + y[8];
    xy[9] = x[9] + y[9];
}

fn sub(xy: &mut Long10, x: &Long10, y: &Long10) {
    xy[0] = x[0] - y[0];
    xy[1] = x[1] - y[1];
    xy[2] = x[2] - y[2];
    xy[3] = x[3] - y[3];
    xy[4] = x[4] - y[4];
    xy[5] = x[5] - y[5];
    xy[6] = x[6] - y[6];
    xy[7] = x[7] - y[7];
    xy[8] = x[8] - y[8];
    xy[9] = x[9] - y[9];
}

fn mulSmall(xy: &mut Long10, x: &Long10, y: i64) {
    let mut t = (x[8] * y);
    xy[8] = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x[9] * y);
    xy[9] = (t & ((1 << 25) - 1));
    t = 19*(t>>25) + (x[0] * y);
    xy[0] = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x[1] * y);
    xy[1] = (t & ((1 << 25) - 1));
    t = (t >> 25) + (x[2] * y);
    xy[2] = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x[3] * y);
    xy[3] = (t & ((1 << 25) - 1));
    t = (t >> 25) + (x[4] * y);
    xy[4] = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x[5] * y);
    xy[5] = (t & ((1 << 25) - 1));
    t = (t >> 25) + (x[6] * y);
    xy[6] = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x[7] * y);
    xy[7] = (t & ((1 << 25) - 1));
    t = (t >> 25) + xy[8];
    xy[8] = (t & ((1 << 26) - 1));
    xy[9] += (t >> 26);
}

fn mul(xy: &mut Long10, x: &Long10, y: &Long10) {
    let mut t = (x[0] * y[8]) + (x[2] * y[6]) + (x[4] * y[4]) + (x[6] * y[2]) + (x[8] * y[0]) + 2*((x[1]*y[7])+(x[3]*y[5])+(x[5]*y[3])+(x[7]*y[1])) + 38*(x[9]*y[9]);
    xy[8] = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x[0] * y[9]) + (x[1] * y[8]) + (x[2] * y[7]) + (x[3] * y[6]) + (x[4] * y[5]) + (x[5] * y[4]) + (x[6] * y[3]) + (x[7] * y[2]) + (x[8] * y[1]) + (x[9] * y[0]);
    xy[9] = (t & ((1 << 25) - 1));
    t = (x[0] * y[0]) + 19*((t>>25)+(x[2]*y[8])+(x[4]*y[6])+(x[6]*y[4])+(x[8]*y[2])) + 38*((x[1]*y[9])+(x[3]*y[7])+(x[5]*y[5])+(x[7]*y[3])+(x[9]*y[1]));
    xy[0] = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x[0] * y[1]) + (x[1] * y[0]) + 19*((x[2]*y[9])+(x[3]*y[8])+(x[4]*y[7])+(x[5]*y[6])+(x[6]*y[5])+(x[7]*y[4])+(x[8]*y[3])+(x[9]*y[2]));
    xy[1] = (t & ((1 << 25) - 1));
    t = (t >> 25) + (x[0] * y[2]) + (x[2] * y[0]) + 19*((x[4]*y[8])+(x[6]*y[6])+(x[8]*y[4])) + 2*(x[1]*y[1]) + 38*((x[3]*y[9])+(x[5]*y[7])+(x[7]*y[5])+(x[9]*y[3]));
    xy[2] = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x[0] * y[3]) + (x[1] * y[2]) + (x[2] * y[1]) + (x[3] * y[0]) + 19*((x[4]*y[9])+(x[5]*y[8])+(x[6]*y[7])+(x[7]*y[6])+(x[8]*y[5])+(x[9]*y[4]));
    xy[3] = (t & ((1 << 25) - 1));
    t = (t >> 25) + (x[0] * y[4]) + (x[2] * y[2]) + (x[4] * y[0]) + 19*((x[6]*y[8])+(x[8]*y[6])) + 2*((x[1]*y[3])+(x[3]*y[1])) + 38*((x[5]*y[9])+(x[7]*y[7])+(x[9]*y[5]));
    xy[4] = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x[0] * y[5]) + (x[1] * y[4]) + (x[2] * y[3]) + (x[3] * y[2]) + (x[4] * y[1]) + (x[5] * y[0]) + 19*((x[6]*y[9])+(x[7]*y[8])+(x[8]*y[7])+(x[9]*y[6]));
    xy[5] = (t & ((1 << 25) - 1));
    t = (t >> 25) + (x[0] * y[6]) + (x[2] * y[4]) + (x[4] * y[2]) + (x[6] * y[0]) + 19*(x[8]*y[8]) + 2*((x[1]*y[5])+(x[3]*y[3])+(x[5]*y[1])) + 38*((x[7]*y[9])+(x[9]*y[7]));
    xy[6] = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x[0] * y[7]) + (x[1] * y[6]) + (x[2] * y[5]) + (x[3] * y[4]) + (x[4] * y[3]) + (x[5] * y[2]) + (x[6] * y[1]) + (x[7] * y[0]) + 19*((x[8]*y[9])+(x[9]*y[8]));
    xy[7] = (t & ((1 << 25) - 1));
    t = (t >> 25) + xy[8];
    xy[8] = (t & ((1 << 26) - 1));
    xy[9] += (t >> 26);
}

fn sqr(x2: &mut Long10, x: &Long10) {
    let mut t = (x[4] * x[4]) + 2*((x[0]*x[8])+(x[2]*x[6])) + 38*(x[9]*x[9]) + 4*((x[1]*x[7])+(x[3]*x[5]));
    x2[8] = (t & ((1 << 26) - 1));
    t = (t >> 26) + 2*((x[0]*x[9])+(x[1]*x[8])+(x[2]*x[7])+(x[3]*x[6])+(x[4]*x[5]));
    x2[9] = (t & ((1 << 25) - 1));
    t = 19*(t>>25) + (x[0] * x[0]) + 38*((x[2]*x[8])+(x[4]*x[6])+(x[5]*x[5])) + 76*((x[1]*x[9])+(x[3]*x[7]));
    x2[0] = (t & ((1 << 26) - 1));
    t = (t >> 26) + 2*(x[0]*x[1]) + 38*((x[2]*x[9])+(x[3]*x[8])+(x[4]*x[7])+(x[5]*x[6]));
    x2[1] = (t & ((1 << 25) - 1));
    t = (t >> 25) + 19*(x[6]*x[6]) + 2*((x[0]*x[2])+(x[1]*x[1])) + 38*(x[4]*x[8]) + 76*((x[3]*x[9])+(x[5]*x[7]));
    x2[2] = (t & ((1 << 26) - 1));
    t = (t >> 26) + 2*((x[0]*x[3])+(x[1]*x[2])) + 38*((x[4]*x[9])+(x[5]*x[8])+(x[6]*x[7]));
    x2[3] = (t & ((1 << 25) - 1));
    t = (t >> 25) + (x[2] * x[2]) + 2*(x[0]*x[4]) + 38*((x[6]*x[8])+(x[7]*x[7])) + 4*(x[1]*x[3]) + 76*(x[5]*x[9]);
    x2[4] = (t & ((1 << 26) - 1));
    t = (t >> 26) + 2*((x[0]*x[5])+(x[1]*x[4])+(x[2]*x[3])) + 38*((x[6]*x[9])+(x[7]*x[8]));
    x2[5] = (t & ((1 << 25) - 1));
    t = (t >> 25) + 19*(x[8]*x[8]) + 2*((x[0]*x[6])+(x[2]*x[4])+(x[3]*x[3])) + 4*(x[1]*x[5]) + 76*(x[7]*x[9]);
    x2[6] = (t & ((1 << 26) - 1));
    t = (t >> 26) + 2*((x[0]*x[7])+(x[1]*x[6])+(x[2]*x[5])+(x[3]*x[4])) + 38*(x[8]*x[9]);
    x2[7] = (t & ((1 << 25) - 1));
    t = (t >> 25) + x2[8];
    x2[8] = (t & ((1 << 26) - 1));
    x2[9] += (t >> 26);
}

fn recip(y: &mut Long10, x: &Long10, sqrtassist: i64) {
    let mut t0: Long10 = [0; 10];
    let mut t1: Long10 = [0; 10];
    let mut t2: Long10 = [0; 10];
    let mut t3: Long10 = [0; 10];
    let mut t4: Long10 = [0; 10];
    /* the chain for x^(2^255-21) is straight from djb's implementation */
    sqr(&mut t1, x);        /*  2 == 2 * 1  */
    sqr(&mut t2, &t1);      /*  4 == 2 * 2  */
    sqr(&mut t0, &t2);      /*  8 == 2 * 4  */
    mul(&mut t2, &t0, x);   /*  9 == 8 + 1  */
    mul(&mut t0, &t2, &t1); /* 11 == 9 + 2  */
    sqr(&mut t1, &t0);      /* 22 == 2 * 11 */
    mul(&mut t3, &t1, &t2); /* 31 == 22 + 9
	   == 2^5   - 2^0  */
    sqr(&mut t1, &t3);      /* 2^6   - 2^1  */
    sqr(&mut t2, &t1);      /* 2^7   - 2^2  */
    sqr(&mut t1, &t2);      /* 2^8   - 2^3  */
    sqr(&mut t2, &t1);      /* 2^9   - 2^4  */
    sqr(&mut t1, &t2);      /* 2^10  - 2^5  */
    mul(&mut t2, &t1, &t3); /* 2^10  - 2^0  */
    sqr(&mut t1, &t2);      /* 2^11  - 2^1  */
    sqr(&mut t3, &t1);      /* 2^12  - 2^2  */
    for _ in 0..4 {
        sqr(&mut t1, &t3);
        sqr(&mut t3, &t1);
    } /* &t3 */ /* 2^20  - 2^10 */
    mul(&mut t1, &t3, &t2); /* 2^20  - 2^0  */
    sqr(&mut t3, &t1);      /* 2^21  - 2^1  */
    sqr(&mut t4, &t3);      /* 2^22  - 2^2  */
    for _ in 0..9 {
        sqr(&mut t3, &t4);
        sqr(&mut t4, &t3);
    } /* &t4 */ /* 2^40  - 2^20 */
    mul(&mut t3, &t4, &t1); /* 2^40  - 2^0  */
    for _ in 0..5 {
        sqr(&mut t1, &t3);
        sqr(&mut t3, &t1);
    } /* &t3 */ /* 2^50  - 2^10 */
    mul(&mut t1, &t3, &t2); /* 2^50  - 2^0  */
    sqr(&mut t2, &t1);      /* 2^51  - 2^1  */
    sqr(&mut t3, &t2);      /* 2^52  - 2^2  */
    for _ in 0..24 {
        sqr(&mut t2, &t3);
        sqr(&mut t3, &t2);
    } /* &t3 */ /* 2^100 - 2^50 */
    mul(&mut t2, &t3, &t1); /* 2^100 - 2^0  */
    sqr(&mut t3, &t2);      /* 2^101 - 2^1  */
    sqr(&mut t4, &t3);      /* 2^102 - 2^2  */
    for _ in 0..49 {
        sqr(&mut t3, &t4);
        sqr(&mut t4, &t3);
    } /* &t4 */ /* 2^200 - 2^100 */
    mul(&mut t3, &t4, &t2); /* 2^200 - 2^0  */
    for _ in 0..25 {
        sqr(&mut t4, &t3);
        sqr(&mut t3, &t4);
    } /* &t3 */ /* 2^250 - 2^50 */
    mul(&mut t2, &t3, &t1); /* 2^250 - 2^0  */
    sqr(&mut t1, &t2);      /* 2^251 - 2^1  */
    sqr(&mut t2, &t1);      /* 2^252 - 2^2  */
    if sqrtassist != 0 {
        mul(y, x, &t2) /* 2^252 - 3 */
    } else {
        sqr(&mut t1, &t2);    /* 2^253 - 2^3  */
        sqr(&mut t2, &t1);    /* 2^254 - 2^4  */
        sqr(&mut t1, &t2);    /* 2^255 - 2^5  */
        mul(y, &t1, &t0) /* 2^255 - 21 */
    }
}

fn isNegative(x: &Long10) -> i64 { // todo obvious optimization
    let mut tmp = 0;
    if isOverflow(x) | (x[9] < 0) {
        tmp = 1;
    }
    return tmp ^ (x[0] & 1 as i64);
}

fn sqrt(x: &mut Long10, u: &Long10) {
    let mut v: Long10 = [0; 10];
    let mut t1: Long10 = [0; 10];
    let mut t2: Long10 = [0; 10];
    add(&mut t1, u, u);    /* t1 = 2u    */
    recip(&mut v, &t1, 1); /* v = (2u)^((p-5)/8) */
    sqr(x, &v);        /* x = v^2    */
    mul(&mut t2, &t1, x);  /* t2 = 2uv^2   */
    t2[0] -= 1;           /* t2 = 2uv^2-1   */
    mul(&mut t1, &v, &t2); /* t1 = v(2uv^2-1)  */
    mul(x, u, &t1);    /* x = uv(2uv^2-1)  */
}

fn montPrep(t1: &mut Long10, t2: &mut Long10, ax: &Long10, az: &Long10) {
    add(t1, ax, az);
    sub(t2, ax, az);
}

fn montAdd(t1: &mut Long10, t2: &mut Long10, t3: &Long10, t4: &Long10, ax: &mut Long10, az: &mut Long10, dx: &Long10) {
    mul(ax, t2, t3);
    mul(az, t1, t4);
    add(t1, ax, az);
    sub(t2, ax, az);
    sqr(ax, t1);
    sqr(t1, t2);
    mul(az, t1, dx);
}

fn montDbl(t1: &mut Long10, t2: &mut Long10, t3: &Long10, t4: &Long10, bx: &mut Long10, bz: &mut Long10) {
    sqr(t1, t3);
    sqr(t2, t4);
    mul(bx, t1, t2);
    sub(t2, t1, t2);
    mulSmall(bz, t2, 121665);
    add(t1, t1, bz);
    mul(bz, t1, t2);
}

fn xToY2(t: &mut Long10, y2: &mut Long10, x: &Long10) {
    sqr(t, x);
    mulSmall(y2, x, 486662);
    add(t, t, y2);
    t[0] += 1;
    mul(y2, t, x);
}

fn core(px: &mut [u8], s: Option<&mut [u8]>, k: &[u8], gx: Option<&[u8]>) {
    let mut dx: Long10 = [0; 10];
    let mut t1: Long10 = [0; 10];
    let mut t2: Long10 = [0; 10];
    let mut t3: Long10 = [0; 10];
    let mut t4: Long10 = [0; 10];
    let mut x: [Long10; 2] = [[0; 10], [0; 10]];
    let mut z: [Long10; 2] = [[0; 10], [0; 10]];

    /* unpack the base */
    if gx.is_some() {
        unpack(&mut dx, gx.unwrap());
    } else {
        set(&mut dx, 9);
    }

    /* 0G = point-at-infinity */
    set(&mut x[0], 1);
    set(&mut z[0], 0);

    /* 1G = G */
    cpy(&mut x[1], &dx);
    set(&mut z[1], 1);

    for i in (1..32).rev() {
        for j in (1..8).rev() {
            let bit1 = (k[i as usize] as u64) >> (j as u64) & 1;
            let bit0 = !(k[i as usize] as u64) >> (j as u64) & 1;
            let ax = &mut x[bit0 as usize];
            let az = &mut z[bit0 as usize];
            let bx = &mut x[bit1 as usize];
            let bz = &mut z[bit1 as usize];

            /* a' = a + b */
            /* b' = 2 b */
            montPrep(&mut t1, &mut t2, ax, az);
            montPrep(&mut t3, &mut t4, bx, bz);
            montAdd(&mut t1, &mut t2, &t3, &t4, ax, az, &dx);
            montDbl(&mut t1, &mut t2, &t3, &t4, bx, bz);
        }
    }

    recip(&mut t1, &mut z[0], 0);
    mul(&mut dx, &mut x[0], &t1);
    pack(&dx, px);

    /* calculate s such that s abs(P) = G  .. assumes G is std base point */
    if s.is_some() {
        let s_unwrapped = s.unwrap();
        xToY2(&mut t2, &mut t1, &dx);      /* t1 = Py^2  */
        recip(&mut t3, &z[1], 0);       /* where Q=P+G ... */
        mul(&mut t2, &x[1], &t3);       /* t2 = Qx  */
        add(&mut t2, &t2, &dx);        /* t2 = Qx + Px  */
        t2[0] += 9 + 486662;       /* t2 = Qx + Px + Gx + 486662  */
        dx[0] -= 9;                /* dx = Px - Gx  */
        sqr(&mut t3, &dx);             /* t3 = (Px - Gx)^2  */
        mul(&mut dx, &t2, &t3);        /* dx = t2 (Px - Gx)^2  */
        sub(&mut dx, &dx, &t1);        /* dx = t2 (Px - Gx)^2 - Py^2  */
        dx[0] -= 39420360;         /* dx = t2 (Px - Gx)^2 - Py^2 - Gy^2  */
        mul(&mut t1, &dx, &BASE_R2Y);    /* t1 = -Py  */
        if isNegative(&t1) != 0 { /* sign is 1, so just copy  */
            s_unwrapped.copy_from_slice(k);
        } else { /* sign is -1, so negate  */
            mulaSmall(s_unwrapped, &ORDER_TIMES_8, 0, k, 32, -1);
        }

        /* reduce s mod q
         * (is this needed?  do it just in case, it's fast anyway) */
//divmod((dstptr) t1, s, 32, order25519, 32);

        /* take reciprocal of s mod q */
        let mut tmp1: [u8; 32] = [0; 32];
        let mut tmp2: [u8; 64] = [0; 64];
        let mut tmp3: [u8; 64] = [0; 64];
        tmp1.copy_from_slice(&ORDER);
        egcd32(s_unwrapped, &mut tmp2, &mut tmp3, s_unwrapped, &mut tmp1);
        if (s_unwrapped[31] & 0x80) != 0 {
            mulaSmall(s_unwrapped, s_unwrapped, 0, &ORDER, 32, 1);
        }
    }
}

pub fn Sign(v: &mut [u8], h: &[u8], x: &[u8], s: &[u8]) -> bool {
    let w = 0;
    let mut h1: [u8; 32] = [0; 32];
    let mut x1: [u8; 32] = [0; 32];
    let mut tmp3: [u8; 32] = [0; 32];
    let mut tmp1: [u8; 64] = [0; 64];
    let mut tmp2: [u8; 64] = [0; 64];
    h1.copy_from_slice(&h);
    x1.copy_from_slice(&x);

    divmod(&mut tmp3, &mut h1, 32, &ORDER, 32);
    divmod(&mut tmp3, &mut x1, 32, &ORDER, 32);

    mulaSmall(v, &x1, 0, &h1, 32, -1);
    mulaSmall(v, v, 0, &ORDER, 32, 1);

    mula32(&mut tmp1, v, s, 32, 1);
    divmod(&mut tmp2, &mut tmp1, 64, &ORDER, 32);

    for i in 0..32 {
        v[i as usize] = tmp1[i as usize];
        w |= (tmp1[i as usize] as i64);
    }
    return w != 0
}

pub fn Verify(Y: &mut [u8], v: &[u8], h: &[u8], P: &[u8]) {
    /* Y = v abs(P) + h G  */
    let mut d: [u8; 32] = [0; 32];
    let mut p: [Long10; 2] = [[0; 10], [0; 10]];
    let mut s: [Long10; 2] = [[0; 10], [0; 10]];
    let mut yx: [Long10; 3] = [[0; 10], [0; 10], [0; 10]];
    let mut yz: [Long10; 3] = [[0; 10], [0; 10], [0; 10]];
    let mut t1: [Long10; 3] = [[0; 10], [0; 10], [0; 10]];
    let mut t2: [Long10; 3] = [[0; 10], [0; 10], [0; 10]];
    let mut vi: i64 = 0;
    let mut hi: i64 = 0;
    let mut di: i64 = 0;
    let mut nvh: i64 = 0;
    let mut j: i64 = 0;
    let mut k: i64 = 0;

    /* set p[0] to G and p[1] to P  */
    set(&mut p[0], 9);
    unpack(&mut p[1], P);

    /* set s[0] to P+G and s[1] to P-G  */

    /* s[0] = (Py^2 + Gy^2 - 2 Py Gy)/(Px - Gx)^2 - Px - Gx - 486662  */
    /* s[1] = (Py^2 + Gy^2 + 2 Py Gy)/(Px - Gx)^2 - Px - Gx - 486662  */

    xToY2(&mut t1[0], &mut t2[0], &p[1]);  /* t2[0] = Py^2  */
    sqrt(&mut t1[0], &t2[0]);         /* t1[0] = Py or -Py  */
    j = isNegative(&t1[0]);      /*      ... check which  */
    t2[0][0] += 39420360;       /* t2[0] = Py^2 + Gy^2  */
    mul(&mut t2[1], &BASE_2Y, &t1[0]);  /* t2[1] = 2 Py Gy or -2 Py Gy  */
    sub(&mut t1[j as usize], &t2[0], &t2[1]);   /* t1[0] = Py^2 + Gy^2 - 2 Py Gy  */
    add(&mut t1[(1-j) as usize], &t2[0], &t2[1]); /* t1[1] = Py^2 + Gy^2 + 2 Py Gy  */
    cpy(&mut t2[0], &p[1]);           /* t2[0] = Px  */
    t2[0][0] -= 9;              /* t2[0] = Px - Gx  */
    sqr(&mut t2[1], &t2[0]);          /* t2[1] = (Px - Gx)^2  */
    recip(&mut t2[0], &t2[1], 0);     /* t2[0] = 1/(Px - Gx)^2  */
    mul(&mut s[0], &t1[0], &t2[0]);    /* s[0] = t1[0]/(Px - Gx)^2  */
    sub(&mut s[0], &s[0], &p[1]);      /* s[0] = t1[0]/(Px - Gx)^2 - Px  */
    s[0][0] -= 9 + 486662;      /* s[0] = X(P+G)  */
    mul(&mut s[1], &t1[1], &t2[0]);    /* s[1] = t1[1]/(Px - Gx)^2  */
    sub(&mut s[1], &s[1], &p[1]);      /* s[1] = t1[1]/(Px - Gx)^2 - Px  */
    s[1][0] -= 9 + 486662;      /* s[1] = X(P-G)  */
    mulSmall(&mut s[0], &s[0], 1);    /* reduce s[0] */
    mulSmall(&mut s[1], &s[1], 1);    /* reduce s[1] */

    /* prepare the chain  */
    for i in 0..32 {
        vi = (vi >> 8) ^ (v[i as usize] as i64) ^ ((v[i as usize] as i64) << 1);
        hi = (hi >> 8) ^ (h[i as usize] as i64) ^ ((h[i as usize] as i64) << 1);
        nvh = !(vi ^ hi);
        di = (nvh & ((di & 0x80) >> 7)) ^ vi;
        di ^= nvh & ((di & 0x01) << 1);
        di ^= nvh & ((di & 0x02) << 1);
        di ^= nvh & ((di & 0x04) << 1);
        di ^= nvh & ((di & 0x08) << 1);
        di ^= nvh & ((di & 0x10) << 1);
        di ^= nvh & ((di & 0x20) << 1);
        di ^= nvh & ((di & 0x40) << 1);
        d[i as usize] = (di) as u8;
    }

    di = ((nvh & ((di & 0x80) << 1)) ^ vi) >> 8;

    /* initialize state */
    set(&mut yx[0], 1);
    cpy(&mut yx[1], &p[di as usize]);
    cpy(&mut yx[2], &s[0]);
    set(&mut yz[0], 0);
    set(&mut yz[1], 1);
    set(&mut yz[2], 1);

    /* y[0] is (even)P + (even)G
     * y[1] is (even)P + (odd)G  if current d-bit is 0
     * y[1] is (odd)P + (even)G  if current d-bit is 1
     * y[2] is (odd)P + (odd)G
     */

    vi = 0;
    hi = 0;

    /* and go for it! */
    for i in (1..32).rev() {
        vi = (vi << 8) | (v[i as usize] as i64);
        hi = (hi << 8) | (h[i as usize] as i64);
        di = (di << 8) | (d[i as usize] as i64);

        for i in (1..8).rev() {
            montPrep(&mut t1[0], &mut t2[0], &yx[0], &yz[0]);
            montPrep(&mut t1[1], &mut t2[1], &yx[1], &yz[1]);
            montPrep(&mut t1[2], &mut t2[2], &yx[2], &yz[2]);

//            k = ((vi ^ vi>>1) >> (j as u64) & 1) + ((hi ^ hi>>1) >> (j as u64) & 1);
//            montDbl(&mut yx[2], &mut yz[2], &t1[k as usize], &t2[k as usize], &mut yx[0], &mut yz[0]);
//            k = (di >> (j as u64) & 2) ^ ((di >> (j as u64) & 1) << 1);
//            montAdd(&mut t1[1], &mut t2[1], &t1[k as usize], &t2[k as usize], &mut yx[1], &mut yz[1], &p[(di>>j&1) as usize]);
//            montAdd(&mut t1[2], &mut t2[2], &t1[0], &t2[0], &mut yx[2], &mut yz[2], &s[(((vi^hi)>>j&2)>>1) as usize]);

            k = ((vi ^ vi>>1) >> j & 1) + ((hi ^ hi>>1) >> j & 1);
            montDbl(&mut yx[2], &mut yz[2], &t1[k as usize], &t2[k as usize], &mut yx[0], &mut yz[0]);
            k = (di >> j & 2) ^ ((di >> j & 1) << 1);
            montAdd(&mut t1[1], &mut t2[1], &t1[k as usize], &t2[k as usize], &mut yx[1], &mut yz[1], &p[(di>>j&1) as usize]);
            montAdd(&mut t1[2], &mut t2[2], &t1[0], &t2[0], &mut yx[2], &mut yz[2], &s[(((vi^hi)>>j&2)>>1) as usize]);
        }
    }

    k = (vi & 1) + (hi & 1);
    recip(&mut t1[0], &yz[k as usize], 0);
    mul(&mut t1[1], &yx[k as usize], &t1[0]);

    pack(&mut t1[1], Y);
}

pub fn IsCanonicalSignature(v: &[u8]) -> bool {
    let mut v_copy: [u8; 32] = [0; 32];
    v_copy.copy_from_slice(v);
    let mut tmp: [u8; 32] = [0; 32];
    divmod(&mut tmp, &mut v_copy, 32, &ORDER, 32);
    for i in 0..32 {
        if v[i as usize] != v_copy[i as usize] {
            return false
        }
    }
    return true
}

pub fn IsCanonicalPublicKey(public_key: &[u8]) -> bool {
    let mut public_key_unpacked: Long10 = [0; 10];
    unpack(&mut public_key_unpacked, public_key);
    let mut public_key_copy: [u8; 32] = [0; 32];
    pack(&public_key_unpacked, &mut public_key_copy);
    for i in 0..32 {
        if public_key_copy[i as usize] != public_key[i as usize] {
            return false
        }
    }
    return true
}

fn numsize(x: &[u8], n: i64) -> i64 {
    let mut i = n - 1;
    while i != -1 {
        if x[i as usize] != 0 {
            return i + 1
        }
        i -= 1;
    }
    return 0
}

fn egcd32(dest: &mut [u8], x: &mut [u8], y: &mut [u8], a: &mut [u8], b: &mut [u8]) {
    let mut an: i64 = 0;
    let mut qn: i64 = 0;
    let mut bn: i64 = 32;

    // TODO efficient zero
    for i in 0..32 {
        x[i as usize] = 0;
        y[i as usize] = 0;
    }

    x[0] = 1;
    an = numsize(a, 32);
    if an == 0 {
        dest.copy_from_slice(y);
        return;
    }

    let mut tmp: [u8; 32] = [0; 32];
    loop {
        qn = bn - an + 1;
        divmod(&mut tmp, b, bn, a, an);
        bn = numsize(b, bn);
        if bn == 0 {
            dest.copy_from_slice(x);
            return;
        }
        mula32(y, x, &tmp, qn, -1);

        qn = an - bn + 1;
        divmod(&mut tmp, a, an, b, bn);
        an = numsize(a, an);
        if an == 0 {
            dest.copy_from_slice(y);
            return;
        }
        mula32(x, y, &tmp, qn, -1);
    }
}
