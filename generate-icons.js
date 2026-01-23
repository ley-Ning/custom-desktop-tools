import fs from 'fs';
import { createCanvas } from 'canvas';

function roundRect(ctx, x, y, width, height, radius) {
    ctx.beginPath();
    ctx.moveTo(x + radius, y);
    ctx.lineTo(x + width - radius, y);
    ctx.quadraticCurveTo(x + width, y, x + width, y + radius);
    ctx.lineTo(x + width, y + height - radius);
    ctx.quadraticCurveTo(x + width, y + height, x + width - radius, y + height);
    ctx.lineTo(x + radius, y + height);
    ctx.quadraticCurveTo(x, y + height, x, y + height - radius);
    ctx.lineTo(x, y + radius);
    ctx.quadraticCurveTo(x, y, x + radius, y);
    ctx.closePath();
}

function generateIcon(size) {
    const canvas = createCanvas(size, size);
    const ctx = canvas.getContext('2d');
    
    // 背景渐变
    const gradient = ctx.createLinearGradient(0, 0, size, size);
    gradient.addColorStop(0, '#667eea');
    gradient.addColorStop(1, '#764ba2');
    
    // 圆角矩形背景
    const radius = size * 0.225;
    ctx.fillStyle = gradient;
    roundRect(ctx, 0, 0, size, size, radius);
    ctx.fill();
    
    // 高光效果
    const highlightGradient = ctx.createRadialGradient(size/2, size*0.2, 0, size/2, size*0.2, size*0.4);
    highlightGradient.addColorStop(0, 'rgba(255,255,255,0.3)');
    highlightGradient.addColorStop(1, 'rgba(255,255,255,0)');
    ctx.fillStyle = highlightGradient;
    ctx.fillRect(0, 0, size, size);
    
    // 工具图标 - 扳手
    ctx.save();
    ctx.translate(size/2, size/2);
    ctx.rotate(-Math.PI/6);
    ctx.fillStyle = 'rgba(255,255,255,0.95)';
    
    roundRect(ctx, -size*0.08, -size*0.35, size*0.16, size*0.55, size*0.08);
    ctx.fill();
    
    ctx.beginPath();
    ctx.arc(0, -size*0.35, size*0.1, 0, Math.PI * 2);
    ctx.fill();
    
    roundRect(ctx, -size*0.12, size*0.12, size*0.24, size*0.16, size*0.03);
    ctx.fill();
    
    ctx.restore();
    
    // 螺丝刀
    ctx.save();
    ctx.translate(size/2, size/2);
    ctx.rotate(Math.PI/6);
    ctx.fillStyle = 'rgba(255,255,255,0.95)';
    
    roundRect(ctx, -size*0.06, -size*0.39, size*0.12, size*0.59, size*0.06);
    ctx.fill();
    
    ctx.beginPath();
    ctx.moveTo(0, -size*0.43);
    ctx.lineTo(-size*0.08, -size*0.35);
    ctx.lineTo(size*0.08, -size*0.35);
    ctx.closePath();
    ctx.fill();
    
    ctx.fillStyle = '#FFD700';
    roundRect(ctx, -size*0.05, size*0.16, size*0.1, size*0.2, size*0.02);
    ctx.fill();
    
    ctx.restore();
    
    // 中心装饰
    ctx.fillStyle = 'rgba(255,255,255,0.9)';
    ctx.beginPath();
    ctx.arc(size/2, size/2, size*0.14, 0, Math.PI * 2);
    ctx.fill();
    
    const centerGradient = ctx.createLinearGradient(size/2, size*0.36, size/2, size*0.64);
    centerGradient.addColorStop(0, '#667eea');
    centerGradient.addColorStop(1, '#764ba2');
    ctx.fillStyle = centerGradient;
    ctx.beginPath();
    ctx.arc(size/2, size/2, size*0.1, 0, Math.PI * 2);
    ctx.fill();
    
    ctx.fillStyle = 'rgba(255,255,255,0.5)';
    ctx.beginPath();
    ctx.arc(size/2, size/2, size*0.06, 0, Math.PI * 2);
    ctx.fill();
    
    return canvas;
}

// 生成图标
const icons = [
    { size: 32, name: '32x32.png' },
    { size: 128, name: '128x128.png' },
    { size: 256, name: '128x128@2x.png' },
    { size: 256, name: '256x256.png' },
    { size: 1024, name: 'icon.png' }
];

icons.forEach(({ size, name }) => {
    const canvas = generateIcon(size);
    const buffer = canvas.toBuffer('image/png');
    fs.writeFileSync(`src-tauri/icons/${name}`, buffer);
    console.log(`生成: ${name}`);
});

console.log('所有图标生成完成！');
