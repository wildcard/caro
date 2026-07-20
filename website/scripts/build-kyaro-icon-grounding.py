#!/usr/bin/env python3
"""Build deterministic Kyaro grounding sheets from every source animation frame."""

from __future__ import annotations

import json
import re
from collections import Counter
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "assets" / "kyaro"
OUTPUT = ROOT / "website" / "docs" / "brand" / "kyaro-icon-system"

STATE_NAMES = {
    "001-idle": "Idle",
    "002-blink": "Blink",
    "003-sleeping": "Sleeping",
    "004-prompt bubble": "Prompt bubble",
    "005-walking": "Walking",
    "006-happy bounce": "Happy bounce",
    "007-pooping": "Pooping",
    "008-shocked": "Shocked",
    "009-upside down": "Upside down",
}


def natural_key(path: Path) -> list[object]:
    return [
        int(part) if part.isdigit() else part.lower()
        for part in re.split(r"(\d+)", path.name)
    ]


def load_font(size: int, bold: bool = False) -> ImageFont.ImageFont:
    candidates = [
        ROOT / "website" / "public" / "fonts" / (
            "Figtree-Bold.ttf" if bold else "Figtree-Regular.ttf"
        ),
        Path("/System/Library/Fonts/SFNS.ttf"),
    ]
    for candidate in candidates:
        if candidate.exists():
            return ImageFont.truetype(str(candidate), size)
    return ImageFont.load_default()


def frames_by_state() -> list[tuple[str, list[Path]]]:
    states: list[tuple[str, list[Path]]] = []
    for state_dir in sorted(path for path in SOURCE.iterdir() if path.is_dir()):
        frames = sorted(state_dir.rglob("*.png"), key=natural_key)
        if frames:
            states.append((STATE_NAMES.get(state_dir.name, state_dir.name), frames))
    return states


def checkerboard(size: tuple[int, int], tile: int = 8) -> Image.Image:
    image = Image.new("RGBA", size, "#f4f1df")
    draw = ImageDraw.Draw(image)
    for y in range(0, size[1], tile):
        for x in range(0, size[0], tile):
            if (x // tile + y // tile) % 2:
                draw.rectangle((x, y, x + tile - 1, y + tile - 1), fill="#e7e4d5")
    return image


def frame_tile(path: Path, size: int = 96) -> Image.Image:
    source = Image.open(path).convert("RGBA")
    scaled = source.resize((size, size), Image.Resampling.NEAREST)
    tile = checkerboard((size, size))
    tile.alpha_composite(scaled)
    return tile


def build_full_sheet(states: list[tuple[str, list[Path]]]) -> None:
    label_width = 176
    tile_size = 96
    gap = 10
    max_frames = max(len(frames) for _, frames in states)
    width = label_width + gap + max_frames * (tile_size + gap) + 24
    row_height = tile_size + 44
    height = 88 + len(states) * row_height + 24
    sheet = Image.new("RGBA", (width, height), "#faf8ec")
    draw = ImageDraw.Draw(sheet)
    title_font = load_font(28, bold=True)
    label_font = load_font(18, bold=True)
    meta_font = load_font(13)
    draw.text((24, 20), "Kyaro — all-state grounding sheet", fill="#2b2b2b", font=title_font)
    draw.text(
        (24, 55),
        f"{sum(len(frames) for _, frames in states)} source frames • nearest-neighbour • no generated art",
        fill="#6b6b6b",
        font=meta_font,
    )
    y = 88
    for state, frames in states:
        draw.text((24, y + 26), state, fill="#2b2b2b", font=label_font)
        draw.text((24, y + 52), f"{len(frames)} frames", fill="#7a7a7a", font=meta_font)
        x = label_width + gap
        for index, frame in enumerate(frames, start=1):
            sheet.alpha_composite(frame_tile(frame, tile_size), (x, y))
            draw.rectangle((x, y, x + tile_size - 1, y + tile_size - 1), outline="#c9c7c1")
            draw.text((x + 4, y + tile_size + 5), f"{index:02}", fill="#7a7a7a", font=meta_font)
            x += tile_size + gap
        y += row_height
    sheet.convert("RGB").save(OUTPUT / "kyaro-all-frames-contact-sheet.jpg", quality=94)


def representative_indices(count: int) -> list[int]:
    if count <= 3:
        return list(range(count))
    return sorted({0, count // 2, count - 1})


def build_representative_sheet(states: list[tuple[str, list[Path]]]) -> None:
    card_width = 380
    card_height = 190
    columns = 3
    rows = (len(states) + columns - 1) // columns
    sheet = Image.new("RGBA", (columns * card_width + 48, rows * card_height + 120), "#faf8ec")
    draw = ImageDraw.Draw(sheet)
    title_font = load_font(30, bold=True)
    label_font = load_font(18, bold=True)
    meta_font = load_font(13)
    draw.text((24, 20), "Kyaro identity across every state", fill="#2b2b2b", font=title_font)
    draw.text(
        (24, 62),
        "First • middle • final frames expose stable anatomy, markings, posture, and expression.",
        fill="#6b6b6b",
        font=meta_font,
    )
    for state_index, (state, frames) in enumerate(states):
        column = state_index % columns
        row = state_index // columns
        x0 = 24 + column * card_width
        y0 = 104 + row * card_height
        draw.rounded_rectangle(
            (x0, y0, x0 + card_width - 16, y0 + card_height - 16),
            radius=12,
            fill="#f4f1df",
            outline="#c9c7c1",
            width=2,
        )
        draw.text((x0 + 16, y0 + 12), state, fill="#2b2b2b", font=label_font)
        draw.text((x0 + 16, y0 + 38), f"{len(frames)} source frames", fill="#7a7a7a", font=meta_font)
        for slot, index in enumerate(representative_indices(len(frames))):
            tile = frame_tile(frames[index], 96)
            x = x0 + 16 + slot * 112
            y = y0 + 64
            sheet.alpha_composite(tile, (x, y))
            draw.rectangle((x, y, x + 95, y + 95), outline="#c9c7c1")
            draw.text((x + 4, y + 76), f"{index + 1:02}", fill="#4f4f4f", font=meta_font)
    sheet.convert("RGB").save(OUTPUT / "kyaro-state-grounding.jpg", quality=95)


def build_palette(states: list[tuple[str, list[Path]]]) -> None:
    colours: Counter[tuple[int, int, int]] = Counter()
    for _, frames in states:
        for path in frames:
            image = Image.open(path).convert("RGBA")
            for red, green, blue, alpha in image.get_flattened_data():
                if alpha > 180:
                    colours[(red, green, blue)] += 1

    palette = [
        {"hex": f"#{red:02X}{green:02X}{blue:02X}", "pixels": count}
        for (red, green, blue), count in colours.most_common(16)
    ]
    (OUTPUT / "kyaro-source-palette.json").write_text(
        json.dumps(
            {
                "source": "All opaque pixels from all Kyaro PNG animation frames",
                "frame_count": sum(len(frames) for _, frames in states),
                "top_colours": palette,
            },
            indent=2,
        )
        + "\n"
    )


def main() -> None:
    OUTPUT.mkdir(parents=True, exist_ok=True)
    states = frames_by_state()
    build_full_sheet(states)
    build_representative_sheet(states)
    build_palette(states)
    print(f"states={len(states)} frames={sum(len(frames) for _, frames in states)}")
    print(f"output={OUTPUT}")


if __name__ == "__main__":
    main()
