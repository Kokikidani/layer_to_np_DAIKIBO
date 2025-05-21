import sys
import matplotlib.pyplot as plt
import numpy as np

# ファイルからデータを読み込みます
data = np.loadtxt(sys.argv[1])

# データを分割します
x = data[:, 0]
y1 = data[:, 1]
y2 = data[:, 2]

fig = plt.figure(figsize=(1920/300, 1080/300))

# グラフの作成
plt.plot(x, y1, label='WXC-based NW', marker='o', markerfacecolor='none')  # 'o' は円のマーカーを表します
plt.plot(x, y2, label='Layer NW', marker='s', markerfacecolor='none')  # 's' は四角のマーカーを表します

# 軸の設定
plt.xscale('linear')
plt.yscale('log')
plt.xlabel('Traffic intensity')
plt.ylabel('Blocking ratio')
plt.grid(True)

# 凡例の表示
plt.legend()

# 別のファイルからテキストを読み込む
with open(sys.argv[2], 'r') as file:
    annotation_text = file.read()[:-1]

# テキストをグラフの右下に表示
plt.annotate(annotation_text, xy=(0.97, 0.05), xycoords='axes fraction',
             fontsize=10, ha='right', va='bottom', bbox=dict(facecolor='white', alpha=0.8))

# グラフの表示
plt.tight_layout()
plt.savefig(sys.argv[3], dpi=300)
