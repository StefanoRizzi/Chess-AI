
# script per analizzare gli errori nelle partite sui diversi elo
# e anche per confrontare le analisi di diversi motori scacchistici

import os
import evaluate_games
import plot_games

DB_FILENAME = "lichess_db_standard_rated_2016-01.pgn"
#MAX_SEARCH = 1_000_000
SAMPLES = 50


HOME_PATH = os.path.expanduser('~')
DB_PATH = os.path.join(HOME_PATH, "Scaricati", DB_FILENAME)
STOCK_PATH = os.path.join(HOME_PATH, "Scaricati", DB_FILENAME + ".stock.txt")
BOSS_PATH = os.path.join(HOME_PATH, "Scaricati", DB_FILENAME + ".boss.txt")

#did_stock: bool = evaluate_games.write_stockfish_eval(DB_PATH, STOCK_PATH, MAX_SEARCH)
#if not did_stock: print(f"{DB_FILENAME} jet read")

if os.path.exists(STOCK_PATH):
    stock_games = evaluate_games.load_games_eval(STOCK_PATH)
    plot_games.plot_analysis(stock_games, "Stockfish eval - " + DB_FILENAME, SAMPLES)
if os.path.exists(BOSS_PATH):
    boss_games = evaluate_games.load_games_eval(BOSS_PATH)
    plot_games.plot_analysis(boss_games, "Boss eval - " + DB_FILENAME, SAMPLES)

