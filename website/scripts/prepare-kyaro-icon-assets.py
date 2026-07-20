#!/usr/bin/env python3
"""Extract genuine transparency and normalize generated Kyaro icon PNGs."""

from __future__ import annotations

import argparse
import json
from collections import deque
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


def is_background(pixel: tuple[int, int, int, int]) -> bool:
    red, green, blue, _ = pixel
    return min(red, green, blue) >= 224 and max(red, green, blue) - min(red, green, blue) <= 12


def extract_foreground(source: Path) -> Image.Image:
    image = Image.open(source).convert("RGBA")
    width, height = image.size
    pixels = image.load()
    background = bytearray(width * height)
    queue: deque[tuple[int, int]] = deque()

    def enqueue(x: int, y: int) -> None:
        index = y * width + x
        if not background[index] and is_background(pixels[x, y]):
            background[index] = 1
            queue.append((x, y))

    for x in range(width):
        enqueue(x, 0)
        enqueue(x, height - 1)
    for y in range(height):
        enqueue(0, y)
        enqueue(width - 1, y)

    while queue:
        x, y = queue.popleft()
        if x:
            enqueue(x - 1, y)
        if x + 1 < width:
            enqueue(x + 1, y)
        if y:
            enqueue(x, y - 1)
        if y + 1 < height:
            enqueue(x, y + 1)

    output = image.copy()
    output_pixels = output.load()
    for y in range(height):
        row = y * width
        for x in range(width):
            if background[row + x]:
                red, green, blue, _ = output_pixels[x, y]
                output_pixels[x, y] = (red, green, blue, 0)
    return output


def normalize(image: Image.Image, canvas_size: int = 1024) -> Image.Image:
    alpha = image.getchannel("A")
    box = alpha.getbbox()
    if not box:
        raise ValueError("No foreground pixels found")

    foreground = image.crop(box)
    target = int(canvas_size * 0.86)
    scale = min(target / foreground.width, target / foreground.height)
    resized = foreground.resize(
        (round(foreground.width * scale), round(foreground.height * scale)),
        Image.Resampling.LANCZOS,
    )
    canvas = Image.new("RGBA", (canvas_size, canvas_size), (0, 0, 0, 0))
    x = (canvas_size - resized.width) // 2
    y = (canvas_size - resized.height) // 2
    canvas.alpha_composite(resized, (x, y))
    return canvas


def checker(size: tuple[int, int], tile: int = 24) -> Image.Image:
    image = Image.new("RGB", size, "#f4f1df")
    draw = ImageDraw.Draw(image)
    for y in range(0, size[1], tile):
        for x in range(0, size[0], tile):
            if (x // tile + y // tile) % 2:
                draw.rectangle((x, y, x + tile - 1, y + tile - 1), fill="#e7e4d5")
    return image


def contact_sheet(assets: dict[str, Image.Image], output: Path) -> None:
    card = 420
    margin = 36
    sheet = Image.new("RGB", (margin * 2 + card * 3, 560), "#faf8ec")
    draw = ImageDraw.Draw(sheet)
    try:
        font = ImageFont.truetype(
            str(Path(__file__).parents[1] / "public/fonts/Figtree-Bold.ttf"),
            28,
        )
    except OSError:
        font = ImageFont.load_default()

    for index, (name, image) in enumerate(assets.items()):
        x = margin + index * card
        preview = checker((360, 360))
        scaled = image.resize((360, 360), Image.Resampling.LANCZOS)
        preview.paste(scaled, (0, 0), scaled)
        sheet.paste(preview, (x + 30, 52))
        draw.text((x + 30, 432), name.title(), fill="#2b2b2b", font=font)
        draw.text((x + 30, 476), "1024×1024 PNG • genuine alpha", fill="#6b6b6b")
    sheet.save(output, quality=95)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--idle", type=Path, required=True)
    parser.add_argument("--happy", type=Path, required=True)
    parser.add_argument("--alert", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--qa-dir", type=Path, required=True)
    args = parser.parse_args()

    args.out_dir.mkdir(parents=True, exist_ok=True)
    args.qa_dir.mkdir(parents=True, exist_ok=True)
    sources = {"idle": args.idle, "happy": args.happy, "alert": args.alert}
    assets: dict[str, Image.Image] = {}
    provenance: dict[str, object] = {"generator": "gpt-image-1.5", "assets": {}}

    for name, source in sources.items():
        asset = normalize(extract_foreground(source))
        target = args.out_dir / f"kyaro-{name}.png"
        web_target = args.out_dir / f"kyaro-{name}-web.png"
        asset.save(target, optimize=True)
        asset.resize((256, 256), Image.Resampling.LANCZOS).save(web_target, optimize=True)
        alpha = asset.getchannel("A")
        assets[name] = asset
        provenance["assets"][name] = {
            "source": str(source),
            "output": str(target),
            "web_output": str(web_target),
            "size": list(asset.size),
            "web_size": [256, 256],
            "alpha_extrema": list(alpha.getextrema()),
            "foreground_bbox": list(alpha.getbbox() or ()),
        }

    contact_sheet(assets, args.qa_dir / "kyaro-generated-icon-family.jpg")
    (args.qa_dir / "kyaro-generated-icon-provenance.json").write_text(
        json.dumps(provenance, indent=2) + "\n"
    )


if __name__ == "__main__":
    main()
