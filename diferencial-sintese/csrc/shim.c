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

/* ---- Observacao do estado interno (campos publicos de ym3438.h), so para diagnostico ---- */
/* Camada: 0 pg_phase, 1 eg_level, 2 eg_out, 3 fm_out, 4 ch_out, 5 pg_inc, 6 eg_state, 7 fm_mod, 8 misc.
   Escreve ate 24 valores em `out`. Devolve quantos. */
int ref_opn2_estado(void *p, int camada, int *out) {
    ym3438_t *c = (ym3438_t *)p;
    int i, n = 24;
    switch (camada) {
    case 0: for (i = 0; i < 24; i++) out[i] = c->pg_phase[i]; break;
    case 1: for (i = 0; i < 24; i++) out[i] = c->eg_level[i]; break;
    case 2: for (i = 0; i < 24; i++) out[i] = c->eg_out[i]; break;
    case 3: for (i = 0; i < 24; i++) out[i] = c->fm_out[i]; break;
    case 4: n = 6; for (i = 0; i < 6; i++) out[i] = c->ch_out[i]; break;
    case 5: for (i = 0; i < 24; i++) out[i] = c->pg_inc[i]; break;
    case 6: for (i = 0; i < 24; i++) out[i] = c->eg_state[i]; break;
    case 7: for (i = 0; i < 24; i++) out[i] = c->fm_mod[i]; break;
    case 9: for (i = 0; i < 24; i++) out[i] = c->pg_reset[i]; break;
    case 10: for (i = 0; i < 24; i++) out[i] = c->mode_kon[i]; break;
    case 11: for (i = 0; i < 24; i++) out[i] = c->eg_kon_latch[i]; break;
    case 12: out[0] = c->fnum[0]; out[1] = c->pg_fnum; out[2] = c->block[0]; out[3] = c->pg_block; break;
    default: n = 8;
        out[0] = c->cycles; out[1] = c->channel; out[2] = c->mol; out[3] = c->mor;
        out[4] = c->eg_timer; out[5] = c->lfo_cnt; out[6] = c->eg_cycle; out[7] = c->ch_acc[0]; break;
    }
    return n;
}
