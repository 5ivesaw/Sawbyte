"""Convert the supplied icon into platform resources on GitHub Actions."""
from pathlib import Path
from PIL import Image

root = Path(__file__).resolve().parents[1]
output = root / "packaging/generated"
output.mkdir(parents=True, exist_ok=True)
with Image.open(root / "assets/sawbyte.png") as source:
    icon = source.convert("RGBA")
    icon.save(output / "sawbyte.ico", sizes=[(n, n) for n in (16, 24, 32, 48, 64, 128, 256)])
    icon.resize((1024, 1024), Image.Resampling.LANCZOS).save(output / "sawbyte.icns")
