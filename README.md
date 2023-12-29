AI di scacchi algoritmo alfa-beta pruning con memorizzazione,

# Giocarci contro
Serve:
* scaricare il repo e ottenere l'eseguibile dell'engine di scacchi
* installare una GUI
* sapere usare la GUI

## Scaricare repo
Con ssh:
'''
git clone git@github.com:StefanoRizzi/Chess-AI.git
'''
Nel repo ci sono gli eseguibili compilati sulla mia macchihna.
Se serve si può ricompilare il programma sulla propria macchina con cargo 'cargo build --release'

## Installazione GUI
Per giocarci contro io uso Scid vs Pc ma si può usare qualsiasi GUI di scacchi che sa usare UCI.
Per installare Scid vs Pc basta soddisfare le dipendenze e lanciare il comandi sotto.
IO ho dovuto anche installare il pacchetto 'libx11-dev'
'''
tar -xzf scid_vs_pc-4.24.tgz
cd scid_vs_pc-4.24
./configure
sudo make install
'''
La pagina principale di Scid vs Pc: https://scidvspc.sourceforge.net/

## Usare la GUI
Aprire un nuovo terminale e lanciare:
'''
scid
'''
Serve prima aggiungere l'engine e poi si può giocare contro.
Aggiungere l'engine:
* menù in alto => terzultima voce "Tools" => prima voce "Analisis Engines"
* bottone "New"
* Nel campo "Name" inserire "Boss Player"
* Nel campo "Command" inserire il path per l'eseguibile che si trova nel repo in "/target/release/chess-rust"
* bottone "OK"

Giocare contro l'engine:
* menù in alto => seconda voce "Play" => quarta voce "Computer - UCI Engine"
* selezionare l'engine "Boss Player"
* forse si vuole deselezionare l'impostazione "Use book", selezionare "Start from current position", togliere i tempi per mossa
* bottone "Play"
