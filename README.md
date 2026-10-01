> 📚 **Complete Documentation Available:** Explore the modular documentation suite in [`docs/`](docs/README.md) covering [Hardware & Wiring](docs/hardware.md), [Protocol Specifications](docs/protocol.md), [Firmware Architecture](docs/firmware.md), [Getting Started](docs/getting-started.md), and the [Engineering Journey](docs/development-journey.md).

Voici la rétrospective structurée de notre projet, décomposée en trois grands axes : les fondations physiques, la guerre temporelle, et la logistique finale.

1. Les obstacles physiques et la fondation
Avant même que le code puisse exister, le circuit électrique devait former une boucle parfaite.

L'échec de la Masse (GND) flottante : Au début, le signal ne passait pas car la soudure de la masse était défaillante.

L'analogie de la plomberie : C'est comme essayer de faire couler de l'eau dans un tuyau (le fil de données) sans avoir branché le tuyau d'évacuation (la masse). L'eau refuse d'avancer. La tension ne pouvait pas s'établir.

L'échec de l'absence de Pull-Up : Le fil de données flottait dans le vide, incapable de remonter à 3.3V assez vite.

L'analogie du ressort : Sans la résistance de 2 kΩ, le fil manquait de "tension mécanique" pour rebondir vers le haut après avoir été tiré vers le bas. L'ajout physique de cette résistance a garanti des signaux carrés et nets.

2. La guerre temporelle (Les échecs logiciels)
C'est ici que nous avons rencontré le plus de défis. La manette GameCube exige une précision à la microseconde (1 millionième de seconde).

L'échec du "Vestiaire" (La lenteur de Rust) : Nous avons d'abord utilisé les outils standards de Rust pour allumer et éteindre la broche.

Le problème : Ces outils vérifient la sécurité du système à chaque appel. Changer l'état prenait 50 cycles de processeur au lieu d'un seul. L'impulsion était trop longue, la manette ne comprenait rien.

La solution : L'accès brut (Macro pull_low!). Nous avons contourné les sécurités pour manipuler l'interrupteur électrique général de la puce en 1 seul cycle.

L'échec du "Couloir" (La mémoire Flash) : Même avec un accès brut, le code était stocké dans la mémoire externe de la puce.

Le problème : Le processeur devait faire des allers-retours dans un bus de communication pour lire chaque ligne de code, créant des micro-retards chaotiques.

La solution : L'étiquette #[link_section = ".data"]. Nous avons forcé le processeur à copier la recette directement sur son bureau (en mémoire RAM) au démarrage.

L'échec du "Téléphone" (Les interruptions USB) :

Le problème : L'ordinateur interrogeait le port USB en plein milieu d'une lecture de manette. Le processeur mettait le chronomètre en pause pour répondre, détruisant la mesure de notre microseconde.

La solution : Le mode "Ne pas déranger" (interrupt::free). Pendant le dialogue avec la manette (400 µs), le processeur devient sourd au reste du monde.

3. La barrière de la langue et de la logistique
Une fois le signal parfait, il fallait le traduire correctement pour Linux.

L'échec de la grammaire (L'inversion logique) :

Le problème : Je traduisais un fil "longtemps en bas" comme un 0, au lieu d'un 1.

L'analogie du miroir : Le signal passait parfaitement, mais le traducteur écrivait le dictionnaire à l'envers. Le PC rejetait ces suites de chiffres illogiques.

L'échec de la "Suffocation" (Le bouton 15 fantôme et les zéros) :

Le problème : Le processeur tournait à 125 millions d'opérations par seconde et harcelait la manette de questions sans arrêt. La manette s'étouffait et renvoyait des données corrompues.

La solution : Le respirateur temporel. Nous avons imposé un délai strict de 10 millisecondes (100 interrogations par seconde) grâce au chronomètre interne de la puce, laissant à la manette le temps de formuler ses réponses.

L'échec de la "Boîte Unique" (Les axes à -32767) :

Le problème : En déclarant deux joysticks distincts au PC avec des variables isolées, Linux les a fusionnés. Les données du C-Stick écrasaient celles du stick principal.

La solution : Le tableau strict (axes: [u8; 4]). Nous avons fabriqué un colis avec 4 compartiments pré-numérotés (0x30 à 0x33). Chaque axe a désormais sa propre place physique dans le rapport USB.

Bilan : Comment la machine finale fonctionne-t-elle aujourd'hui ?
Ton adaptateur est désormais un pont d'une efficacité chirurgicale. Voici le déroulement exact de son cycle de vie en 4 étapes logiques :

Le Métronome (Chaque 10 ms) : Le processeur consulte son horloge absolue matérielle. Si 10 millisecondes se sont écoulées, il déclenche l'interrogation.

L'Isolement : Le processeur coupe toutes ses communications externes (USB) et exécute son code depuis la RAM pour éviter toute latence physique.

Le Dialogue Brut :

Il envoie l'ordre de parler (0x40 0x03 0x00) en tapant des impulsions électriques parfaites de 1 et 3 microsecondes.

Il attend que la manette baisse le fil, attend très exactement 2 microsecondes, et photographie l'état du fil. Si c'est en bas, c'est un 0. Si c'est remonté à 3.3V, c'est un 1. Il répète cela 64 fois pour lire les 8 octets.

Le Traitement et l'Expédition :

Il rouvre les communications.

Il efface mathématiquement le tout premier bit (la signature Nintendo qui allumait le Bouton 15).

Il range les boutons dans deux octets distincts, et distribue précisément l'axe X, l'axe Y, le C-Stick X et le C-Stick Y dans un tableau de 4 cases.

Le PC vient récupérer ce rapport USB standardisé et l'interprète sans aucun effort via jstest.

Le système est stable, déterministe (aucune variation de temps n'est laissée au hasard) et conforme aux normes électriques de Nintendo comme aux normes logicielles de l'USB.
