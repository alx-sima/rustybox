# Rustybox

## autor: Sima Alexandru

### Comenzi acceptate
- `pwd`: print working directory
- `echo [-n] MESSAGES...`: display messages
- `grep [-i] PATTERN FILE`: 
- `cat FILES...`: print file contents
- `mkdir DIRS...`: create directories
- `mv DEST SOURCE`: move/rename files
- `ln [-s] SOURCE DEST`: (sym)link a file
- `rmdir DIRS...`: remove empty directory
- `rm [-r|-d] FILES...`: remove files
- `ls [-R|-a|-l] FILES...`: list files
- `cp [-r] SOURCE DEST`: copy files
- `touch [-a|-c|-m] FILES...`: modify atime/mtime of files
- `chmod MODE FILE`: change permissions of a file

### Implementare
Se iau argumentele programului: primul este numele comenzii de executat, restul
fiind argumente pentru comanda, fiind, unde este necesar, împărțite în flag-uri
(încep cu "-") și argumente.

#### pwd
Afișează calea curentă.

#### echo
Afișează argumentele primite, urmate (dacă flagul `-n` e absent) de un newline.

#### grep
Se parsează patternul pentru a se crea un validator, care conține un vector de 
clojure-uri, care verifică dacă un caracter dintr-un string se potrivește cu 
acel token (acea poziție din pattern) și avansează (unde este cazul), pentru a 
continua căutarea.

Tokenii implementați sunt următorii:
- `^` (beginning of line): caracterul este primul din string;
- `$` (end of line): caracterul este ultimul din string;
- `.`: caracterul poate fi orice;
- `*`: caracterul anterior poate apărea de 0 sau mai multe ori. Pentru aceasta, 
nu am adăugat un token nou, ci practic l-am modificat pe primul pentru a consuma
cât mai multe caractere posibil și să valideze orice caracter primește (deoarece
poate apărea și de 0 ori).
- ...: caracterul trebuie să fie cel dat.  

#### cat
Afișează conținuturile fișierelor date ca argumente.

#### mkdir
Creează directoare.

#### mv
Redenumește un fișier.

#### ln
Creează o legătură (simbolică, cu flagul `-s`) pentru un fișier.

#### rmdir
Șterge un director (gol).

#### rm
Șterge fișiere sau directoare (cu flagul `-r`) recursiv. Alternativ, se comportă
ca `rmdir`, daca flagul `-d` este prezent.

#### ls
Afișează lista de fișiere din directoare (din directorul curent dacă nu este 
precizat altul). Dacă flagul `-a` nu este prezent, ignoră fișierele ascunse 
(care încep cu .).

Dacă flagul `-R` este prezent, se afișează fișierele recursiv.

Dacă flagul `-l` este prezent, afișează mai multe informații despre fișiere:
- string care semnifică tipul fișierului și permisiunile;
- owner (deoarece din metadate se poate obține doar uid-ul ownerului, numele 
acestuia este căutat în fișierul `/etc/passwd`)
- group (analog ca la owner)
- dimensiunea fișierului
- mtime (formatat cu ajutorul crateului `chrono`)
- numele

#### cp
Copiază un fișier la destinație. Dacă este prezent flagul `-r`, copiază 
directorul recursiv (întâi creează directorul destinație, apoi toate fișierele 
din el, iar dacă întâlnește alt director, repetă).

#### touch
Modifică timpii de acces și modificare ai unui fișier.
Pentru a nu folosi syscall-uri, am forțat modificarea `atime`-ului prin citirea
unui octet din fișier, și a `mtime`-ului prin scrierea unui octet și ștergerea 
lui.

Prin flaguri se poate modifica funcționalitatea pentru a modifica doar `atime`, 
doar `mtime`, sau pentru a nu crea un fișier dacă nu există deja.

#### chmod
Modifică permisiunile unui fișier. Permisiunile pot fi date în 2 moduri, fie un 
număr octal, caz în care doar se atribuie acele permisiuni fișierului, fie sub 
formă de string.

---

Stringul de permisiuni este de forma `/[ugoa]*[+-][rwx]*/`, având următoarele 
semnificații:

- Primul grup reprezintă cui se aplică permisiunile: **U**ser, **G**roup, 
**O**ther sau **A**ll;
- Simbolul de la mijloc indică dacă permisiunile se adaugă (`+`) sau elimină 
(`-`);
- Ultimul grup marchează permisiunile care se modifică: **R**ead, **W**rite, 
e**X**ecute.

Pentru a se modifica permisiunile, se creează o mască, știind că acestea sunt de
forma: `(u:) rwx (g:) rwx (o:) rwx`, iar `a` însemnând toate cele 3. Astfel, 
pentru a selecta cui se aplică permisiunile, se setează 7 (`0b111`) pe poziția 
corespunzătoare. Apoi, pentru a selecta permisiunile, masca face ȘI cu biții 
corespunzători permisiunii (de ex. 4 (`0b100`) pentru *r*). În final, se 
folosește masca pentru a seta sau șterge permisiunile deja existente ale unui 
fișier.
