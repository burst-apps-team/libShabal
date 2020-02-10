/* Ported back to C for use with Nxt Platform by mystcoin 01/15
 */
/* Ported from C to Java by Dmitry Skiba [sahn0], 23/02/08.
 * Original: http://cds.xs4all.nl:8081/ecdh/
 */
/* Generic 64-bit integer implementation of Curve25519 ECDH
 * Written by Matthijs van Duin, 200608242056
 * Public domain.
 *
 * Based on work by Daniel J Bernstein, http://cr.yp.to/ecdh.html
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h> // isxdigit

typedef unsigned char BYTE;
typedef long long LL;
typedef struct {LL _0, _1, _2, _3, _4, _5, _6, _7, _8, _9;} LL10;

/* group order (a prime near 2^252+2^124) */
BYTE ORDER[32] = {237, 211, 245, 92, 26,  99,  18,  88, 214, 156, 247, 162, 222, 249, 222, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16};

/* smallest multiple of the order that's >= 2^255 */
BYTE ORDER_TIMES_8[32] = {104, 159, 174, 231, 210, 24,  147, 192, 178, 230, 188, 23, 245, 206, 247, 166, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128};

/* constants 2Gy and 1/(2Gy) */
LL10 BASE_2Y = {39999547, 18689728, 59995525, 1648697, 57546132, 24010086, 19059592, 5425144, 63499247, 16420658};
LL10 BASE_R2Y = {5744, 8160848, 4790893, 13779497, 35730846, 12541209, 49101323, 30047407, 40071253, 6226132};

/********************* radix 2^8 math *********************/

void cpy32(BYTE *d, BYTE *s) {
    int i; for (i = 0; i < 32; i++) d[i] = s[i];
}

/* p[m..n+m-1] = q[m..n+m-1] + z * x */
/* n is the size of x */
/* n+m is the size of p and q */
int mula_small(BYTE *p, BYTE *q, int m, BYTE *x, int n, int z) {
    int v=0;
    int i; for (i = 0; i < n; ++i) {
        v += (q[i + m] & 0xFF) + z * (x[i] & 0xFF);
        p[i + m] = (BYTE)v;
        v >>= 8;
    }
    return v;
}

/* p += x * y * z  where z is a small integer
 * x is size 32, y is size t, p is size 32+t
 * y is allowed to overlap with p+32 if you don't care about the upper half
 */
int mula32(BYTE *p, BYTE *x, BYTE *y, int t, int z) {
    int n = 31;
    int w = 0;
    int i = 0;
    for (; i < t; i++) {
        int zy = z * (y[i] & 0xFF);
        w += mula_small(p, p, i, x, n, zy) + (p[i+n] & 0xFF) + zy * (x[n] & 0xFF);
        p[i+n] = (BYTE)w;
        w >>= 8;
    }
    p[i+n] = (BYTE)(w + (p[i+n] & 0xFF));
    return w >> 8;
}

/* divide r (size n) by d (size t), returning quotient q and remainder r
 * quotient is size n-t+1, remainder is size t
 * requires t > 0 && d[t-1] != 0
 * requires that r[-1] and d[-1] are valid memory locations
 * q may overlap with r+t
 */
void divmod(BYTE *q, BYTE *r, int n, BYTE *d, int t) {
    int rn = 0;
    int dt = ((d[t - 1] & 0xFF) << 8);
    if(t > 1) dt |= (d[t - 2] & 0xFF);
    while (n-- >= t) {
        int z = (rn << 16) | ((r[n] & 0xFF) << 8);
        if(n > 0) z |= (r[n - 1] & 0xFF);
        z /= dt;
        rn += mula_small(r, r, n - t + 1, d, t, -z);
        q[n - t + 1] = (BYTE)((z + rn) & 0xFF); /* rn is 0 or -1 (underflow) */
        mula_small(r, r, n - t + 1, d, t, -rn);
        rn = (r[n] & 0xFF);
        r[n] = 0;
    }
    r[t-1] = (BYTE)rn;
}

int numsize(BYTE *x, int n) {
    while (n--!=0 && x[n]==0);
    return n+1;
}

/* Returns x if a contains the gcd, y if b.
 * Also, the returned buffer contains the inverse of a mod b,
 * as 32-byte signed.
 * x and y must have 64 bytes space for temporary use.
 * requires that a[-1] and b[-1] are valid memory locations
 */
BYTE *egcd32(BYTE *x, BYTE *y, BYTE *a, BYTE *b) {
    int an, bn = 32, qn, i;
    for (i = 0; i < 32; i++) x[i] = y[i] = 0;
    x[0] = 1;
    an = numsize(a, 32);
    if (an==0) return y; /* division by zero */
    BYTE temp[32] = {0};
    while(1) {
        qn = bn - an + 1;
        divmod(temp, b, bn, a, an);
        bn = numsize(b, bn);
        if (bn==0) return x;
        mula32(y, x, temp, qn, -1);

        qn = an - bn + 1;
        divmod(temp, a, an, b, bn);
        an = numsize(a, an);
        if (an==0) return y;
        mula32(x, y, temp, qn, -1);
    }
}

/********************* radix 2^25.5 GF(2^255-19) math *********************/

int P25=33554431; /* (1 << 25) - 1 */
int P26=67108863; /* (1 << 26) - 1 */

/* Check if reduced-form input >= 2^255-19 */
int is_overflow(LL10 *x) {
    return (((x->_0 > P26-19)) && ((x->_1 & x->_3 & x->_5 & x->_7 & x->_9) == P25) && ((x->_2 & x->_4 & x->_6 & x->_8) == P26)) || (x->_9 > P25);
}

/* checks if x is "negative", requires reduced input */
int is_negative(LL10 *x) {
    return (int)(((is_overflow(x) || (x->_9 < 0))?1:0) ^ (x->_0 & 1));
}

