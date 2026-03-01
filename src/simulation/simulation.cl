/// Générateur pseudo aléatoire xorshift
/// Modifie le state et retourne un nombre pseudo aléatoire
/// https://en.wikipedia.org/wiki/Xorshift
uint xorshift32(__global uint *state)
{
	/* Algorithm "xor" from p. 4 of Marsaglia, "Xorshift RNGs" */
	uint x = *state;
	x ^= x << 13;
	x ^= x >> 17;
	x ^= x << 5;
	*state = x;
	return x;
}

/// Générateur pseudo aléatoire
/// Modifie le state et retourne un nombre pseudo aléatoire
/// https://fr.wikipedia.org/wiki/G%C3%A9n%C3%A9rateur_congruentiel_lin%C3%A9aire
inline uint lcg(__global uint *state) {
    const uint nb_iterations = 1000;
    const uint a = 0xfffeb28du;
    const uint c = 1u;
    for (uint i = 0; i < nb_iterations; i++){
        *state = a * *state + c % 2^(32);
    }
    return *state;
}

void add_dir_to_pos(uint *pos,__constant uchar *dir,  uint n, uchar d){
    for (uchar i = 0; i < d; i++){
        switch (dir[i]){
            case 1:
                pos[i] = pos[i] == (n-1) ? 0 : pos[i] + 1;
                return;
            case 2:
               pos[i] = pos[i] == 0 ? n-1 : pos[i] - 1;
                return;
        }
    }
}


__constant uchar dir_d1[2][1] =  {{1}, {2}};

void simulation_step_d1(__global uint *state, __global uchar *tab, __global uchar *background_buffer, uint first_cell_index, uint nb_cells, uint n){
    for (int i = 0; i < nb_cells; i++){
        uint index = first_cell_index + i;
        uint x = index;

        uchar r = xorshift32(state) & 1; // equivalent à % 2
        uint pos[1] = {x};
        add_dir_to_pos(pos, dir_d1[r], n, 1);
        uint index_n = pos[0];
        background_buffer[index] = tab[index_n];
    }
}


__constant uchar dir_d2[4][2] =  {{0, 2}, {0, 1}, {1, 0}, {2, 0}};

void simulation_step_d2(__global uint *state, __global uchar *tab, __global uchar *background_buffer, uint first_cell_index, uint nb_cells, uint n){
    for (int i = 0; i < nb_cells; i++){
        uint index = first_cell_index + i;
        uint x = index % n;
        uint y = index / n;

        uchar r = xorshift32(state) & 3; // equivalent à % 4
        uint pos[2] = {x, y};
        add_dir_to_pos(pos, dir_d2[r], n, 2);
        uint index_n = n * pos[1] + pos[0];
        background_buffer[index] = tab[index_n];
    }
}


__constant uchar dir_d3[6][3] =  {{0, 0, 1}, {0, 0, 2}, {0, 1, 0}, {0, 2, 0}, {1, 0, 0}, {2, 0, 0}};

void simulation_step_d3(__global uint *state, __global uchar *tab, __global uchar *background_buffer, uint first_cell_index, uint nb_cells, uint n){
    for (int i = 0; i < nb_cells; i++){
        uint index = first_cell_index + i;
        uint x = index % n;
        uint y = index / n;
        uint z = index / (n * n);

        uchar r = xorshift32(state) % 6;
        uint pos[3] = {x, y, z};
        add_dir_to_pos(pos, dir_d2[r], n, 3);
        uint index_n = n * n * pos[2] + n * pos[1] + pos[0];
        background_buffer[index] = tab[index_n];
    }
}


/// nb_cell = q * nb_processes + r (division euclidienne)
///
/// nb_cell = (q + 1) * r + q * (nb_processes-r) (On a bien nb_processes > r)
/// 
/// Les r premiers traitent q+1 éléments et les nb_processes-r derniers en traitent q
///
/// neighbour_weights est de taille nb_opinions * nb_processes
///
/// rng  de taille nb_processes
__kernel void simulation_step(__global uchar* tab, __global uchar* background_buffer, __global uint* seeds,
                              uint nb_interation,
                              uchar nb_opinions, 
                              uint n, 
                              uchar d,
                              uint nb_process, 
                              uint q, uint r) {
    uint process_index = get_global_id(0);

    // On détermine les cellules dont on s'occupe dans ce processus
    uint nb_cells;
    uint fist_cell_index;
    if (process_index < r) {
        nb_cells = q+1;
        fist_cell_index = (q+1) * process_index;
    } else{
        nb_cells = q;
        fist_cell_index = (q+1) * r + q * (process_index - r);
    }

    // On récupère la seed du thread correspondant
    __global uint *seed_ptr = seeds + process_index;
   
    switch (d){
        case 1:
            simulation_step_d1(seed_ptr, tab, background_buffer, fist_cell_index, nb_cells, n);
            break;
        case 2:
            simulation_step_d2(seed_ptr, tab, background_buffer, fist_cell_index, nb_cells, n);
            break;
        case 3:
            simulation_step_d3(seed_ptr, tab, background_buffer, fist_cell_index, nb_cells, n);
            break;
    }
}

