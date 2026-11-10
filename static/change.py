# change.py
# 易經六十四卦繪圖程式庫 for Pyodide / Python
# 作者：依使用者需求設計
# 使用方式：字串 "111111" 表示由上到下六爻（1=陽爻實線，0=陰爻斷線）
# 乾卦 = "111111"，坤卦 = "000000"
#
# 支援指定 HTML div id 作為繪圖目標（Pyodide 環境）

"""
易經卦象繪圖模組
================

主要功能：
- draw(binary_str, title=None, target_div=None, clear=True, show=True, ...)
    依 6 位 0/1 字串繪製卦象（字串由上到下）
    target_div 可指定要顯示的 div 元素 id（例如 "mpl_output1"）
- print_hexagram(binary_str)
    以文字方式輸出卦象
- get_name(binary_str)
    取得卦名（文王卦序）
- list_all()
    列出全部 64 卦
- from_king_wen(n)
    依文王卦序 1~64 取得 binary 字串（上到下）

字串規則（本模組統一採用「由上到下」）：
  字串第 0 字元 = 上爻（第 6 爻）
  字串第 5 字元 = 初爻（第 1 爻）
  1 = 陽爻（—）
  0 = 陰爻（- -）

Pyodide 繪圖目標：
  draw(..., target_div="mpl_output1")
  會把圖放到該 id 的 div 內。
"""

from __future__ import annotations
import matplotlib
import matplotlib.pyplot as plt
from typing import Optional, Tuple, Dict, List, Union
import io
import base64

# ============================================================
# 文王卦序 1~64 對應的「由上到下」binary（1=陽, 0=陰）
# ============================================================
KING_WEN: Dict[int, Tuple[str, str, str]] = {
    1:  ("111111", "乾", "乾為天"),
    2:  ("000000", "坤", "坤為地"),
    3:  ("010001", "屯", "水雷屯"),
    4:  ("100010", "蒙", "山水蒙"),
    5:  ("010111", "需", "水天需"),
    6:  ("111010", "訟", "天水訟"),
    7:  ("000010", "師", "地水師"),
    8:  ("010000", "比", "水地比"),
    9:  ("110111", "小畜", "風天小畜"),
    10: ("111011", "履", "天澤履"),
    11: ("000111", "泰", "地天泰"),
    12: ("111000", "否", "天地否"),
    13: ("111101", "同人", "天火同人"),
    14: ("101111", "大有", "火天大有"),
    15: ("000100", "謙", "地山謙"),
    16: ("001000", "豫", "雷地豫"),
    17: ("011001", "隨", "澤雷隨"),
    18: ("100110", "蠱", "山風蠱"),
    19: ("000011", "臨", "地澤臨"),
    20: ("110000", "觀", "風地觀"),
    21: ("101001", "噬嗑", "火雷噬嗑"),
    22: ("100101", "賁", "山火賁"),
    23: ("100000", "剝", "山地剝"),
    24: ("000001", "復", "地雷復"),
    25: ("111001", "無妄", "天雷無妄"),
    26: ("100111", "大畜", "山天大畜"),
    27: ("100001", "頤", "山雷頤"),
    28: ("011110", "大過", "澤風大過"),
    29: ("010010", "坎", "坎為水"),
    30: ("101101", "離", "離為火"),
    31: ("011100", "咸", "澤山咸"),
    32: ("001110", "恆", "雷風恆"),
    33: ("111100", "遯", "天山遯"),
    34: ("001111", "大壯", "雷天大壯"),
    35: ("101000", "晉", "火地晉"),
    36: ("000101", "明夷", "地火明夷"),
    37: ("110101", "家人", "風火家人"),
    38: ("101011", "睽", "火澤睽"),
    39: ("010100", "蹇", "水山蹇"),
    40: ("001010", "解", "雷水解"),
    41: ("100011", "損", "山澤損"),
    42: ("110001", "益", "風雷益"),
    43: ("011111", "夬", "澤天夬"),
    44: ("111110", "姤", "天風姤"),
    45: ("011000", "萃", "澤地萃"),
    46: ("000110", "升", "地風升"),
    47: ("011010", "困", "澤水困"),
    48: ("010110", "井", "水風井"),
    49: ("011101", "革", "澤火革"),
    50: ("101110", "鼎", "火風鼎"),
    51: ("001001", "震", "震為雷"),
    52: ("100100", "艮", "艮為山"),
    53: ("110100", "漸", "風山漸"),
    54: ("001011", "歸妹", "雷澤歸妹"),
    55: ("001101", "豐", "雷火豐"),
    56: ("101100", "旅", "火山旅"),
    57: ("110110", "巽", "巽為風"),
    58: ("011011", "兌", "兌為澤"),
    59: ("110010", "渙", "風水渙"),
    60: ("010011", "節", "水澤節"),
    61: ("110011", "中孚", "風澤中孚"),
    62: ("001100", "小過", "雷山小過"),
    63: ("010101", "既濟", "水火既濟"),
    64: ("101010", "未濟", "火水未濟"),
}

