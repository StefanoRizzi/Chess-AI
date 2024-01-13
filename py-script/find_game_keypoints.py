import os
import matplotlib.pyplot as plt

HOME = os.path.expanduser('~')
FILENAME = os.path.join(HOME, "Scaricati", "lichess_db_standard_rated_2014-01.pgn")   

MAX_SEARCH = 10_000_000
ELO_STEP = 50
IS_ON_GAMES_OR_ON_MOVES = False

db = open(FILENAME, "r")
info_elos = [ {} for _ in range(int(4000 / ELO_STEP)) ]

def read_game_metadata(db):
    line = None
    while True:
        line = db.readline()
        if len(line) == 0:
            return {}
        if line[0] == "[":
            break
    
    metadata = {}

    while line[0] == "[":
        field, value, _ = line.split("\"")
        field = field[1:-1]
        
        metadata[field] = value

        line = db.readline()

    return metadata

def game_analisis(db):
    def read_notes(line, is_white):
        if line[0] == "{":
            result = {}
            notes, line = line[2:].split("} ", 1)
            
            if is_white:
                turn_line = line.split(" ", 1)
                if len(turn_line) == 2:
                    _, line = turn_line
            
            
            notes = notes.split("[")[1:]
            for note in notes:
                field, value = note[:-2].split(" ")
                result[field] = value
            return result, line
        else:
            return {}, line
        
    def get_evaluation(notes):
        eval = notes.get(f"%eval", None)
        if eval is not None:
            if eval[0] == "#":
                if eval[1] == "-":
                    eval = -999.0
                else:
                    eval = 999.0
            else:
                eval = float(eval)
        return eval

    def get_game_status(eval):
        if eval < -8:
            return "black_winning", -5
        elif eval < -5:
            return "black_huge_advantage", -4
        elif eval < -3:
            return "black_big_advantage", -3
        elif eval < -1.2:
            return "black_advantage", -2
        elif eval < -0.4:
            return "black_slight_advantage", -1
        elif eval < -0.1:
            return "roughtly_equal", 0
        elif eval < 0.1:
            return "equal", 1
        elif eval < 0.4:
            return "roughtly_equal", 2
        elif eval < 1.2:
            return "white_slight_advantage", 3
        elif eval < 3:
            return "white_advantage", 4
        elif eval < 5:
            return "white_big_advantage", 5
        elif eval < 8:
            return "white_huge_advantage", 6
        else:
            return "white_winning", 7
        

    def add_move_type(prev_eval, eval, info, is_white):
        
        if prev_eval is not None and eval is not None:
            info["evaluated"] = 1
            info["tot_eval_moves"] = info.get("tot_eval_moves", 0) + 1
            diff = prev_eval - eval
            if not is_white:
                diff = -diff
            if prev_eval > 8 and eval > 8:
                info["normal"] = info.get("normal", 0) + 1
            elif prev_eval < -8 and eval < -8:
                info["normal"] = info.get("normal", 0) + 1
            elif diff > 4:
                info["wasted"] = info.get("wasted", 0) + 1
            elif diff > 2:
                info["big_error"] = info.get("big_error", 0) + 1
            elif diff > 1:
                info["error"] = info.get("error", 0) + 1
            elif diff > 0.5:
                info["imprecision"] = info.get("imprecision", 0) + 1
            else:
                info["normal"] = info.get("normal", 0) + 1

            game_status = get_game_status(eval)
            if not is_white:
                game_status = game_status[0], -game_status[1]
                info["game_status"] = info["game_status"][0], -info["game_status"][1]

            if game_status[1] >= 2:
                if info["game_status"][1] <= -3:
                    info["ribalta"] = info.get("ribalta", 0) + 1
                elif info["game_status"][1] <= -2:
                    info["ribaltino"] = info.get("ribaltino", 0) + 1
            elif game_status[1] >= 0:
                if info["game_status"][1] <= -3:
                    info["rimonta"] = info.get("rimonta", 0) + 1
                if info["game_status"][1] <= -2:
                    info["rimontina"] = info.get("rimontina", 0) + 1

            if info["game_status"][1] < game_status[1]:
                info["game_status"] = game_status
            
            if not is_white:
                #game_status[1] = -game_status[1]
                info["game_status"] = info["game_status"][0], -info["game_status"][1]


    info = { "game_count" : 1, "game_status" : get_game_status(0.1) }
    line = db.readline()
    prev_eval = 0.1

    while True:
        turn_line = line.split(" ", 1)
        if len(turn_line) == 1:
            break
        turn, line = turn_line
        
        white_line = line.split(" ", 1)
        if len(white_line) == 1:
            break
        white_move, line = white_line
        
        white_notes, line = read_notes(line, True)

        white_eval = get_evaluation(white_notes)
        if white_eval == None:
            break

        add_move_type(prev_eval, white_eval, info, True)
        prev_eval = white_eval

        black_line = line.split(" ", 1)
        if len(black_line) == 1:
            break
        black_move, line = black_line

        black_notes, line = read_notes(line, False)

        black_eval = get_evaluation(black_notes)
        
        add_move_type(prev_eval, black_eval, info, False)
        prev_eval = black_eval

    info.pop("game_status")
    return info

if __name__ == "__main__":
    searched = 0
    info_sum = {}
    while True:
        if searched == MAX_SEARCH:
            break

        meta = read_game_metadata(db)
        if len(meta) == 0:
            break

        searched += 1
        print(f"\r\x1b[Jsearched: {searched}", end = "", flush=True)

        if meta["WhiteElo"] == "?" or meta["BlackElo"] == "?":
            continue

        elo = max(int(meta["WhiteElo"]), int(meta["BlackElo"]))

        info = game_analisis(db)
        info_sum = info_elos[int(elo / ELO_STEP)]
        for key, value in info.items():
            info_sum[key] = info_sum.get(key, 0) + value

    print()

    data = {}
    xelo = []
    elo = 0
    for info in info_elos:
        if len(info) != 0:
            xelo.append(elo)

            for key in info.keys():
                if key not in data:
                    data[key] = [0] * (len(xelo) - 1)

            tot = info.get("evaluated", 1)
            if not IS_ON_GAMES_OR_ON_MOVES:
                tot = info.get("tot_eval_moves", 1) 
            for key, value in data.items():
                value.append(info.get(key, 0) / tot)
        elo += ELO_STEP

    for key, values in data.items():
        if key in ("game_count", "tot_eval_moves", "evaluated", "normal"):
            continue
        plt.plot(xelo, values, label = key)
    
    plt.xlabel("player - elo")
    plt.ylabel("move - times")

    plt.legend()
    plt.show()