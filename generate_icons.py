#!/usr/bin/env python3
from PIL import Image, ImageDraw
import math

def create_ios_icon(size):
    # 创建图像
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    # iOS 圆角比例
    radius = int(size * 0.225)
    
    # 绘制圆角矩形背景（渐变效果用纯色代替）
    draw.rounded_rectangle([(0, 0), (size, size)], radius=radius, fill='#667eea')
    
    # 绘制工具图标
    center = size // 2
    
    # 扳手（简化版）
    wrench_width = int(size * 0.16)
    wrench_height = int(size * 0.55)
    wrench_x = center - wrench_width // 2 - int(size * 0.1)
    wrench_y = center - wrench_height // 2
    draw.rounded_rectangle(
        [(wrench_x, wrench_y), (wrench_x + wrench_width, wrench_y + wrench_height)],
        radius=int(size * 0.08),
        fill='white'
    )
    
    # 螺丝刀（简化版）
    screwdriver_width = int(size * 0.12)
    screwdriver_height = int(size * 0.59)
    screwdriver_x = center - screwdriver_width // 2 + int(size * 0.1)
    screwdriver_y = center - screwdriver_height // 2
    draw.rounded_rectangle(
        [(screwdriver_x, screwdriver_y), (screwdriver_x + screwdriver_width, screwdriver_y + screwdriver_height)],
        radius=int(size * 0.06),
        fill='white'
    )
    
    # 中心圆
    circle_radius = int(size * 0.1)
    draw.ellipse(
        [(center - circle_radius, center - circle_radius), 
         (center + circle_radius, center + circle_radius)],
        fill='#764ba2'
    )
    
    return img

# 生成所有尺寸的图标
icons = [
    (32, 'src-tauri/icons/32x32.png'),
    (128, 'src-tauri/icons/128x128.png'),
    (256, 'src-tauri/icons/128x128@2x.png'),
    (256, 'src-tauri/icons/256x256.png'),
    (1024, 'src-tauri/icons/icon.png'),
]

for size, filename in icons:
    img = create_ios_icon(size)
    img.save(filename)
    print(f'生成: {filename}')

print('所有图标生成完成！')
