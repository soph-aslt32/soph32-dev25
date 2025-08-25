"""
このコードは，OR-ToolsのCP-SATソルバーを使用して製品のスケジューリング問題を解決します。
しかし，製品数が増えると計算時間が急激に増加するため，実用的な規模の問題には向いていません。
"""

from ortools.sat.python import cp_model
import random
import time
import io
import sys

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")
sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding="utf-8")

"""
uv run cp_sat01.py > log.txt
"""


def schedule_products(W, P, durations, skills, deadlines: list[int]):
    """
    W: int (作業員数)
    P: int (製品数)
    durations: [[d0,d1,d2,d3] for each product]
    skills: list of sets, skills[w] = {対応可能な工程番号}
    deadlines: list of int
    """
    model = cp_model.CpModel()

    num_steps = 4

    # 最大の終了時刻(最も遅い締め切り)
    horizon = max(deadlines)

    # 変数
    start = {}
    end = {}
    assigned_worker = {}

    for p in range(P):
        for s in range(num_steps):
            start[(p, s)] = model.NewIntVar(0, horizon, f"start_{p}_{s}")
            end[(p, s)] = model.NewIntVar(0, horizon, f"end_{p}_{s}")
            # 工程時間
            d = durations[p][s]
            model.Add(end[(p, s)] == start[(p, s)] + d)

            for w in range(W):
                assigned_worker[(p, s, w)] = model.NewBoolVar(f"assign_{p}_{s}_{w}")
                # スキルがないなら0固定
                if s not in skills[w]:
                    model.Add(assigned_worker[(p, s, w)] == 0)

            # 1つの工程に必ず1人
            model.Add(sum(assigned_worker[(p, s, w)] for w in range(W)) == 1)

    # 工程順序制約
    for p in range(P):
        for s in range(num_steps - 1):
            model.Add(start[(p, s + 1)] >= end[(p, s)])

    # 作業員の重複作業禁止（No-overlap）
    for w in range(W):
        intervals = []
        for p in range(P):
            for s in range(num_steps):
                # optional interval
                interval = model.NewOptionalIntervalVar(
                    start[(p, s)],
                    durations[p][s],
                    end[(p, s)],
                    assigned_worker[(p, s, w)],
                    f"interval_{p}_{s}_{w}",
                )
                intervals.append(interval)
        model.AddNoOverlap(intervals)

    # 締め切り制約
    for p in range(P):
        model.Add(end[(p, num_steps - 1)] <= deadlines[p])

    # 目的関数: 全体のmakespan最小化
    makespan = model.NewIntVar(0, horizon, "makespan")
    for p in range(P):
        model.Add(makespan >= end[(p, num_steps - 1)])
    model.Minimize(makespan)

    # 求解
    solver = cp_model.CpSolver()
    solver.parameters.max_time_in_seconds = 300  # タイムリミット300秒
    status = solver.Solve(model)

    if status in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        for p in range(P):
            print(f"Product {p}:")
            for s in range(num_steps):
                st = solver.Value(start[(p, s)])
                en = solver.Value(end[(p, s)])
                w = next(
                    w for w in range(W) if solver.Value(assigned_worker[(p, s, w)]) == 1
                )
                print(f"  Step {s}: Worker {w}, [{st}, {en}]")
        print(f"Makespan = {solver.Value(makespan)}")
        print(f"status: {status}")
    else:
        print("No solution found.")


# # ==== サンプル入力 ====
# W = 3
# P = 2
# durations = [
#     [3, 2, 2, 1],  # 製品0の4工程
#     [2, 3, 2, 2]   # 製品1の4工程
# ]
# skills = [
#     {0, 1},        # 作業員0は工程0,1担当可
#     {2, 3},        # 作業員1は工程2,3担当可
#     {0, 1, 2, 3}   # 作業員2は全工程対応可
# ]
# deadlines = [15, 15]

# schedule_products(W, P, durations, skills, deadlines)

# ==== 大規模問題入力 ====
# シード固定
random.seed(42)

W = 100
P = 400

# P個のdurationsを1から10の範囲でランダム生成
durations = []
for _ in range(P):
    durations.append([random.randint(1, 10) for _ in range(4)])
# W人のスキルをランダム生成
skills = []
while True:
    for _ in range(W):
        skills.append(set(random.sample(range(4), random.randint(1, 4))))

    # skillsを確認して，対応できない工程があれば再生成
    if all(any(s in skills[w] for w in range(W)) for s in range(4)):
        break
    skills = []

# 締め切りは全て非常に大きな数字に設定
deadlines = [10000 for _ in range(P)]

# 設定値を表示
print("=== 設定値 ===")
print(f"作業員数: {W}")
print(f"製品数: {P}")
print("工程時間:")
for p in range(P):
    print(f"  製品{p}: {durations[p]}")
print("スキル:")
for w in range(W):
    print(f"  作業員{w}: {skills[w]}")
# print("締め切り:")
# for p in range(P):
#     print(f"  製品{p}: {deadlines[p]}")

# スケジューリングを実行
# 計算時間を計測
start_time = time.time()
schedule_products(W, P, durations, skills, deadlines)
end_time = time.time()
print(f"計算時間: {end_time - start_time}秒")

# 入力情報のdurationsの総和を計算して表示
total_durations = sum(sum(d) for d in durations)
print(f"総工程時間: {total_durations}")