/* Convert to internal format from little-endian byte format */
/* 256 bits split into 10 25-26 bit chunks: 26, 25, 26, 25, 26, 25, 26, 25, 26, 26 */
void unpack(LL10 *x, BYTE *m) {
    x->_0 = m[0] | m[1] << 8  | m[2] << 16 | (m[3] & 3) << 24;
    x->_1 = (m[3] & ~3) >> 2  | m[4] << 6  | m[5] << 14 | (m[6] & 7) << 22;
    x->_2 = (m[6] & ~7) >> 3  | m[7] << 5  | m[8] << 13 | (m[9] & 31) << 21;
    x->_3 = (m[9] & ~31) >> 5  | m[10] << 3 | m[11] << 11 | (m[12] & 63) << 19;
    x->_4 = (m[12] & ~63) >> 6 | m[13] << 2 | m[14] << 10 | m[15] << 18;
    x->_5 = m[16] | m[17] << 8 | m[18] << 16 | (m[19] & 1) << 24;
    x->_6 = (m[19] & ~1) >> 1  | m[20] << 7 | m[21] << 15 | (m[22] & 7) << 23;
    x->_7 = (m[22] & ~7) >> 3  | m[23] << 5 | m[24] << 13 | (m[25] & 15) << 21;
    x->_8 = (m[25] & ~15) >> 4 | m[26] << 4 | m[27] << 12 | (m[28] & 63) << 20;
    x->_9 = (m[28] & ~63) >> 6 | m[29] << 2 | m[30] << 10 | m[31] << 18;
}

/* Convert from internal format to little-endian byte format.  The
 * number must be in a reduced form which is output by the following ops:
 *     unpack, mul, sqr
 *     set --  if input in range 0 .. P25
 * If you're unsure if the number is reduced, first multiply it by 1.
 */
void pack(LL10 *x, BYTE *m) {
    int ld = 0, ud = 0;
    LL t;
    ld = (is_overflow(x)?1:0) - ((x->_9 < 0)?1:0);
    ud = ld * -(P25+1);
    ld *= 19;
    t = ld + x->_0 + (x->_1 << 26);
    m[ 0] = (BYTE)t;
    m[ 1] = (BYTE)(t >> 8);
    m[ 2] = (BYTE)(t >> 16);
    m[ 3] = (BYTE)(t >> 24);
    t = (t >> 32) + (x->_2 << 19);
    m[ 4] = (BYTE)t;
    m[ 5] = (BYTE)(t >> 8);
    m[ 6] = (BYTE)(t >> 16);
    m[ 7] = (BYTE)(t >> 24);
    t = (t >> 32) + (x->_3 << 13);
    m[ 8] = (BYTE)t;
    m[ 9] = (BYTE)(t >> 8);
    m[10] = (BYTE)(t >> 16);
    m[11] = (BYTE)(t >> 24);
    t = (t >> 32) + (x->_4 <<  6);
    m[12] = (BYTE)t;
    m[13] = (BYTE)(t >> 8);
    m[14] = (BYTE)(t >> 16);
    m[15] = (BYTE)(t >> 24);
    t = (t >> 32) + x->_5 + (x->_6 << 25);
    m[16] = (BYTE)t;
    m[17] = (BYTE)(t >> 8);
    m[18] = (BYTE)(t >> 16);
    m[19] = (BYTE)(t >> 24);
    t = (t >> 32) + (x->_7 << 19);
    m[20] = (BYTE)t;
    m[21] = (BYTE)(t >> 8);
    m[22] = (BYTE)(t >> 16);
    m[23] = (BYTE)(t >> 24);
    t = (t >> 32) + (x->_8 << 12);
    m[24] = (BYTE)t;
    m[25] = (BYTE)(t >> 8);
    m[26] = (BYTE)(t >> 16);
    m[27] = (BYTE)(t >> 24);
    t = (t >> 32) + ((x->_9 + ud) << 6);
    m[28] = (BYTE)t;
    m[29] = (BYTE)(t >> 8);
    m[30] = (BYTE)(t >> 16);
    m[31] = (BYTE)(t >> 24);
}

/* Copy a number */
void cpy(LL10 *out, LL10 *in) {
    out->_0=in->_0; out->_1=in->_1;
    out->_2=in->_2; out->_3=in->_3;
    out->_4=in->_4; out->_5=in->_5;
    out->_6=in->_6; out->_7=in->_7;
    out->_8=in->_8; out->_9=in->_9;
}

/* Set a number to value, which must be in range -185861411 .. 185861411 */
void set(LL10 *out, int in) {
    out->_0 = in; out->_1 = 0;
    out->_2 = 0; out->_3 = 0;
    out->_4 = 0; out->_5 = 0;
    out->_6 = 0; out->_7 = 0;
    out->_8 = 0; out->_9 = 0;
}

/* Add/subtract two numbers.  The inputs must be in reduced form, and the
 * output isn't, so to do another addition or subtraction on the output,
 * first multiply it by one to reduce it.
 */
void add(LL10 *xy, LL10 *x, LL10 *y) {
    xy->_0 = x->_0 + y->_0;	xy->_1 = x->_1 + y->_1;
    xy->_2 = x->_2 + y->_2;	xy->_3 = x->_3 + y->_3;
    xy->_4 = x->_4 + y->_4;	xy->_5 = x->_5 + y->_5;
    xy->_6 = x->_6 + y->_6;	xy->_7 = x->_7 + y->_7;
    xy->_8 = x->_8 + y->_8;	xy->_9 = x->_9 + y->_9;
}
void sub(LL10 *xy, LL10 *x, LL10 *y) {
    xy->_0 = x->_0 - y->_0;	xy->_1 = x->_1 - y->_1;
    xy->_2 = x->_2 - y->_2;	xy->_3 = x->_3 - y->_3;
    xy->_4 = x->_4 - y->_4;	xy->_5 = x->_5 - y->_5;
    xy->_6 = x->_6 - y->_6;	xy->_7 = x->_7 - y->_7;
    xy->_8 = x->_8 - y->_8;	xy->_9 = x->_9 - y->_9;
}

