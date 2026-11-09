/* Casca fina sobre os chips de referencia, para o Rust chamar sem conhecer as structs. */
#include <stdlib.h>
#include <string.h>
#include "ym3438.h"
#include "ympsg.h"

/* ---- PSG ---- */
void *ref_psg_novo(void) { ympsg_t *c = malloc(sizeof(ympsg_t)); YMPSG_Init(c); return c; }
void ref_psg_libera(void *c) { free(c); }
void ref_psg_escreve(void *c, unsigned char d) { YMPSG_Write((ympsg_t *)c, d); }
void ref_psg_ciclo(void *c) { YMPSG_Clock((ympsg_t *)c); }
unsigned short ref_psg_le(void *c) { return YMPSG_Read((ympsg_t *)c); }

/* ---- OPN2 (modo YM2612) ---- */
void *ref_opn2_novo(void) {
    ym3438_t *c = malloc(sizeof(ym3438_t));
    OPN2_SetChipType(ym3438_mode_ym2612);
    OPN2_Reset(c);
    return c;
}
void ref_opn2_libera(void *c) { free(c); }
void ref_opn2_escreve(void *c, unsigned int porta, unsigned char d) { OPN2_Write((ym3438_t *)c, porta, d); }
/* Um ciclo interno. Devolve esquerda e direita (9 bits com sinal). */
void ref_opn2_ciclo(void *c, short *lr) { OPN2_Clock((ym3438_t *)c, lr); }
unsigned char ref_opn2_le(void *c, unsigned int porta) { return OPN2_Read((ym3438_t *)c, porta); }
