Giocatore di scacchi AI con algoritmo alfa-beta pruning e memorizzazione

# Giocarci contro
Serve:
* Scaricare il repo e ottenere l'eseguibile dell'**engine** di scacchi
* Installare una **GUI**
* Sapere usare la **GUI**

## Scaricare repo
Con ssh:
```bash
git clone git@github.com:StefanoRizzi/Chess-AI.git
```
> Nel repo ci sono gli eseguibili compilati sulla mia macchina nella cartella versions. \
> Se serve si può ricompilare il programma sulla propria con cargo `cargo build --release` \
> L'eseguibile viene salvato in [target/release/chess-rust]()

## Installazione GUI
Per giocarci contro io uso **Scid vs Pc** ma si può usare qualsiasi **GUI** di scacchi che implementa il protocollo **UCI**.

Per scaricare **Scid vs Pc** seguire questo [link](https://sourceforge.net/projects/scidvspc/postdownload). \
Per installare **Scid vs Pc** basta soddisfare le dipendenze e lanciare i comandi sotto.
> Nella mia distribuzione ho dovuto anche installare il pacchetto **libx11-dev**
```bash
tar -xzf scid_vs_pc-4.24.tgz
cd scid_vs_pc-4.24
./configure
sudo make install
```
Vedere [**Scid vs Pc**](https://scidvspc.sourceforge.net/) per maggiori informazioni

## Usare la GUI
Aprire un nuovo terminale e lanciare il comando `scid`

Prima serve **aggiungere l'engine** e poi si può **giocare contro**.

**Aggiungere l'engine:**
* ***Menù*** in alto => terzultima voce ***Tools*** => prima voce ***Analisis Engines***
* Bottone ***New***
* Nel campo ***Name*** inserire **"Boss Player"**
* Nel campo ***Command*** inserire il path dell'eseguibile che si trova nel repo. L'ultima versione: [versions/boss_pieces_map](versions/boss_pieces_map)
* Bottone ***OK***

**Giocare contro l'engine:**
* ***Menù*** in alto => seconda voce ***Play*** => quarta voce ***Computer - UCI Engine***
* Selezionare l'engine **"Boss Player"**
* Forse si vuole deselezionare l'impostazione **Use book**, selezionare **Start from current position**, togliere i tempi per mossa
* Bottone ***Play***

# Implementazione Engine
Il programma utilizza l'**algoritmo di alpha-beta pruning** e una **transposition table** per la memorizzazione dei nodi (posizioni visitate). \
La scacchiera è rappresentata da una **matrice 8x8** e dalle **liste dei pezzi**. Sono momerizzate anche delle **matrici 8x8** con dentro i quadrati visti dai pezzi. \
La posizione è hashata secondo la tecnica **zobrist**, per poter vedere le **ripetizioni** e per la **transposition table**. \
La posizione viene valutata solamente come differenza della somma dei valori dei pezzi, il valore di un pezzo dipende dalla sua locazione e dal tipo di pezzo secondo delle mappe predeterminate. \
L'engine comunica con il protocollo **UCI**.

## Step successivi per migliorare l'engine
* usare rappresentazione **bitboard** per migliorare la velocità e facilitare la valutazione delle posizioni
* aggiungere un **database di aperture** da seguire per dare varietà al gioco
* migliorare la **valutazione** delle posizioni
* migliorare il **"search"** estendendo la ricerca per le mosse più significative.