/* Multiply a number by a small integer in range -185861411 .. 185861411.
 * The output is in reduced form, the input x need not be.  x and xy may point
 * to the same buffer.
 */
void mul_small(LL10 *xy, LL10 *x, LL y) {
    LL t;
    t = (x->_8*y);
    xy->_8 = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x->_9*y);
    xy->_9 = (t & ((1 << 25) - 1));
    t = 19 * (t >> 25) + (x->_0*y);
    xy->_0 = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x->_1*y);
    xy->_1 = (t & ((1 << 25) - 1));
    t = (t >> 25) + (x->_2*y);
    xy->_2 = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x->_3*y);
    xy->_3 = (t & ((1 << 25) - 1));
    t = (t >> 25) + (x->_4*y);
    xy->_4 = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x->_5*y);
    xy->_5 = (t & ((1 << 25) - 1));
    t = (t >> 25) + (x->_6*y);
    xy->_6 = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x->_7*y);
    xy->_7 = (t & ((1 << 25) - 1));
    t = (t >> 25) + xy->_8;
    xy->_8 = (t & ((1 << 26) - 1));
    xy->_9 += (t >> 26);
}

/* Multiply two numbers.  The output is in reduced form, the inputs need not
 * be.
 */
void mul(LL10 *xy, LL10 *x, LL10 *y) {
    /* sahn0:
     * Using local variables to avoid class access.
     * This seem to improve performance a bit...
     */
    LL x_0=x->_0,x_1=x->_1,x_2=x->_2,x_3=x->_3,x_4=x->_4, x_5=x->_5,x_6=x->_6,x_7=x->_7,x_8=x->_8,x_9=x->_9;
    LL y_0=y->_0,y_1=y->_1,y_2=y->_2,y_3=y->_3,y_4=y->_4, y_5=y->_5,y_6=y->_6,y_7=y->_7,y_8=y->_8,y_9=y->_9;
    LL t;
    t = (x_0*y_8) + (x_2*y_6) + (x_4*y_4) + (x_6*y_2) + (x_8*y_0) + 2 * ((x_1*y_7) + (x_3*y_5) + (x_5*y_3) + (x_7*y_1)) + 38 * (x_9*y_9);
    xy->_8 = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x_0*y_9) + (x_1*y_8) + (x_2*y_7) + (x_3*y_6) + (x_4*y_5) + (x_5*y_4) + (x_6*y_3) + (x_7*y_2) + (x_8*y_1) + (x_9*y_0);
    xy->_9 = (t & ((1 << 25) - 1));
    t = (x_0*y_0) + 19 * ((t >> 25) + (x_2*y_8) + (x_4*y_6) + (x_6*y_4) + (x_8*y_2)) + 38 * ((x_1*y_9) + (x_3*y_7) + (x_5*y_5) + (x_7*y_3) + (x_9*y_1));
    xy->_0 = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x_0*y_1) + (x_1*y_0) + 19 * ((x_2*y_9) + (x_3*y_8) + (x_4*y_7) + (x_5*y_6) + (x_6*y_5) + (x_7*y_4) + (x_8*y_3) + (x_9*y_2));
    xy->_1 = (t & ((1 << 25) - 1));
    t = (t >> 25) + (x_0*y_2) + (x_2*y_0) + 19 * ((x_4*y_8) + (x_6*y_6) + (x_8*y_4)) + 2 * (x_1*y_1) + 38 * ((x_3*y_9) + (x_5*y_7) + (x_7*y_5) + (x_9*y_3));
    xy->_2 = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x_0*y_3) + (x_1*y_2) + (x_2*y_1) + (x_3*y_0) + 19 * ((x_4*y_9) + (x_5*y_8) + (x_6*y_7) + (x_7*y_6) + (x_8*y_5) + (x_9*y_4));
    xy->_3 = (t & ((1 << 25) - 1));
    t = (t >> 25) + (x_0*y_4) + (x_2*y_2) + (x_4*y_0) + 19 * ((x_6*y_8) + (x_8*y_6)) + 2 * ((x_1*y_3) + (x_3*y_1)) + 38 * ((x_5*y_9) + (x_7*y_7) + (x_9*y_5));
    xy->_4 = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x_0*y_5) + (x_1*y_4) + (x_2*y_3) + (x_3*y_2) + (x_4*y_1) + (x_5*y_0) + 19 * ((x_6*y_9) + (x_7*y_8) + (x_8*y_7) + (x_9*y_6));
    xy->_5 = (t & ((1 << 25) - 1));
    t = (t >> 25) + (x_0*y_6) + (x_2*y_4) + (x_4*y_2) + (x_6*y_0) + 19 * (x_8*y_8) + 2 * ((x_1*y_5) + (x_3*y_3) + (x_5*y_1)) + 38 * ((x_7*y_9) + (x_9*y_7));
    xy->_6 = (t & ((1 << 26) - 1));
    t = (t >> 26) + (x_0*y_7) + (x_1*y_6) + (x_2*y_5) + (x_3*y_4) + (x_4*y_3) + (x_5*y_2) + (x_6*y_1) + (x_7*y_0) + 19 * ((x_8*y_9) + (x_9*y_8));
    xy->_7 = (t & ((1 << 25) - 1));
    t = (t >> 25) + xy->_8;
    xy->_8 = (t & ((1 << 26) - 1));
    xy->_9 += (t >> 26);
}

