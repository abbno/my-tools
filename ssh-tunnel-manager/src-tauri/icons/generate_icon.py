# PNG 图标生成脚本
# 这是一个 32x32 的简单 SSH 隧道图标

import struct
import zlib

def create_png_icon():
    width, height = 32, 32

    # 创建像素数据 (RGBA)
    pixels = []
    for y in range(height):
        row = []
        for x in range(width):
            # 创建一个简单的图标：中心是绿色圆形（代表隧道连接），边缘是深色
            cx, cy = width // 2, height // 2
            dist = ((x - cx) ** 2 + (y - cy) ** 2) ** 0.5

            if dist < 10:
                # 内圈：绿色（代表连接/隧道）
                r, g, b, a = 46, 139, 87, 255  # Sea green
            elif dist < 14:
                # 外圈：深蓝色边框
                r, g, b, a = 30, 60, 90, 255
            else:
                # 透明背景
                r, g, b, a = 0, 0, 0, 0

            row.extend([r, g, b, a])
        pixels.append(row)

    # 构建 PNG 文件
    def make_chunk(chunk_type, data):
        chunk = chunk_type + data
        crc = zlib.crc32(chunk) & 0xffffffff
        return struct.pack('>I', len(data)) + chunk + struct.pack('>I', crc)

    # PNG 签名
    signature = b'\x89PNG\r\n\x1a\n'

    # IHDR chunk
    ihdr_data = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)
    ihdr = make_chunk(b'IHDR', ihdr_data)

    # IDAT chunk (图像数据)
    raw_data = b''
    for row in pixels:
        raw_data += b'\x00'  # 过滤器类型 (None)
        for pixel in row:
            raw_data += bytes([pixel])

    compressed = zlib.compress(raw_data, 9)
    idat = make_chunk(b'IDAT', compressed)

    # IEND chunk
    iend = make_chunk(b'IEND', b'')

    return signature + ihdr + idat + iend

# 生成图标
png_data = create_png_icon()

# 输出为 base64 以便写入
import base64
print(base64.b64encode(png_data).decode())