_BINARY_TO_INFO: Dict[str, Tuple[int, str, str]] = {
    v[0]: (k, v[1], v[2]) for k, v in KING_WEN.items()
}

TRIGRAMS: Dict[str, Tuple[str, str]] = {
    "111": ("乾", "天"),
    "110": ("兌", "澤"),
    "101": ("離", "火"),
    "100": ("震", "雷"),
    "011": ("巽", "風"),
    "010": ("坎", "水"),
    "001": ("艮", "山"),
    "000": ("坤", "地"),
}


def _validate(binary: str) -> str:
    """檢查並正規化 6 位 0/1 字串"""
    s = str(binary).strip().replace(" ", "")
    if len(s) != 6 or any(c not in "01" for c in s):
        raise ValueError(
            f"卦象必須是 6 位由 0 與 1 組成的字串（由上到下），收到：{binary!r}"
        )
    return s


def get_name(binary: str) -> str:
    """回傳卦名（若為標準文王卦則含序號與別名）"""
    s = _validate(binary)
    if s in _BINARY_TO_INFO:
        n, name, alias = _BINARY_TO_INFO[s]
        return f"{n}. {name}（{alias}）"
    upper = TRIGRAMS.get(s[:3], ("?", "?"))
    lower = TRIGRAMS.get(s[3:], ("?", "?"))
    return f"{upper[0]}{lower[0]}（上{upper[1]}下{lower[1]}）"


def from_king_wen(n: int) -> str:
    """依文王卦序 1~64 回傳由上到下的 binary 字串"""
    if n not in KING_WEN:
        raise ValueError("文王卦序必須是 1 到 64 的整數")
    return KING_WEN[n][0]


def list_all() -> None:
    """印出全部 64 卦對照表"""
    print("文王卦序 | binary(上→下) | 卦名")
    print("-" * 40)
    for n in range(1, 65):
        b, name, alias = KING_WEN[n]
        print(f"{n:2d} | {b} | {name}（{alias}）")


def print_hexagram(binary: str, show_name: bool = True) -> None:
    """以文字方式繪製卦象（由上到下）"""
    s = _validate(binary)
    if show_name:
        print(get_name(s))
        print()
    for i, bit in enumerate(s):
        if bit == "1":
            line = "━━━━━━━"
        else:
            line = "━━━  ━━━"
        pos = ["上", "五", "四", "三", "二", "初"][i]
        print(f"  {line}   {pos}")


# ------------------------------------------------------------
# Pyodide / 瀏覽器 輔助函式
# ------------------------------------------------------------
def _is_pyodide() -> bool:
    """粗略判斷是否在 Pyodide 環境"""
    try:
        from js import document  # type: ignore
        return True
    except Exception:
        return False