/* Square a number.  Optimization of  mul25519(x2, x, x)  */
void sqr(LL10 *x2, LL10 *x) {
    LL x_0=x->_0, x_1=x->_1, x_2=x->_2, x_3=x->_3, x_4=x->_4,  x_5=x->_5, x_6=x->_6, x_7=x->_7, x_8=x->_8, x_9=x->_9;
    LL t;
    t = (x_4*x_4) + 2 * ((x_0*x_8) + (x_2*x_6)) + 38 * (x_9*x_9) + 4 * ((x_1*x_7) + (x_3*x_5));
    x2->_8 = (t & ((1 << 26) - 1));
    t = (t >> 26) + 2 * ((x_0*x_9) + (x_1*x_8) + (x_2*x_7) + (x_3*x_6) + (x_4*x_5));
    x2->_9 = (t & ((1 << 25) - 1));
    t = 19 * (t >> 25) + (x_0*x_0) + 38 * ((x_2*x_8) + (x_4*x_6) + (x_5*x_5)) + 76 * ((x_1*x_9) + (x_3*x_7));
    x2->_0 = (t & ((1 << 26) - 1));
    t = (t >> 26) + 2 * (x_0*x_1) + 38 * ((x_2*x_9) + (x_3*x_8) + (x_4*x_7) + (x_5*x_6));
    x2->_1 = (t & ((1 << 25) - 1));
    t = (t >> 25) + 19 * (x_6*x_6) + 2 * ((x_0*x_2) + (x_1*x_1)) + 38 * (x_4*x_8) + 76 * ((x_3*x_9) + (x_5*x_7));
    x2->_2 = (t & ((1 << 26) - 1));
    t = (t >> 26) + 2 * ((x_0*x_3) + (x_1*x_2)) + 38 * ((x_4*x_9) + (x_5*x_8) + (x_6*x_7));
    x2->_3 = (t & ((1 << 25) - 1));
    t = (t >> 25) + (x_2*x_2) + 2 * (x_0*x_4) + 38 * ((x_6*x_8) + (x_7*x_7)) + 4 * (x_1*x_3) + 76 * (x_5*x_9);
    x2->_4 = (t & ((1 << 26) - 1));
    t = (t >> 26) + 2 * ((x_0*x_5) + (x_1*x_4) + (x_2*x_3)) + 38 * ((x_6*x_9) + (x_7*x_8));
    x2->_5 = (t & ((1 << 25) - 1));
    t = (t >> 25) + 19 * (x_8*x_8) + 2 * ((x_0*x_6) + (x_2*x_4) + (x_3*x_3)) + 4 * (x_1*x_5) + 76 * (x_7*x_9);
    x2->_6 = (t & ((1 << 26) - 1));
    t = (t >> 26) + 2 * ((x_0*x_7) + (x_1*x_6) + (x_2*x_5) + (x_3*x_4)) + 38 * (x_8*x_9);
    x2->_7 = (t & ((1 << 25) - 1));
    t = (t >> 25) + x2->_8;
    x2->_8 = (t & ((1 << 26) - 1));
    x2->_9 += (t >> 26);
}

/* Calculates a reciprocal.  The output is in reduced form, the inputs need not
 * be.  Simply calculates  y = x^(p-2)  so it's not too fast.
 * When sqrtassist is true, it instead calculates y = x^((p-5)/8)
 */
void recip(LL10 *y, LL10 *x, int sqrtassist) {
    LL10 t0, t1, t2, t3, t4;
    int i;
    /* the chain for x^(2^255-21) is straight from djb's implementation */
    sqr(&t1, x);	/*  2 == 2 * 1	*/
    sqr(&t2, &t1);	/*  4 == 2 * 2	*/
    sqr(&t0, &t2);	/*  8 == 2 * 4	*/
    mul(&t2, &t0, x);	/*  9 == 8 + 1	*/
    mul(&t0, &t2, &t1);	/* 11 == 9 + 2	*/
    sqr(&t1, &t0);	/* 22 == 2 * 11	*/
    mul(&t3, &t1, &t2);	/* 31 == 22 + 9
                == 2^5   - 2^0	*/
    sqr(&t1, &t3);	/* 2^6   - 2^1	*/
    sqr(&t2, &t1);	/* 2^7   - 2^2	*/
    sqr(&t1, &t2);	/* 2^8   - 2^3	*/
    sqr(&t2, &t1);	/* 2^9   - 2^4	*/
    sqr(&t1, &t2);	/* 2^10  - 2^5	*/
    mul(&t2, &t1, &t3);	/* 2^10  - 2^0	*/
    sqr(&t1, &t2);	/* 2^11  - 2^1	*/
    sqr(&t3, &t1);	/* 2^12  - 2^2	*/
    for (i = 1; i < 5; i++) {
        sqr(&t1, &t3);
        sqr(&t3, &t1);
    } /* t3 */		/* 2^20  - 2^10	*/
    mul(&t1, &t3, &t2);	/* 2^20  - 2^0	*/
    sqr(&t3, &t1);	/* 2^21  - 2^1	*/
    sqr(&t4, &t3);	/* 2^22  - 2^2	*/
    for (i = 1; i < 10; i++) {
        sqr(&t3, &t4);
        sqr(&t4, &t3);
    } /* t4 */		/* 2^40  - 2^20	*/
    mul(&t3, &t4, &t1);	/* 2^40  - 2^0	*/
    for (i = 0; i < 5; i++) {
        sqr(&t1, &t3);
        sqr(&t3, &t1);
    } /* t3 */		/* 2^50  - 2^10	*/
    mul(&t1, &t3, &t2);	/* 2^50  - 2^0	*/
    sqr(&t2, &t1);	/* 2^51  - 2^1	*/
    sqr(&t3, &t2);	/* 2^52  - 2^2	*/
    for (i = 1; i < 25; i++) {
        sqr(&t2, &t3);
        sqr(&t3, &t2);
    } /* t3 */		/* 2^100 - 2^50 */
    mul(&t2, &t3, &t1);	/* 2^100 - 2^0	*/
    sqr(&t3, &t2);	/* 2^101 - 2^1	*/
    sqr(&t4, &t3);	/* 2^102 - 2^2	*/
    for (i = 1; i < 50; i++) {
        sqr(&t3, &t4);
        sqr(&t4, &t3);
    } /* t4 */		/* 2^200 - 2^100 */
    mul(&t3, &t4, &t2);	/* 2^200 - 2^0	*/
    for (i = 0; i < 25; i++) {
        sqr(&t4, &t3);
        sqr(&t3, &t4);
    } /* t3 */		/* 2^250 - 2^50	*/
    mul(&t2, &t3, &t1);	/* 2^250 - 2^0	*/
    sqr(&t1, &t2);	/* 2^251 - 2^1	*/
    sqr(&t2, &t1);	/* 2^252 - 2^2	*/
    if (sqrtassist!=0) {
        mul(y, x, &t2);	/* 2^252 - 3 */
    } else {
        sqr(&t1, &t2);	/* 2^253 - 2^3	*/
        sqr(&t2, &t1);	/* 2^254 - 2^4	*/
        sqr(&t1, &t2);	/* 2^255 - 2^5	*/
        mul(y, &t1, &t0); /* 2^255 - 21	*/
    }
}

