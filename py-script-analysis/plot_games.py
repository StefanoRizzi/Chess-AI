
import matplotlib.pyplot as plt

# PUBLIC

def plot_analysis(games: list[tuple[int, int, int, list[int]]], plot_name: str, sample = 50):
    MAX_ELO: int = 4000
    NUM_GAMES: int = len(games)
    num_moves: int = 0

    # analysis    
    elos_analysis: list[dict[str, int]] = [ {} for _ in range(int(MAX_ELO / sample)) ]
    elos_count: list[dict[str, int]] = [ { "num_games": 0, "num_moves": 0 } for _ in range(int(MAX_ELO / sample)) ]

    count = 0
    for id, welo, belo, moves in games:
        if count % (1 + (NUM_GAMES // 100)) == 0:
            print(f"\r\x1b[Janalysing... {100 * count / NUM_GAMES:.0f}%", end='', flush=True)
        count += 1
        num_moves += len(moves)
        white_analysis, black_analysis = get_moves_analysis(moves)
        
        elo_analysis = elos_analysis[int(welo / sample)]
        for name, value in white_analysis.items():
            elo_analysis[name] = elo_analysis.get(name, 0) + value

        elo_analysis = elos_analysis[int(belo / sample)]
        for name, value in black_analysis.items():
            elo_analysis[name] = elo_analysis.get(name, 0) + value
        
        half_moves, extra = divmod(len(moves), 2)
        elos_count[int(welo / sample)]["num_games"] += 1
        elos_count[int(belo / sample)]["num_games"] += 1
        elos_count[int(welo / sample)]["num_moves"] += half_moves + extra
        elos_count[int(belo / sample)]["num_moves"] += half_moves
    print(f"\r\x1b[Janalysed {count} games")
        
    #avg
    y_analysis_avg: dict[str, list[float]] = {}
    x_elo: list[int] = []

    for elo_count, elo_analysis, elo in zip(elos_count, elos_analysis, range(0, MAX_ELO, sample)):
        elo_games, elo_moves = elo_count["num_games"], elo_count["num_moves"]

        if elo_games == 0: continue
        x_elo.append(elo)

        for name in elo_analysis.keys():
            if name not in y_analysis_avg:
                y_analysis_avg[name] = [0] * (len(x_elo) - 1)

        for name in y_analysis_avg.keys():
            value = elo_analysis.get(name, 0)
            y_analysis_avg[name].append(value / elo_moves) # avg

    # plot
    if len(x_elo) != 0:
        if x_elo[-1] - x_elo[0] != sample * (len(x_elo) - 1):
            print(f"Plot problem: gap tra gli elo\nElos = {x_elo}")
            #exit(0)

    for name, y_values in y_analysis_avg.items():
        if name in ("normal"): continue
        plt.plot(x_elo, y_values, label = name)
    
    plt.xlabel("elo players")
    plt.ylabel("analysis avg move")

    plt.legend()
    plt.show()

#PRIVATE

def get_moves_analysis(moves: list[float]) -> tuple[dict[str, int], dict[str, int]]:
    white_analysis: dict[str, int] = {}
    black_analysis: dict[str, int] = {}
    prev_eval: float = 0.2
    prev_status_grade: int = get_game_status(prev_eval)[1]
    is_white: bool = True

    for eval in moves:
        analysis, status_grade = move_analysis(prev_eval, eval, prev_status_grade, is_white)

        current_analysis = white_analysis
        if not is_white: current_analysis = black_analysis
        for name in analysis:
            current_analysis[name] = current_analysis.get(name, 0) + 1

        prev_eval = eval
        prev_status_grade = status_grade
        is_white = not is_white

    return (white_analysis, black_analysis)


def get_game_status(eval):
    if eval < -8:
        return "black_winning", -6
    elif eval < -5:
        return "black_huge_advantage", -5
    elif eval < -3:
        return "black_big_advantage", -4
    elif eval < -1.2:
        return "black_advantage", -3
    elif eval < -0.4:
        return "black_slight_advantage", -2
    elif eval < -0.1:
        return "roughtly_equal", -1
    elif eval < 0.1:
        return "equal", 0
    elif eval < 0.4:
        return "roughtly_equal", 1
    elif eval < 1.2:
        return "white_slight_advantage", 2
    elif eval < 3:
        return "white_advantage", 3
    elif eval < 5:
        return "white_big_advantage", 4
    elif eval < 8:
        return "white_huge_advantage", 5
    else:
        return "white_winning", 6
    

def move_analysis(prev_eval: float, now_eval: float, prev_status_grade: int, is_white: bool) -> tuple[list[str], int]:

    if not is_white:
        prev_eval = -prev_eval
        now_eval = -now_eval
        prev_status_grade = -prev_status_grade
    eval_diff = now_eval - prev_eval

    analysis = []
    if prev_eval > 8 and now_eval > 8:
        analysis.append("normal")
    elif prev_eval < -8 and now_eval < -8:
        analysis.append("normal")
    elif eval_diff < -4:
        analysis.append("wasted")
    elif eval_diff < -2:
        analysis.append("big error")
    elif eval_diff < -1:
        analysis.append("error")
    elif eval_diff < -0.5:
        analysis.append("imprecision")
    else:
        analysis.append("normal")
    
    _, now_status_grade = get_game_status(now_eval)

    if now_status_grade >= 2:
        if prev_status_grade <= -3:
            analysis.append("ribalta")
        elif prev_status_grade <= -2:
            analysis.append("ribaltino")
    elif now_status_grade >= 0:
        if prev_status_grade <= -3:
            analysis.append("rimonta")
        if prev_status_grade <= -2:
            analysis.append("rimontina")

    now_status_grade_white = now_status_grade
    prev_status_grade_white = prev_status_grade
    if not is_white:
        now_status_grade_white = -now_status_grade_white
        prev_status_grade_white = -prev_status_grade_white
    
    if prev_status_grade < 0:
        if now_status_grade < prev_status_grade:
            return (analysis, now_status_grade_white)
        return (analysis, prev_status_grade_white)
    else:
        if now_status_grade > prev_status_grade:
            return (analysis, now_status_grade_white)
        return (analysis, prev_status_grade_white)