#!/usr/bin/env python3
"""Generate icon.ico from SVG for Tauri Windows app."""
from PIL import Image
import io
import os

# Try to use cairosvg for SVG conversion
try:
    import cairosvg
    USE_CAIROSVG = True
except ImportError:
    USE_CAIROSVG = False
    print("cairosvg not available, will generate simple icon")

# Paths
base_dir = os.path.dirname(os.path.abspath(__file__))
svg_path = os.path.join(base_dir, 'icon.svg')
ico_path = os.path.join(base_dir, 'icon.ico')

# Sizes for ICO file
sizes = [16, 32, 48, 64, 128, 256]
images = []

if USE_CAIROSVG and os.path.exists(svg_path):
    # Convert SVG to PNG at multiple sizes
    for size in sizes:
        png_data = cairosvg.svg2png(url=svg_path, output_width=size, output_height=size)
        img = Image.open(io.BytesIO(png_data))
        # Convert to RGBA if needed
        if img.mode != 'RGBA':
            img = img.convert('RGBA')
        images.append(img)
else:
    # Generate a simple colored icon without SVG conversion
    for size in sizes:
        # Create a blue rounded rectangle with white lines and colored dots
        img = Image.new('RGBA', (size, size), (0, 0, 0, 0))

        # Simple approach: create a basic icon
        from PIL import ImageDraw
        draw = ImageDraw.Draw(img)

        # Blue background with rounded corners
        radius = size // 8
        draw.rounded_rectangle([0, 0, size-1, size-1], radius=radius, fill=(0, 82, 217, 255))

        # White horizontal lines
        line_width = size // 16
        y_positions = [size//3, size//2, 2*size//3]
        for y in y_positions:
            draw.line([(size//4, y), (3*size//4, y)], fill=(255, 255, 255, 255), width=line_width)

        # Green dot on first line
        dot_radius = size // 10
        draw.ellipse([(3*size//4 - dot_radius, y_positions[0] - dot_radius),
                      (3*size//4 + dot_radius, y_positions[0] + dot_radius)],
                     fill=(0, 168, 112, 255))

        # Red dot on second line
        draw.ellipse([(5*size//8 - dot_radius, y_positions[1] - dot_radius),
                      (5*size//8 + dot_radius, y_positions[1] + dot_radius)],
                     fill=(227, 77, 89, 255))

        images.append(img)

# Save as ICO
if images:
    # PIL's ICO format requires specific handling
    # Save the largest image as ICO with embedded sizes
    largest = images[-1]  # 256x256

    # Create ICO with all sizes
    largest.save(ico_path, format='ICO', sizes=[(img.width, img.height) for img in images])
    print(f'icon.ico created at {ico_path}')
else:
    print('No images generated')