/* a square root */
void _sqrt(LL10 *x, LL10 *u) {
    LL10 v, t1, t2;
    add(&t1, u, u);	/* t1 = 2u		*/
    recip(&v, &t1, 1);	/* v = (2u)^((p-5)/8)	*/
    sqr(x, &v);		/* x = v^2		*/
    mul(&t2, &t1, x);	/* t2 = 2uv^2		*/
    t2._0--;		/* t2 = 2uv^2-1		*/
    mul(&t1, &v, &t2);	/* t1 = v(2uv^2-1)	*/
    mul(x, u, &t1);	/* x = uv(2uv^2-1)	*/
}

/********************* Elliptic curve *********************/

/* y^2 = x^3 + 486662 x^2 + x  over GF(2^255-19) */

/* t1 = ax + az
 * t2 = ax - az
 */
void mont_prep(LL10 *t1, LL10 *t2, LL10 *ax, LL10 *az) {
    add(t1, ax, az);
    sub(t2, ax, az);
}

/* A = P + Q   where
 *  X(A) = ax/az
 *  X(P) = (t1+t2)/(t1-t2)
 *  X(Q) = (t3+t4)/(t3-t4)
 *  X(P-Q) = dx
 * clobbers t1 and t2, preserves t3 and t4
 */
void mont_add(LL10 *t1, LL10 *t2, LL10 *t3, LL10 *t4, LL10 *ax, LL10 *az, LL10 *dx) {
    mul(ax, t2, t3);
    mul(az, t1, t4);
    add(t1, ax, az);
    sub(t2, ax, az);
    sqr(ax, t1);
    sqr(t1, t2);
    mul(az, t1, dx);
}

/* B = 2 * Q   where
 *  X(B) = bx/bz
 *  X(Q) = (t3+t4)/(t3-t4)
 * clobbers t1 and t2, preserves t3 and t4
 */
void mont_dbl(LL10 *t1, LL10 *t2, LL10 *t3, LL10 *t4, LL10 *bx, LL10 *bz) {
    sqr(t1, t3);
    sqr(t2, t4);
    mul(bx, t1, t2);
    sub(t2, t1, t2);
    mul_small(bz, t2, 121665);
    add(t1, t1, bz);
    mul(bz, t1, t2);
}

/* Y^2 = X^3 + 486662 X^2 + X
 * t is a temporary
 */
void x_to_y2(LL10 *t, LL10 *y2, LL10 *x) {
    sqr(t, x);
    mul_small(y2, x, 486662);
    add(t, t, y2);
    t->_0++;
    mul(y2, t, x);
}

/* P = kG   and  s = sign(P)/k  */
void core(BYTE *Px, BYTE *s, BYTE *k, BYTE *Gx) {
    LL10 dx, t1, t2, t3, t4;
    LL10 x[2], z[2];
    int i, j;

    /* unpack the base */
    if (Gx!=NULL) unpack(&dx, Gx);
    else set(&dx, 9);

    /* 0G = point-at-infinity */
    set(&x[0], 1);
    set(&z[0], 0);

    /* 1G = G */
    cpy(&x[1], &dx);
    set(&z[1], 1);

    for (i = 32; i--!=0; ) {
        for (j = 8; j--!=0; ) {
            /* swap arguments depending on bit */
            int bit1 = (k[i] & 0xFF) >> j & 1;
            int bit0 = ~(k[i] & 0xFF) >> j & 1;
            LL10 *ax = &x[bit0];
            LL10 *az = &z[bit0];
            LL10 *bx = &x[bit1];
            LL10 *bz = &z[bit1];

            /* a' = a + b */
            /* b' = 2 b	*/
            mont_prep(&t1, &t2, ax, az);
            mont_prep(&t3, &t4, bx, bz);
            mont_add(&t1, &t2, &t3, &t4, ax, az, &dx);
            mont_dbl(&t1, &t2, &t3, &t4, bx, bz);
        }
    }

    recip(&t1, &z[0], 0);
    mul(&dx, &x[0], &t1);
    pack(&dx, Px);

    /* calculate s such that s abs(P) = G  .. assumes G is std base point */
    if (s!=NULL) {
        x_to_y2(&t2, &t1, &dx);	/* t1 = Py^2  */
        recip(&t3, &z[1], 0);	/* where Q=P+G ... */
        mul(&t2, &x[1], &t3);	/* t2 = Qx  */
        add(&t2, &t2, &dx);	/* t2 = Qx + Px  */
        t2._0 += 9 + 486662;	/* t2 = Qx + Px + Gx + 486662  */
        dx._0 -= 9;		/* dx = Px - Gx  */
        sqr(&t3, &dx);	/* t3 = (Px - Gx)^2  */
        mul(&dx, &t2, &t3);	/* dx = t2 (Px - Gx)^2  */
        sub(&dx, &dx, &t1);	/* dx = t2 (Px - Gx)^2 - Py^2  */
        dx._0 -= 39420360;	/* dx = t2 (Px - Gx)^2 - Py^2 - Gy^2  */
        mul(&t1, &dx, &BASE_R2Y);	/* t1 = -Py  */
        if (is_negative(&t1)!=0)	/* sign is 1, so just copy  */
            cpy32(s, k);
        else			/* sign is -1, so negate  */
            mula_small(s, ORDER_TIMES_8, 0, k, 32, -1);

        /* reduce s mod q
         * (is this needed?  do it just in case, it's fast anyway)
         */
        //divmod((dstptr) t1, s, 32, order25519, 32);

        /* take reciprocal of s mod q */
        BYTE temp1[32];
        BYTE temp2[64];
        BYTE temp3[64];
        cpy32(temp1, ORDER);
        cpy32(s, egcd32(temp2, temp3, s, temp1));
        if ((s[31] & 0x80)!=0)
            mula_small(s, s, 0, ORDER, 32, 1);
    }
}

