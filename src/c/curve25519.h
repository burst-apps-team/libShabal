typedef unsigned char BYTE;

void curve25519_c_keygen(BYTE *P, BYTE *s, BYTE *k);
void curve25519_c_sign(BYTE *v, BYTE *h, BYTE *x, BYTE *s);
void curve25519_c_verify(BYTE *Y, BYTE *v, BYTE *h, BYTE *P);
BYTE curve25519_c_isCanonicalSignature(BYTE *v);
BYTE curve25519_c_isCanonicalPublicKey(BYTE *publicKey);
