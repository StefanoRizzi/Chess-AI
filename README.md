AI di scacchi algoritmo alfa-beta pruning con memorizzazione,

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
Per giocarci contro io uso **Scid vs Pc** ma si può usare qualsiasi **GUI** di scacchi che sa usare **UCI**.

Per installare **Scid vs Pc** basta soddisfare le dipendenze e lanciare il comandi sotto.
> Io ho dovuto anche installare il pacchetto **libx11-dev**
```bash
tar -xzf scid_vs_pc-4.24.tgz
cd scid_vs_pc-4.24
./configure
sudo make install
```
Vedere [**Scid vs Pc**](https://scidvspc.sourceforge.net/) per maggiori informazioni

## Usare la GUI
Aprire un nuovo terminale e lanciare `scid`

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