/********* KEY AGREEMENT *********/

/* Private key clamping
 *   k [out] your private key for key agreement
 *   k  [in]  32 random bytes
 */
void curve25519_c_clamp(BYTE *k) {
    k[31] &= 0x7F;
    k[31] |= 0x40;
    k[ 0] &= 0xF8;
}

/* Key-pair generation
 *   P  [out] your public key
 *   s  [out] your private key for signing
 *   k  [out] your private key for key agreement
 *   k  [in]  32 random bytes
 * s may be NULL if you don't care
 *
 * WARNING: if s is not NULL, this function has data-dependent timing
 */
void curve25519_c_keygen(BYTE *P, BYTE *s, BYTE *k) {
    curve25519_c_clamp(k);
    core(P, s, k, NULL);
}

/********* DIGITAL SIGNATURES *********/

/* deterministic EC-KCDSA
 *
 *    s is the private key for signing
 *    P is the corresponding public key
 *    Z is the context data (signer public key or certificate, etc)
 *
 * signing:
 *
 *    m = hash(Z, message)
 *    x = hash(m, s)
 *    keygen25519(Y, NULL, x);
 *    r = hash(Y);
 *    h = m XOR r
 *    sign25519(v, h, x, s);
 *
 *    output (v,r) as the signature
 *
 * verification:
 *
 *    m = hash(Z, message);
 *    h = m XOR r
 *    verify25519(Y, v, h, P)
 *
 *    confirm  r == hash(Y)
 *
 * It would seem to me that it would be simpler to have the signer directly do
 * h = hash(m, Y) and send that to the recipient instead of r, who can verify
 * the signature by checking h == hash(m, Y).  If there are any problems with
 * such a scheme, please let me know.
 *
 * Also, EC-KCDSA (like most DS algorithms) picks x random, which is a waste of
 * perfectly good entropy, but does allow Y to be calculated in advance of (or
 * parallel to) hashing the message.
 */

/* Signature generation primitive, calculates (x-h)s mod q
 *   v  [out] signature value
 *   h  [in]  signature hash (of message, signature pub key, and context data)
 *   x  [in]  signature private key
 *   s  [in]  private key for signing
 * returns true on success, false on failure (use different x or h)
 */
int curve25519_c_sign(BYTE *v, BYTE *h, BYTE *x, BYTE *s) {
    // v = (x - h) s  mod q
    int w, i;
    BYTE h1[32], x1[32];
    BYTE tmp1[64] = {0}, tmp2[64] = {0}, tmp3[32] = {0};

    // Don't clobber the arguments, be nice!
    cpy32(h1, h);
    cpy32(x1, x);

    // Reduce modulo group order
    divmod(tmp3, h1, 32, ORDER, 32);
    divmod(tmp3, x1, 32, ORDER, 32);

    // v = x1 - h1
    // If v is negative, add the group order to it to become positive.
    // If v was already positive we don't have to worry about overflow
    // when adding the order because v < ORDER and 2*ORDER < 2^256
    mula_small(v, x1, 0, h1, 32, -1);
    mula_small(v, v , 0, ORDER, 32, 1);

    // tmp1 = (x-h)*s mod q
    mula32(tmp1, v, s, 32, 1);
    divmod(tmp2, tmp1, 64, ORDER, 32);

    for (w = 0, i = 0; i < 32; i++) w |= (v[i] = tmp1[i]);
    return w != 0;
}

/* Signature verification primitive, calculates Y = vP + hG
 *   Y  [out] signature public key
 *   v  [in]  signature value
 *   h  [in]  signature hash
 *   P  [in]  public key
 */
