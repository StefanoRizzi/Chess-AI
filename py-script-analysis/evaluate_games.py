import os
import re

# PUB

def write_stockfish_eval(file_name: str, to_file_name: str, max_search: int) -> bool:
    if os.path.exists(to_file_name): return False
    
    games: list[dict] = read_games(file_name, True, max_search)
    save_games_evals(games, to_file_name)

    return True

def write_boss_eval(file_name: str, to_file_name: str, max_search: int) -> bool:
    if os.path.exists(to_file_name): return False
    
    games: list[dict] = read_games(file_name, True, max_search)
    
    #TODO
    
    save_games_evals(games, to_file_name)

    return True


# PRIVATE
    
# Game { [note name] -> str, moves -> list[Move] }
# Move tuple[eval -> str, text -> str]
def read_game(file_db, check_eval: bool) -> tuple[dict, bool]:
    def read_game_notes(file_db) -> dict:
        game = {}
        note: str = None
        while True:
            note = file_db.readline()
            if note == None: return None
            if note.startswith("["): break

        while note.startswith("["):
            name, value = note[1:-2].split(" ", 1)
            game[name] = value[1:-1]
            note = file_db.readline()
        return game
    
    def read_move_note_eval(str_notes: str) -> str:
        #move = {}

        #notes = notes_str.split("[%")[1:]
        #for note in notes:
        #    name, value = note[:-2].split(" ")
        #    move[name] = value
        move_eval = notes_str.partition(f"[%eval ")[2].partition("]")[0]
        return move_eval

    game: dict = read_game_notes(file_db)
    if game is None: return None
    str_moves: str = file_db.readline()
    moves: list[tuple[str, str]] = []
    
    if check_eval:
        if not str_moves.partition(" ")[2].partition(" ")[2].startswith("{"):
            return (game, False)

    while True:
        white_turn, _, str_moves = str_moves.partition(" ")
        if str_moves == "": break

        white_move_text, _, str_moves = str_moves.partition(" ")
        black_move_text, _, str_moves = str_moves.partition(" ")
        
        white_move_eval = ""
        black_move_eval = ""
        if black_move_text == "{":
            notes_str, _, str_moves = str_moves.partition("} ")
            white_move_eval = read_move_note_eval(notes_str)
            black_turn, _, str_moves = str_moves.partition(" ")
            if str_moves != "":
                black_move_text, _, str_moves = str_moves.partition(" ")
        if str_moves.startswith("{"):
            notes_str, _, str_moves = str_moves.partition("} ")
            black_move_eval = read_move_note_eval(notes_str)

        moves.append((white_move_text, white_move_eval))
        if str_moves == "": break
        moves.append((black_move_text, black_move_eval))

    game["moves"] = moves
    return (game, True)

def read_games(file_name: str, check_eval: bool, max_search: int) -> list[dict]:
    file_db = open(file_name, "r")

    games: list[dict] = []
    count: int = 0
    found: int = 0
    while True:
        if count >= max_search: break
        print(f"\r\x1b[Jreading games: {len(games)} found {count} done", end='', flush=True)

        res = read_game(file_db, check_eval=True)
        if res is None: break
        game, has_eval = res

        if has_eval:
            game["id"] = count
            games.append(game)
            found += 1
        count += 1

    print(f"\r\x1b[Jfound {len(games)} on {count} games")
    
    return games

def save_games_evals(games: list[dict], to_file_name: str):
    print(f"saving to {to_file_name}")
    f_out = open(to_file_name, "w")
    #print(games[13238])
    f_out.write(f"{len(games)}\n")
    for game in games:
        id, welo, belo = game["id"], game["WhiteElo"], game["BlackElo"]
        f_out.write(f"id {id} welo {welo} belo {belo}\n")
        
        last_eval = None
        for move in game["moves"]:
            move_eval = move[1]
            if move_eval == "": move_eval = last_eval
            f_out.write(f"{move_eval} ")
            last_eval = move_eval
        f_out.write("\n")
    print("Done")

def load_games_eval(file_name: str) -> list[tuple[int, int, int, list[int]]]:
    print(f"loading from {file_name}")
    file_evals = open(file_name, "r")

    num = int(file_evals.readline())
    games: list[tuple[int, int, int, list[int]]] = []
    count = 0
    while True:
        line = file_evals.readline()
        if line == "": break

        if num > 0:
            if count % (1 + (num // 100)) == 0:
                print(f"\r\x1b[Jloading... {100 * count / num:.0f}%", end='', flush=True)
        count += 1

        _, id, _, welo, _, belo = line.split(" ")
        id, welo, belo = int(id), int(welo), int(belo)
        
        eval_moves: list[float] = []
        line = file_evals.readline()

        for move_eval in line[:-2].split(" "):
            if move_eval.startswith("#"):
                if int(move_eval[1:]) > 0: eval_moves.append(999)
                else: eval_moves.append(-999)
            else:
                eval_moves.append(float(move_eval))
        
        games.append((id, welo, belo, eval_moves))
    print(f"\r\x1b[Jloaded {len(games)} games")

    return games