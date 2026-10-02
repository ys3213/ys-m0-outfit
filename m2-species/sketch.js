// 霧晶囊：水分改變花瓣、重量與聚集。時間與數值是展示設定。
let creatures = [], mist = [], waves = [];
let time = 0, rainLeft = 0, paused = false, inside = false, vortex = false;
const LIMIT = 28; // 避免畫面無限增生，保持展示穩定。

function setup() {
  const canvas = createCanvas(windowWidth, max(windowHeight, 540));
  canvas.parent('garden');
  pixelDensity(1);
  canvas.elt.addEventListener('pointerenter', () => inside = true);
  canvas.elt.addEventListener('pointerleave', () => inside = false);
  document.querySelector('#rain').onclick = () => rainLeft = 8;
  document.querySelector('#vortex').onclick = () => {
    vortex = !vortex;
    document.querySelector('#vortex').textContent = vortex ? '讓風停下' : '喚起旋風';
    document.querySelector('#vortex').setAttribute('aria-pressed', String(vortex));
  };
  document.querySelector('#pause').onclick = togglePause;
  document.querySelector('#reset').onclick = resetGarden;
  resetGarden();
}

function makeCreature(x, y, size = random(17, 29)) {
  return {x, y, vx: 0, vy: 0, size, water: .45, energy: .45,
    phase: random(TWO_PI), seed: random(1000), age: 0, bloom: .5};
}

function resetGarden() {
  paused = false;
  vortex = false;
  document.querySelector('#vortex').textContent = '喚起旋風';
  document.querySelector('#vortex').setAttribute('aria-pressed', 'false');
  document.querySelector('#pause').textContent = '暫停觀察';
  randomSeed(42); noiseSeed(42);
  creatures = []; mist = []; waves = []; time = 0; rainLeft = 0;
  for (let i = 0; i < 16; i++) {
    creatures.push(makeCreature(random(width * .18, width * .82), random(height * .3, height * .72)));
  }
}

function togglePause() {
  paused = !paused;
  document.querySelector('#pause').textContent = paused ? '繼續漂移' : '暫停觀察';
}

function draw() {
  const dt = min(deltaTime / 1000, .05);
  if (!paused) {time += dt; updateGarden(dt);}
  drawSky();
  // 鄰近個體用淡光連起來；不是硬合併成一個身體。
  for (let i = 0; i < creatures.length; i++) {
    for (let j = i + 1; j < creatures.length; j++) {
      const a = creatures[i], b = creatures[j], d = dist(a.x, a.y, b.x, b.y);
      if (d < 125 && a.water > .28 && b.water > .28) {
        stroke(160, 218, 230, (1 - d / 125) * 65); strokeWeight(.7);
        line(a.x, a.y, b.x, b.y);
      }
    }
  }
  for (const c of creatures) drawCreature(c);
  noStroke();
  for (const m of mist) {fill(171, 229, 248, m.life * 100); circle(m.x, m.y, m.r);}
  noFill();
  for (const w of waves) {stroke(217, 192, 255, max(0, 1 - w.r / 350) * 130);circle(w.x, w.y, w.r * 2);}
  if (inside) {stroke(206, 227, 255, 65);circle(mouseX, mouseY, 32 + sin(time * 2) * 5);}
  if (frameCount % 20 === 0) {
    document.querySelector('#status').textContent = paused ? '時間停在這一刻' : rainLeft > 0 ? '雨中 · 飽水的花園正在下沉' : creatures.some(c => c.water < .2) ? '乾燥中 · 花瓣收攏休眠' : '雲霧中 · ' + creatures.length + ' 個生命漂移著';
  }
}