void curve25519_c_verify(BYTE *Y, BYTE *v, BYTE *h, BYTE *P) {
    /* Y = v abs(P) + h G  */
    BYTE d[32];
    LL10 p[2], s[2], yx[3], yz[3], t1[3], t2[3];

    int vi = 0, hi = 0, di = 0, nvh = 0, i, j, k;

    /* set p[0] to G and p[1] to P  */

    set(&p[0], 9);
    unpack(&p[1], P);

    /* set s[0] to P+G and s[1] to P-G  */

    /* s[0] = (Py^2 + Gy^2 - 2 Py Gy)/(Px - Gx)^2 - Px - Gx - 486662  */
    /* s[1] = (Py^2 + Gy^2 + 2 Py Gy)/(Px - Gx)^2 - Px - Gx - 486662  */

    x_to_y2(&t1[0], &t2[0], &p[1]);	/* t2[0] = Py^2  */
    _sqrt(&t1[0], &t2[0]);	/* t1[0] = Py or -Py  */
    j = is_negative(&t1[0]);		/*      ... check which  */
    t2[0]._0 += 39420360;		/* t2[0] = Py^2 + Gy^2  */
    mul(&t2[1], &BASE_2Y, &t1[0]);/* t2[1] = 2 Py Gy or -2 Py Gy  */
    sub(&t1[j], &t2[0], &t2[1]);	/* t1[0] = Py^2 + Gy^2 - 2 Py Gy  */
    add(&t1[1-j], &t2[0], &t2[1]);/* t1[1] = Py^2 + Gy^2 + 2 Py Gy  */
    cpy(&t2[0], &p[1]);		/* t2[0] = Px  */
    t2[0]._0 -= 9;			/* t2[0] = Px - Gx  */
    sqr(&t2[1], &t2[0]);		/* t2[1] = (Px - Gx)^2  */
    recip(&t2[0], &t2[1], 0);	/* t2[0] = 1/(Px - Gx)^2  */
    mul(&s[0], &t1[0],&t2[0]);	/* s[0] = t1[0]/(Px - Gx)^2  */
    sub(&s[0], &s[0], &p[1]);	/* s[0] = t1[0]/(Px - Gx)^2 - Px  */
    s[0]._0 -= 9 + 486662;		/* s[0] = X(P+G)  */
    mul(&s[1], &t1[1], &t2[0]);	/* s[1] = t1[1]/(Px - Gx)^2  */
    sub(&s[1], &s[1],&p[1]);	/* s[1] = t1[1]/(Px - Gx)^2 - Px  */
    s[1]._0 -= 9 + 486662;		/* s[1] = X(P-G)  */
    mul_small(&s[0], &s[0], 1);	/* reduce s[0] */
    mul_small(&s[1], &s[1], 1);	/* reduce s[1] */

    /* prepare the chain  */
    for (i = 0; i < 32; i++) {
        vi = (vi >> 8) ^ (v[i] & 0xFF) ^ ((v[i] & 0xFF) << 1);
        hi = (hi >> 8) ^ (h[i] & 0xFF) ^ ((h[i] & 0xFF) << 1);
        nvh = ~(vi ^ hi);
        di = (nvh & (di & 0x80) >> 7) ^ vi;
        di ^= nvh & (di & 0x01) << 1;
        di ^= nvh & (di & 0x02) << 1;
        di ^= nvh & (di & 0x04) << 1;
        di ^= nvh & (di & 0x08) << 1;
        di ^= nvh & (di & 0x10) << 1;
        di ^= nvh & (di & 0x20) << 1;
        di ^= nvh & (di & 0x40) << 1;
        d[i] = (BYTE)di;
    }

    di = ((nvh & (di & 0x80) << 1) ^ vi) >> 8;

    /* initialize state */
    set(&yx[0], 1);
    cpy(&yx[1], &p[di]);
    cpy(&yx[2], &s[0]);
    set(&yz[0], 0);
    set(&yz[1], 1);
    set(&yz[2], 1);

    /* y[0] is (even)P + (even)G
     * y[1] is (even)P + (odd)G  if current d-bit is 0
     * y[1] is (odd)P + (even)G  if current d-bit is 1
     * y[2] is (odd)P + (odd)G
     */

    vi = 0;
    hi = 0;

    /* and go for it! */
    for (i = 32; i--!=0; ) {
        vi = (vi << 8) | (v[i] & 0xFF);
        hi = (hi << 8) | (h[i] & 0xFF);
        di = (di << 8) | (d[i] & 0xFF);

        for (j = 8; j--!=0; ) {
            mont_prep(&t1[0], &t2[0], &yx[0], &yz[0]);
            mont_prep(&t1[1], &t2[1], &yx[1], &yz[1]);
            mont_prep(&t1[2], &t2[2], &yx[2], &yz[2]);

            k = ((vi ^ vi >> 1) >> j & 1) + ((hi ^ hi >> 1) >> j & 1);
            mont_dbl(&yx[2], &yz[2], &t1[k], &t2[k], &yx[0], &yz[0]);

            k = (di >> j & 2) ^ ((di >> j & 1) << 1);
            mont_add(&t1[1], &t2[1], &t1[k], &t2[k], &yx[1], &yz[1], &p[di >> j & 1]);

            mont_add(&t1[2], &t2[2], &t1[0], &t2[0], &yx[2], &yz[2], &s[((vi ^ hi) >> j & 2) >> 1]);
        }
    }

    k = (vi & 1) + (hi & 1);
    recip(&t1[0], &yz[k], 0);
    mul(&t1[1], &yx[k], &t1[0]);

    pack(&t1[1], Y);
}

BYTE curve25519_c_isCanonicalSignature(BYTE *v) {
    BYTE vCopy[32];
    // todo memcpy
    int i; for(i = 0; i < 32; i++) vCopy[i] = v[i];
    BYTE tmp[32] = {0};
    divmod(tmp, vCopy, 32, ORDER, 32);
    for (i = 0; i < 32; i++) {
        if (v[i] != vCopy[i]) return 0;
    }
    return 1;
}

BYTE curve25519_c_isCanonicalPublicKey(BYTE *publicKey) {
    LL10 publicKeyUnpacked;
    unpack(&publicKeyUnpacked, publicKey);
    BYTE publicKeyCopy[32];
    pack(&publicKeyUnpacked, publicKeyCopy);
    int i; for (i = 0; i < 32; i++){
        if (publicKeyCopy[i] != publicKey[i]) return 0;
    }
    return 1;
}

/************** Interface *************/