def _set_target_div(div_id: str, clear: bool = True) -> None:
    """
    設定 matplotlib 輸出目標 div，並可選擇清空。
    使用 document.pyodideMplTarget（Pyodide 官方建議方式）
    """
    from js import document  # type: ignore

    el = document.getElementById(div_id)
    if el is None:
        raise ValueError(f"找不到 id 為 '{div_id}' 的 HTML 元素")

    # 設定繪圖目標
    document.pyodideMplTarget = el

    if clear:
        el.innerHTML = ""


def _inject_png_to_div(fig, div_id: str, clear: bool = True) -> None:
    """
    備用方案：把 figure 存成 PNG base64，直接塞進指定 div。
    不依賴特定 matplotlib backend，相容性較高。
    """
    from js import document  # type: ignore

    el = document.getElementById(div_id)
    if el is None:
        raise ValueError(f"找不到 id 為 '{div_id}' 的 HTML 元素")

    if clear:
        el.innerHTML = ""

    buf = io.BytesIO()
    fig.savefig(buf, format="png", dpi=120, bbox_inches="tight",
                facecolor=fig.get_facecolor(), edgecolor="none")
    buf.seek(0)
    b64 = base64.b64encode(buf.read()).decode("ascii")
    buf.close()

    img = document.createElement("img")
    img.src = f"data:image/png;base64,{b64}"
    img.style.maxWidth = "100%"
    img.style.height = "auto"
    img.style.display = "block"
    img.style.margin = "0.5em auto"
    el.appendChild(img)


def draw(
    binary: str,
    title: Optional[str] = None,
    target_div: Optional[str] = None,
    clear: bool = True,
    ax: Optional[plt.Axes] = None,
    show: bool = True,
    method: str = "auto",          # "auto" | "mpl_target" | "png"
    figsize: Tuple[float, float] = (3.2, 4.5),
    line_color: str = "#1a1a1a",
    bg_color: str = "#faf8f0",
    yang_width: float = 2.6,
    yin_gap: float = 0.55,
) -> plt.Axes:
    """
    使用 matplotlib 繪製卦象，並可指定輸出到特定 div。

    參數
    ----
    binary : str
        6 位 0/1 字串，由上到下（第 0 字元 = 上爻）
    title : str, optional
        圖標題，預設自動使用卦名
    target_div : str, optional
        目標 HTML 元素的 id，例如 "mpl_output1"。
        若為 None，則依 matplotlib 預設行為（通常 append 到 body）。
    clear : bool
        繪製前是否清空 target_div 內容（預設 True）
    method : str
        "auto"       → 優先用 pyodideMplTarget，失敗則改用 base64 圖片
        "mpl_target" → 強制使用 document.pyodideMplTarget + plt.show()
        "png"        → 強制把 PNG 圖片注入 div
    show : bool
        是否立即顯示（在非 target_div 情況下呼叫 plt.show()）
    """
    s = _validate(binary)

    if ax is None:
        fig, ax = plt.subplots(figsize=figsize, facecolor=bg_color)
        ax.set_facecolor(bg_color)
    else:
        fig = ax.figure

    # 座標設定
    ax.set_xlim(0, 4)
    ax.set_ylim(0, 6.5)
    ax.set_aspect("equal")
    ax.axis("off")

    y_positions = [5.5, 4.5, 3.5, 2.5, 1.5, 0.5]
    center_x = 2.0
    half_w = yang_width / 2

    for i, bit in enumerate(s):
        y = y_positions[i]
        if bit == "1":
            ax.plot(
                [center_x - half_w, center_x + half_w],
                [y, y],
                color=line_color,
                linewidth=6.0,
                solid_capstyle="butt",
            )
        else:
            gap = yin_gap / 2
            ax.plot(
                [center_x - half_w, center_x - gap],
                [y, y],
                color=line_color,
                linewidth=6.0,
                solid_capstyle="butt",
            )
            ax.plot(
                [center_x + gap, center_x + half_w],
                [y, y],
                color=line_color,
                linewidth=6.0,
                solid_capstyle="butt",
            )

    if title is None:
        title = get_name(s)
    ax.set_title(title, fontsize=14, fontweight="bold", pad=12, color="#222")

    labels = ["上", "五", "四", "三", "二", "初"]
    for i, y in enumerate(y_positions):
        ax.text(3.55, y, labels[i], va="center", ha="left", fontsize=10, color="#555")

    # ---------- 輸出到指定 div ----------
    if target_div is not None and _is_pyodide():
        use_png = (method == "png")
        use_mpl = (method == "mpl_target")

        if method == "auto":
            # 優先嘗試官方 pyodideMplTarget
            try:
                _set_target_div(target_div, clear=clear)
                use_mpl = True
            except Exception:
                use_png = True

        if use_mpl and not use_png:
            try:
                _set_target_div(target_div, clear=clear)
                plt.tight_layout()
                if show:
                    plt.show()
                return ax
            except Exception as e:
                # 失敗就 fallback 到 png
                print(f"[change] mpl_target 失敗，改用 png 注入：{e}")
                use_png = True

        if use_png:
            plt.tight_layout()
            _inject_png_to_div(fig, target_div, clear=clear)
            # 關閉 figure 避免記憶體累積
            plt.close(fig)
            return ax

    # 一般情況（本機或未指定 target_div）
    if show:
        plt.tight_layout()
        plt.show()

    return ax


