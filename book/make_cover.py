# -*- coding: utf-8 -*-
"""Обложка книги Кенга: тёмный A5, то же семейство, что Z-система."""
import os
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

HERE = os.path.dirname(os.path.abspath(__file__))
rng = np.random.default_rng(13)


def _front():
    fig = plt.figure(figsize=(5.5, 8.5), dpi=300)
    ax = fig.add_axes([0, 0, 1, 1])
    ax.set_xlim(0, 1)
    ax.set_ylim(0, 1)
    ax.axis("off")
    ax.set_facecolor("#0a1428")

    x = np.linspace(0, 1, 320)
    for i, (y0, beta, w, col) in enumerate([
        (0.28, 1.1, 2.1, "#2ee6d6"),
        (0.24, 1.8, 1.5, "#5b9dff"),
        (0.20, 2.6, 1.1, "#b0d4ff"),
        (0.32, 1.5, 1.7, "#1a6b6b"),
    ]):
        y = y0 + 0.15 * np.exp(-beta * x) * np.cos(12 * x + i * 0.7)
        ax.plot(x, y, color=col, lw=w, alpha=0.82)

    pts = rng.normal(0.5, 0.17, (70, 2))
    sizes = 180 * np.exp(-3 * np.abs(pts - 0.5).mean(1))
    ax.scatter(pts[:, 0], pts[:, 1], s=sizes, c="#ffd97a", alpha=0.72, lw=0)

    ax.text(0.5, 0.90, "КЕНГА", ha="center", va="center",
            fontsize=36, color="white", weight="bold")
    ax.text(0.5, 0.82, "Язык, который компилирует\nи учит себя",
            ha="center", va="center", fontsize=14, color="#b0d4ff")
    ax.text(0.5, 0.075, "Герман Янтарас", ha="center", va="center",
            fontsize=16, color="white")
    ax.text(0.5, 0.045, "при участии kenga-lite 3.13", ha="center", va="center",
            fontsize=9, color="#8899bb")
    fig.savefig(os.path.join(HERE, "cover.png"), facecolor="#0a1428")
    plt.close()


def _back():
    fig = plt.figure(figsize=(5.5, 8.5), dpi=300)
    ax = fig.add_axes([0, 0, 1, 1])
    ax.set_xlim(0, 1)
    ax.set_ylim(0, 1)
    ax.axis("off")
    ax.set_facecolor("#0a1428")
    x = np.linspace(0, 1, 300)
    ax.plot(x, 0.48 + 0.09 * np.exp(-2 * x) * np.cos(11 * x),
            color="#2ee6d6", lw=1.6)
    ax.text(0.5, 0.70,
            "Что доказывает язык,\nесли модель на нём\nпишет программу\nи запускает её?",
            ha="center", va="center", fontsize=14, color="#ffd97a")
    ax.text(0.5, 0.12, "Первое издание · 2026",
            ha="center", va="center", fontsize=10, color="#8899bb")
    fig.savefig(os.path.join(HERE, "back.png"), facecolor="#0a1428")
    plt.close()


if __name__ == "__main__":
    _front()
    _back()
    print("обложки готовы")
