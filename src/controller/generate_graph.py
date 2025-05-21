import sys
import os
import numpy as np
import networkx as nx
import matplotlib.pyplot as plt

def main():
    if len(sys.argv) != 2:
        print("Usage: python main.py <adjacency_matrix_file>")
        exit(1)

    # 入力ファイルの読み込み
    input_file = sys.argv[1]
    arr = np.loadtxt(input_file, delimiter=',')

    # グラフを作成
    G = nx.from_numpy_array(arr)
    pos = nx.kamada_kawai_layout(G)
    nx.draw(G, pos, with_labels=True)

    # 新しいグラフ G2 を作成
    G2 = G.copy()
    node_count = len(G.nodes())
    
    # 擬似ノードの追加
    for node in list(G.nodes()):
        n = int(node)
        G2.add_node(n + node_count, weight=1)
    
    # 擬似ノードと元のノードの接続
    for node in list(G2.nodes()):
        if node == node_count:
            break

        for neighbor in G2.neighbors(node):
            G2.add_edge(node + node_count, neighbor)
    
    # 擬似ノードの位置を調整
    for node in list(G2.nodes()):
        if node == node_count:
            break
        pos[node + node_count] = (pos[node][0], pos[node][1] - 10)

    nx.draw(G2, pos, with_labels=True)

    # ファイルの保存先ディレクトリを指定
    output_dir = os.path.join("files", "topology")
    os.makedirs(output_dir, exist_ok=True)  # ディレクトリが存在しない場合は作成

    # 出力ファイルのパス
    output_file = os.path.join(output_dir, "adjacency_matrix.txt")

    # 隣接行列を保存
    adjacency_matrix = nx.to_numpy_array(G2, dtype=int)
    np.savetxt(output_file, adjacency_matrix, delimiter=',', fmt='%d')
    print(f"Adjacency matrix saved to {output_file}")

if __name__ == "__main__":
    main()