def draw_multiple(
    binaries: List[str],
    titles: Optional[List[str]] = None,
    target_div: Optional[str] = None,
    clear: bool = True,
    ncols: int = 4,
    figsize_per: Tuple[float, float] = (2.8, 4.0),
    method: str = "auto",
) -> None:
    """一次繪製多個卦象到同一個 target_div（或預設位置）"""
    n = len(binaries)
    nrows = (n + ncols - 1) // ncols
    fig, axes = plt.subplots(
        nrows, ncols,
        figsize=(figsize_per[0] * ncols, figsize_per[1] * nrows),
        facecolor="#faf8f0",
    )
    if n == 1:
        axes = [axes]
    else:
        axes = axes.flatten()

    for i, b in enumerate(binaries):
        t = titles[i] if titles and i < len(titles) else None
        # 重用 draw 的繪製邏輯，但不讓它 show / 注入
        draw(b, title=t, ax=axes[i], show=False, target_div=None)
    for j in range(i + 1, len(axes)):
        axes[j].axis("off")

    plt.tight_layout()

    if target_div is not None and _is_pyodide():
        if method == "png" or method == "auto":
            try:
                _inject_png_to_div(fig, target_div, clear=clear)
                plt.close(fig)
                return
            except Exception as e:
                print(f"[change] 多圖 png 注入失敗：{e}")
        # fallback
        try:
            _set_target_div(target_div, clear=clear)
            plt.show()
        except Exception as e:
            print(f"[change] 多圖 mpl_target 失敗：{e}")
            plt.show()
    else:
        plt.show()


# ------------------------------------------------------------
# 快捷函式
# ------------------------------------------------------------
def qian(target_div: Optional[str] = None, **kwargs):
    """乾卦"""
    return draw("111111", target_div=target_div, **kwargs)


def kun(target_div: Optional[str] = None, **kwargs):
    """坤卦"""
    return draw("000000", target_div=target_div, **kwargs)


def demo(target_div: Optional[str] = None):
    """示範幾個常見卦"""
    print("=== 文字版示範 ===")
    print_hexagram("111111")
    print()
    print_hexagram("000000")
    print()
    print_hexagram("010001")  # 屯
    print()
    print("=== 圖形版示範 ===")
    draw_multiple(
        ["111111", "000000", "010001", "010101"],
        target_div=target_div,
        ncols=4,
    )


if __name__ == "__main__":
    demo()
