#!/usr/bin/env python3
"""Generate 32x32.png icon for Tauri tray."""
from PIL import Image, ImageDraw

# Create a 32x32 PNG icon
size = 32
img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
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

img.save('D:/Projects/OtherProjects/ssh-proxy/src-tauri/icons/32x32.png', 'PNG')
print('32x32.png created successfully')