/* The main() function below, along with several utility functions,
 * provides an interface to the curve25519 functions above.
 *
 * Usage: curve25519 -[gGsvk] hexString64 ...
 *
 * curve25519 is the name of the executable (compilation of this file).
 *
 * -g: generate signing key pair, given random hexString64
 *     returns: signingPublicKey signingPrivateKey
 * -G: generate key agreement key pair, given random hexString64
 *     returns: keyAgreementPublicKey keyAgreementPrivateKey
 * -s: sign, given signatureHash keyAgreementPrivateKey signingPrivateKey
 *     returns: signatureValue
 * -v: verify, given signatureValue signatureHash signingPublicKey
 *     returns: keyAgreementPublicKey
 * -k: generate shared secret, given myKeyAgreementPrivateKey yourKeyAgreementPublicKey
 *     returns: sharedSecret
 *
 * All input arguments hexString64 are 64-digit hex strings.
 * Output consists of one or two 64-digit hex strings, separated by a space.
 * Errors: reported to stderr; stdout is empty; exit code is 1.
 */
//
//void hexStringToByteArray(BYTE *b, char *str) {
//    int i = 64, j = 32;
//    LL ll;
//    char s[i + 1];
//    strcpy(s, str);
//    do {
//        i -= 8;
//        ll = strtoll(s + i, NULL, 16);
//        int k; for(k = 0; k < 4; k++) {
//            b[--j] = ll & 0xff;
//            ll >>= 8;
//        }
//        s[i] = '\0';
//    } while(i > 0);
//}
//
//void byteArrayToHexString(char *s, BYTE *b) {
//    int i = 64, j = 32;
//    s[i] = '\0';
//    char hexDigits[] = "0123456789ABCDEF";
//    do {
//        BYTE c = b[--j];
//        s[--i] = hexDigits[c & 0xf];
//        s[--i] = hexDigits[c >> 4];
//    } while(j > 0);
//}
//
//int isHexString64(char *s) {
//    int len = strlen(s);
//    if(len != 64) return 0;
//    while(--len >= 0) {
//        if(!isxdigit(s[len])) return 0;
//    }
//    return 1;
//}
//
//void terminate(int code, char *specific) {
//    char *general;
//    switch(code) {
//        case 0: general = "Usage"; break;
//        case 1: general = "Required 64-digit hex string(s)"; break;
//        case 2: default: general = "Error";
//    }
//    fprintf(stderr, "%s: %s\n", general, specific);
//    exit(1);
//}

//char USAGE[] = "curve25519 -[gGsvk] hexString64 ...";
//
//int main(int argc, char **argv) {
//    if(sizeof(LL) < 8 || sizeof(int) < 4 || sizeof(BYTE) > 1)
//        terminate(2, "invalid data type found on this system");
//    if(argc < 2 || argv[1][0] != '-' || strlen(argv[1]) < 2) terminate(0, USAGE);
//    char c = argv[1][1];
//    switch(c) {
//        case 'g': { // generate key pair for signing
//            if(argc != 3 || !isHexString64(argv[2])) terminate(1, "random");
//            BYTE P[32], s[32], k[32];
//            hexStringToByteArray(k, argv[2]);
//            keygen(P, s, k);
//            char Pstr[65]; byteArrayToHexString(Pstr, P); // signingPublicKey
//            char sstr[65]; byteArrayToHexString(sstr, s); // signingPrivateKey
//            printf("%s %s\n", Pstr, sstr);
//            break;
//        }
//        case 'G': { // generate key pair for key agreement
//            if(argc != 3 || !isHexString64(argv[2])) terminate(1, "random");
//            BYTE P[32], k[32];
//            hexStringToByteArray(k, argv[2]);
//            keygen(P, NULL, k);
//            char Pstr[65]; byteArrayToHexString(Pstr, P); // keyAgreementPublicKey
//            char kstr[65]; byteArrayToHexString(kstr, k); // keyAgreementPrivateKey
//            printf("%s %s\n", Pstr, kstr);
//            break;
//        }
//        case 's': { // sign
//            if(argc != 5 || !isHexString64(argv[2]) || !isHexString64(argv[3]) || !isHexString64(argv[4]))
//                terminate(1, "signatureHash keyAgreementPrivateKey signingPrivateKey");
//            BYTE v[32], h[32], x[32], s[32];
//            hexStringToByteArray(h, argv[2]);
//            hexStringToByteArray(x, argv[3]);
//            hexStringToByteArray(s, argv[4]);
//            if(!sign(v, h, x, s)) terminate(2, "signing failed");
//            char vs[65]; byteArrayToHexString(vs, v); puts(vs); // signatureValue
//            break;
//        }
//        case 'v': { // verify
//            if(argc != 5 || !isHexString64(argv[2]) || !isHexString64(argv[3]) || !isHexString64(argv[4]))
//                terminate(1, "signatureValue signatureHash signingPublicKey");
//            BYTE Y[32], v[32], h[32], P[32];
//            hexStringToByteArray(v, argv[2]);
//            hexStringToByteArray(h, argv[3]);
//            hexStringToByteArray(P, argv[4]);
//            if(!isCanonicalSignature(v)) terminate(2, "signature is not canonical");
//            if(!isCanonicalPublicKey(P)) terminate(2, "public key is not canonical");
//            verify(Y, v, h, P);
//            char Ys[65]; byteArrayToHexString(Ys, Y); puts(Ys); // keyAgreementPublicKey
//            break;
//        }
//        case 'k': { // generate shared secret (key agreement)
//            if(argc != 4 || !isHexString64(argv[2]) || !isHexString64(argv[3]))
//                terminate(1, "myKeyAgreementPrivateKey yourKeyAgreementPublicKey");
//            BYTE Z[32], k[32], P[32];
//            hexStringToByteArray(k, argv[2]);
//            hexStringToByteArray(P, argv[3]);
//            curve(Z, k, P);
//            char Zs[65]; byteArrayToHexString(Zs, Z); puts(Zs); // sharedSecret
//            break;
//        }
//        default:
//            terminate(0, USAGE);
//            break;
//    }
//    exit(0);
//}
