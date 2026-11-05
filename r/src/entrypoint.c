/* Keep extendr's routine registration in the linked library. */
void R_init_ddes_extendr(void *dll);

void R_init_ddes(void *dll) {
    R_init_ddes_extendr(dll);
}
