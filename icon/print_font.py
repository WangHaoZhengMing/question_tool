from fontTools.ttLib import TTFont

font = TTFont(r"C:\Users\hallm\OneDrive\桌面\question_tool\icon\SF-Symbols.ttf")
cmap = font["cmap"].getBestCmap()

with open("font_chars.txt", "w", encoding="utf-8") as f:
    for codepoint in cmap:
        f.write(chr(codepoint))