function updateGarden(dt) {
  rainLeft = max(0, rainLeft - dt);
  if (inside && frameCount % 2 === 0) {
    mist.push({x: mouseX + random(-24, 24), y: mouseY + random(-24, 24), life: 1, r: random(2, 5)});
  }
  if (rainLeft > 0) for (let i = 0; i < 4; i++) mist.push({x:random(width), y:random(height),life:1,r:random(2,4)});
  for (const m of mist) {m.y += dt * 9; m.x += dt * 5; m.life -= dt * .55;}
  mist = mist.filter(m => m.life > 0).slice(-350);
  for (const w of waves) w.r += dt * 150;
  waves = waves.filter(w => w.r < 350);
  const children = [];
  for (const c of creatures) {
    c.age += dt;
    const d = dist(c.x, c.y, mouseX, mouseY);
    const watering = inside && d < 145;
    const wet = watering || rainLeft > 0;
    c.water = constrain(c.water + dt * (wet ? .15 : -.016), 0, 1);
    // 水霧代表溶解有機物的取得機會；能量值不是量測結果。
    c.energy = constrain(c.energy + dt * (wet ? .08 : -.008), 0, 1);
    c.bloom = lerp(c.bloom, c.water, dt * 2);
    let fx = (noise(c.seed, time * .08) - .5) * 22;
    let fy = (noise(c.seed + 20, time * .08) - .5) * 14;
    if (watering && d > 25) {fx += (mouseX - c.x) * .08; fy += (mouseY - c.y) * .08;}
    // Shift 旋風：垂直於游標方向的力，讓群體繞行。
    const windX = inside ? mouseX : width / 2, windY = inside ? mouseY : height / 2;
    if ((vortex || (inside && keyIsDown(SHIFT))) && dist(c.x,c.y,windX,windY) < 230) {
      fx += -(windY - c.y) * .32; fy += (windX - c.x) * .32;
    }
    for (const w of waves) {
      const wd = dist(c.x, c.y, w.x, w.y);
      if (abs(wd - w.r) < 22 && wd > 1) {fx += (c.x - w.x) / wd * 85; fy += (c.y - w.y) / wd * 85;}
    }
    // 輕微聚集與近距離排斥，保留每個個體的輪廓。
    for (const other of creatures) {
      if (other === c) continue;
      const dx = other.x - c.x, dy = other.y - c.y, dd = max(1, sqrt(dx * dx + dy * dy));
      if (dd < 60) {fx -= dx / dd * 16; fy -= dy / dd * 16;}
      else if (dd < 145 && c.water > .3) {fx += dx * .014; fy += dy * .014;}
    }
    // 飽水增加沉降；雲帶底部的上升氣流將個體托回來。
    fy += max(0, c.water - .73) * 100;
    if (c.y > height * .72) fy -= (c.y - height * .72) * .7;
    if (c.y < height * .24) fy += (height * .24 - c.y) * .5;
    if (c.x < 70) fx += (70 - c.x) * .4;
    if (c.x > width - 70) fx -= (c.x - width + 70) * .4;
    const activity = c.water < .2 ? .2 : 1;
    c.vx = (c.vx + fx * dt) * exp(-dt * 1.5);
    c.vy = (c.vy + fy * dt) * exp(-dt * 1.5);
    c.x = constrain(c.x + c.vx * dt * activity, 25, width - 25);
    c.y = constrain(c.y + c.vy * dt * activity, height * .2, height - 140);
    // 能量充足才出芽，母體消耗資源；達上限停止增生。
    if (c.age > 18 && c.energy > .82 && c.water > .6 && creatures.length + children.length < LIMIT) {
      children.push(makeCreature(c.x + 36, c.y, c.size * .65));
      c.energy -= .45; c.water -= .16; c.age = 0;
    }
  }
  creatures.push(...children);
}

function drawSky() {
  background(9, 15, 32);
  noStroke();
  for (let y = 0; y < height; y += 6) {
    const glow = exp(-pow((y / height - .6) / .3, 2));
    fill(15 + glow * 14, 22 + glow * 18, 40 + glow * 22);rect(0,y,width,6);
  }
  for (let i = 0; i < 65; i++) {
    const x = (noise(i * 7) * width + time * (2 + i % 3)) % width;
    const y = noise(i * 7 + 2) * height;
    fill(150, 198, 212, 16 + 16 * sin(time + i)); circle(x,y,1.8);
  }
}

function drawCreature(c) {
  push(); translate(c.x,c.y);rotate(sin(time * .3 + c.phase) * .24);
  const breath = 1 + sin(time * 1.6 + c.phase) * .065;
  const r = c.size * (.7 + c.bloom * .6) * breath;
  const petals = 5, opening = .38 + c.bloom * .8;
  // 淡淡的外暈、五片晶瓣、中央膜囊。
  noStroke();fill(147,174,244,9);circle(0,0,r*4.1);
  for (let i = 0; i < petals; i++) {
    push();rotate(i * TWO_PI / petals + time * .025);
    const reach = r * (1.05 + opening);
    fill(175,200 + c.bloom*30,240,20 + c.bloom * 18);stroke(171,219,242,95);strokeWeight(.8);
    beginShape();vertex(r*.12,0);vertex(reach*.65,-r*.4*opening);vertex(reach,0);vertex(reach*.65,r*.4*opening);endShape(CLOSE);
    stroke(218,202,251,85);line(r*.3,0,reach,0);pop();
  }
  fill(130,205,224,24);stroke(199,229,240,145);ellipse(0,0,r*1.8,r*2);
  noFill();stroke(234,214,255,65);arc(-r*.15,-r*.12,r*1.3,r*1.5,PI,TWO_PI);
  noStroke();fill(225,225,255,130 + sin(time*2+c.phase)*35);circle(r*.08,-r*.1,4+c.energy*6);
  fill(180,224,229,120);circle(-r*.4,-r*.5,3);pop();
}

function mousePressed(event) {
  if (event.target.tagName === 'CANVAS') waves.push({x:mouseX,y:mouseY,r:1});
}
function keyPressed(event) {
  if (event.target.tagName === 'BUTTON') return;
  if (key === ' ') {togglePause();return false;}
  if (key === 'r' || key === 'R') resetGarden();
}
function windowResized() {
  const oldW=width,oldH=height;resizeCanvas(windowWidth,max(windowHeight,540));
  for (const c of creatures) {c.x *= width/oldW;c.y *= height/oldH;}
}
