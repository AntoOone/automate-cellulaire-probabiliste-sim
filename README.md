# Simulation à Grande échelle du modèle probabiliste de l'électeur

Dans cette application, l'objectif est de se concentrer sur les grandes simulations avec `N`, `d` et `k` suffisament grands pour ne pas pouvoir aisément énumérer exhaustivement les cas.
Seul le cas périodique fait l'objet de ces simulations, le cas non périodique pourraît être implémenté sans difficulté.

# Comment lancer la simulation

La simulation fonctionne sur Windows, Linux et MacOS, avec ou sans OpenCL d'installé.

### Format des commandes :

```bash
cargo run --release -- [FICHIER_DE_CONFIGURATION] [OPTIONS_AFFICHAGE] [OPTIONS_DE_SAUVEGARDE]
```

Pour la lancer avec opencl :

```bash
cargo run --release --features opencl -- [FICHIER_DE_CONFIGURATION] [OPTIONS_AFFICHAGE] [OPTIONS_DE_SAUVEGARDE]
```

### Exemples :

Pour une petite simulation 5x5 :

```bash
cargo run --release -- simulation_setup/config5x5.json display frequency 20 simulation_save/sim.json 5
```

Pour une simulation à partir d'une image 300x300 :

```bash
cargo run --release -- simulation_setup/config300x300.png display frequency 60 simulation_save/sim.json 50
```

Pour une simulation gourmande sans affichage sur une image 500x500 :

```bash
cargo run --release -- simulation_setup/config500x500.png hide inf simulation_save/sim.json 100
```

<details>
<summary><b>Pour arrêter la simulation</b></summary>

- En mode display (interface graphique) : fermer la fenêtre.

- En mode hide (terminal) : Appuyer sur `Espace` ou attendre que le nb_iteration soit atteint.
  La simulation peut durer infiniment avec `nb_iteration` à `inf`

</details>
<br/>
<details>
<summary><b>OPTIONS DE CONFIGURATION</b></summary>

Le fichier de configuration est donné par un `path`. Il indique à la simulation dans quel état elle doit commencer, quelles sont ses dimensions (`n` et `d`) et comment elle doit s'afficher (si elle est affichée).
Ce fichier doit être soit au format json ou png.

exemple :

```bash
simulation_setup/config5x5.json
```

### format png

L'image doit être carrée et les couleurs représentent les catégories des cellules.
Il ne peut pas y avoir plus de 256 couleurs différentes.

### format json

exemple d'un tel fichier :

```json
{
  "d": 2,
  "tab": [
    0,
    0,
    1,
    0,
    0,
    1,
    0,
    1,
    0,
    0,
    2,
    2,
    0,
    0,
    0,
    2,
    0,
    1,
    1,
    0,
    0,
    1,
    1,
    0,
    1
  ],
  "cell_to_color": [
    [255, 255, 255],
    [255, 0, 0],
    [0, 255, 255]
  ]
}
```

Le champ `cell_to_color` est uniquement requis si l'on veut afficher la simulation dans les options d'affichage.

</details>
<details>
<summary><b>OPTIONS D'AFFICHAGE</b></summary>

### Pour un affichage graphique

L'application fournit un affichage graphique grâce aux librairies rust `winit` et `pixels` qui permet de suivre la simulation en temps réel.

Pour des raisons de performance, on doit lui fournir `display_interval` qui représente le nombre d'itérations à attendre avant d'afficher la simulation.

Format :

```bash
display iteration display_interval
```

Exemple :

```bash
display iteration 10 # Affiche la simulation toutes les 10 intérations
```

Pour fixer la vitesse de la simulation à la fréquence `display_frequency`. C'est utile pour ralentir l'observation afin de l'observer.

```bash
display frequency display_frequency
```

Exemple :

```bash
display frequency 20 # Fixe la vitesse de la simulation à 20Hz
```

### Pour un affichage terminal seulement

On indique à la simulation de ne pas s'afficher et de s'arrêter au bout de `nb_iteration` itérations.
`nb_iteration` peut être soit un nombre soit `inf` si on veut qu'elle soit infinie.
La simulation offre la possibilité d'appuyer sur la touche `Espace` pour l'arrêter et sauvegarder.

Format :

```bash
hide nb_iteration
```

</details>
<details>
<summary><b>OPTIONS DE SAUVEGARDE</b></summary>

A la fin de la simulation, l'application va écrire dans un fichier de sauvegarde.

La sauvegarde sera sous la forme d'un fichier json qui va stocke différents élémements importants de la simulation.

Les données sauvegardées sont : la variation de l'entropie, des fréquences des catégories, leur quantité ainsi que n, d et k.

Le `save_interval` indique à la simulation qu'il faut prendre en compte les données de la simulation 1 fois toutes les `save_interval` itérations.
Le `save_path` indique le chemin du fichier json de sauvegarde.

Format :

```bash
save_path save_interval
```

Exemple :

```bash
simulation_save/sim.json 10
```

</details>
<br/>
<details>
<summary><b>OpenCL</b></summary>

# Dépendance OpenCL

Ce projet utilise OpenCL via la crate rust `ocl`.

Il est nécessaire d'avoir OpenCL installé sur sa machine pour pouvoir exécuter la simulation avec OpenCL.
Si vous ne souhaitez pas l'utiliser et lancer la simulation sur votre CPU, il suffit de ne pas mettre `--features opencl` dans la commande d'exécution.

</details>
