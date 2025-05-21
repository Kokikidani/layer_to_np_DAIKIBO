import sys
import matplotlib.pyplot as plt

# ファイルのパスを指定
file_path1 = sys.argv[1]
file_path2 = sys.argv[2]

# データを格納する辞書を作成
data1 = {}
data2 = {}

average_x1 = 0.0
average_x2 = 0.0

# ファイルを読み込んでデータを辞書に保存
with open(file_path1, 'r') as file:
    for line in file:
        if ':' in line:
            key, value = line.split(':')
            key = key.strip()
            value = value.strip()
            if key.isdigit():
                data1[int(key)] = int(value)
            else:
                average_x1 = float(value)

# ファイルを読み込んでデータを辞書に保存
with open(file_path2, 'r') as file:
    for line in file:
        if ':' in line:
            key, value = line.split(':')
            key = key.strip()
            value = value.strip()
            if key.isdigit():
                data2[int(key)] = int(value)
            else:
                average_x2 = float(value)

# x, yのリストを作成
x1 = list(data1.keys())
y1 = list(data1.values())
x2 = list(data2.keys())
y2 = list(data2.values())

# 総和で正規化
total_sum1 = sum(y1)
y_normalized1 = [value / total_sum1 for value in y1]

total_sum2 = sum(y2)
y_normalized2 = [value / total_sum2 for value in y2]

while len(y_normalized2) < len(y_normalized1):
    y_normalized2.append(0.0)
while len(y_normalized1) < len(y_normalized2):
    y_normalized1.append(0.0)
if len(x1) < len(x2):
    x1 = x2

y_max = round(max(max(y_normalized1), max(y_normalized2)) / 0.05) * 0.05

# グラフを上下に並べるためにサブプロットを作成
fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(10, 8))

# 上のグラフ
ax1.set_title("WXC-based NW")
ax1.set_ylim(0, y_max)
ax1.bar(x1, y_normalized1, color="tab:blue")
ax1.set_xlabel('WXC Port Traverse Count')
ax1.set_ylabel('Path Proportion')
even_ticks = [i for i in x1 if i % 2 == 0]
ax1.set_xticks(even_ticks)
ax1.axvline(x=average_x1, color='orange', linestyle='--', linewidth=2)
ax1.annotate(f"Average: {average_x1}", xy=(0.98, 0.96), xycoords='axes fraction',
             fontsize=10, ha='right', va='top', bbox=dict(facecolor='white', alpha=0.8))

# 下のグラフ
ax2.set_title("Layer NW")
ax2.set_ylim(0, y_max)
ax2.bar(x1, y_normalized2, color="tab:green")
ax2.set_xlabel('WXC Port Traverse Count')
ax2.set_ylabel('Path Proportion')
ax2.set_xticks(even_ticks)
ax2.axvline(x=average_x2, color='tab:red', linestyle='--', linewidth=2)
ax2.annotate(f"Average: {average_x2}", xy=(0.98, 0.96), xycoords='axes fraction',
             fontsize=10, ha='right', va='top', bbox=dict(facecolor='white', alpha=0.8))

# グラフを表示
plt.tight_layout()
plt.savefig(sys.argv[3], dpi=300)
