// 霧晶囊：水分改變花瓣、重量與聚集。時間與數值是展示設定。
let creatures = [], mist = [], waves = [];
let time = 0, rainLeft = 0, paused = false, inside = false, vortex = false;
let viewYaw = -.28, viewPitch = -.16, viewZoom = 1, viewMode = false;
let pointerDrag = null, ready = false;
const LIMIT = 28; // 避免畫面無限增生，保持展示穩定。

function setup() {
  let canvas;
  try {canvas = createCanvas(windowWidth, max(windowHeight, 540), WEBGL);}
  catch (error) {
    document.querySelector('#status').textContent = '3D 無法啟動：請使用支援 WebGL 的瀏覽器並開啟圖形加速';
    console.error(error); noLoop(); return;
  }
  canvas.parent('garden');
  pixelDensity(1);
  canvas.elt.addEventListener('pointerenter', () => inside = true);
  canvas.elt.addEventListener('pointerleave', () => inside = false);
  canvas.elt.addEventListener('pointerdown', event => {
    if (!viewMode) return;
    pointerDrag = {x:event.clientX, y:event.clientY};
    canvas.elt.setPointerCapture(event.pointerId);
  });
  canvas.elt.addEventListener('pointermove', event => {
    if (!viewMode || !pointerDrag) return;
    viewYaw += (event.clientX - pointerDrag.x) * .008;
    viewPitch = constrain(viewPitch + (event.clientY - pointerDrag.y) * .008, -1.1, 1.1);
    pointerDrag = {x:event.clientX, y:event.clientY};
  });
  canvas.elt.addEventListener('pointerup', () => pointerDrag = null);
  canvas.elt.addEventListener('pointercancel', () => pointerDrag = null);
  canvas.elt.addEventListener('wheel', event => {
    event.preventDefault();
    viewZoom = constrain(viewZoom * exp(-event.deltaY * .001), .65, 1.8);
    document.querySelector('#zoom').value = round(viewZoom * 100);
  }, {passive:false});
  document.querySelector('#view').onclick = () => {
    viewMode = !viewMode; pointerDrag = null;
    document.querySelector('#view').textContent = viewMode ? '回到水霧' : '轉動視角';
    document.querySelector('#view').setAttribute('aria-pressed', String(viewMode));
  };
  document.querySelector('#front').onclick = () => {viewYaw=0;viewPitch=0;viewZoom=1;document.querySelector('#zoom').value=100;};
  document.querySelector('#zoom').oninput = event => viewZoom = Number(event.target.value) / 100;
  document.querySelector('#rain').onclick = () => rainLeft = 8;
  document.querySelector('#vortex').onclick = () => {
    vortex = !vortex;
    document.querySelector('#vortex').textContent = vortex ? '讓風停下' : '喚起旋風';
    document.querySelector('#vortex').setAttribute('aria-pressed', String(vortex));
  };
  document.querySelector('#pause').onclick = togglePause;
  document.querySelector('#reset').onclick = resetGarden;
  resetGarden();
  ready = true;
}

function makeCreature(x, y, size = random(24, 35), z = random(-140, 140)) {
  return {x, y, z, vx: 0, vy: 0, vz: 0, size, water: .45, energy: .45,
    phase: random(TWO_PI), seed: random(1000), age: 0, bloom: .5};
}

function resetGarden() {
  paused = false;
  vortex = false;
  document.querySelector('#vortex').textContent = '喚起旋風';
  document.querySelector('#vortex').setAttribute('aria-pressed', 'false');
  document.querySelector('#pause').textContent = '暫停觀察';
  viewYaw=-.28;viewPitch=-.16;viewZoom=1;viewMode=false;pointerDrag=null;
  document.querySelector('#zoom').value=100;
  document.querySelector('#view').textContent='轉動視角';
  document.querySelector('#view').setAttribute('aria-pressed','false');
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

// 同一套旋轉也用來對齊滑鼠：轉過視角後，補水仍落在看得到的生物上。
function projectPoint(c) {
  const dx=c.x-width/2, dy=c.y-height/2;
  const x=cos(viewYaw)*dx+sin(viewYaw)*c.z;
  const z=-sin(viewYaw)*dx+cos(viewYaw)*c.z;
  return {x:width/2+x*viewZoom, y:height/2+(cos(viewPitch)*dy-sin(viewPitch)*z)*viewZoom,
    depth:sin(viewPitch)*dy+cos(viewPitch)*z};
}
function pointerInWorld(depth=0) {
  const x=(mouseX-width/2)/viewZoom, y=(mouseY-height/2)/viewZoom;
  const dy=cos(viewPitch)*y+sin(viewPitch)*depth;
  const z=-sin(viewPitch)*y+cos(viewPitch)*depth;
  return {x:width/2+cos(viewYaw)*x-sin(viewYaw)*z,y:height/2+dy,z:sin(viewYaw)*x+cos(viewYaw)*z};
}
function draw() {
  if (!ready) return;
  const dt = min(deltaTime / 1000, .05);
  if (!paused) {time += dt; updateGarden(dt);}
  background(9,15,32);
  camera(0,0,1400,0,0,0,0,1,0);
  ortho(-width/2,width/2,-height/2,height/2,1,3000);
  ambientLight(95,115,145);
  directionalLight(175,225,255,-.4,.2,-1);
  directionalLight(170,120,220,.8,-.4,.5);
  push();scale(viewZoom);rotateX(viewPitch);rotateY(viewYaw);translate(-width/2,-height/2,0);
  drawSky();
  for (let i = 0; i < creatures.length; i++) {
    for (let j = i + 1; j < creatures.length; j++) {
      const a = creatures[i], b = creatures[j], d = dist(a.x,a.y,a.z,b.x,b.y,b.z);
      if (d < 155 && a.water > .28 && b.water > .28) {
        stroke(160,218,230,(1-d/155)*80);strokeWeight(.7);
        line(a.x,a.y,a.z,b.x,b.y,b.z);
      }
    }
  }
  // 由遠到近畫半透明身體，減少前後遮擋的違和感。
  const ordered=[...creatures].sort((a,b)=>projectPoint(a).depth-projectPoint(b).depth);
  for (const c of ordered) drawCreature(c);
  noLights();noStroke();
  for (const m of mist) {push();translate(m.x,m.y,m.z);fill(171,229,248,m.life*115);sphere(m.r,5,3);pop();}
  for (const w of waves) {
    push();translate(w.x,w.y,w.z);noFill();stroke(217,192,255,max(0,1-w.r/350)*135);strokeWeight(1);
    circle(0,0,w.r*2);rotateX(HALF_PI);circle(0,0,w.r*2);rotateY(HALF_PI);circle(0,0,w.r*2);pop();
  }
  pop();
  if (frameCount % 20 === 0) {
    const water=round(creatures.reduce((sum,c)=>sum+c.water,0)/creatures.length*100);
    document.querySelector('#status').textContent = paused ? '時間停在這一刻' : rainLeft > 0 ? '雨中 · 飽水的花園正在下沉' :
      creatures.some(c=>c.water<.2) ? '乾燥中 · 花瓣收攏休眠' : '雲霧中 · '+creatures.length+' 個生命 · 含水 '+water+'%';
    document.querySelector('#garden').dataset.water=water;
    document.querySelector('#garden').dataset.population=creatures.length;
  }
}

function updateGarden(dt) {
  rainLeft = max(0, rainLeft - dt);
  if (inside && !viewMode && frameCount % 2 === 0) {
    const p=pointerInWorld();
    mist.push({x:p.x+random(-24,24),y:p.y+random(-24,24),z:p.z+random(-80,80),life:1,r:random(2,5)});
  }
  if (rainLeft > 0) for (let i = 0; i < 4; i++) mist.push({x:random(width), y:random(height),z:random(-180,180),life:1,r:random(2,4)});
  for (const m of mist) {m.y += dt * 9; m.x += dt * 5; m.life -= dt * .55;}
  mist = mist.filter(m => m.life > 0).slice(-350);
  for (const w of waves) w.r += dt * 150;
  waves = waves.filter(w => w.r < 350);
  const children = [];
  for (const c of creatures) {
    c.age += dt;
    const screen=projectPoint(c), target=pointerInWorld(screen.depth);
    const d=dist(screen.x,screen.y,mouseX,mouseY);
    const watering=inside && !viewMode && d<145*viewZoom;
    const wet = watering || rainLeft > 0;
    c.water = constrain(c.water + dt * (wet ? .15 : -.016), 0, 1);
    // 水霧代表溶解有機物的取得機會；能量值不是量測結果。
    c.energy = constrain(c.energy + dt * (wet ? .08 : -.008), 0, 1);
    c.bloom = lerp(c.bloom, c.water, dt * 2);
    let fx = (noise(c.seed, time * .08) - .5) * 22;
    let fy = (noise(c.seed + 20, time * .08) - .5) * 14;
    let fz = (noise(c.seed + 40,time * .08)-.5)*18-c.z*.025;
    if (watering && d > 25) {fx += (target.x-c.x)*.08;fy += (target.y-c.y)*.08;fz += (target.z-c.z)*.08;}
    // Shift 旋風：垂直於游標方向的力，讓群體繞行。
    const windX=inside && !viewMode ? target.x : width/2, windY=inside && !viewMode ? target.y : height/2;
    if ((vortex || (inside && !viewMode && keyIsDown(SHIFT))) && dist(c.x,c.y,windX,windY) < 230) {
      fx += -(windY - c.y) * .32; fy += (windX - c.x) * .32;
    }
    for (const w of waves) {
      const wd=dist(c.x,c.y,c.z,w.x,w.y,w.z);
      if (abs(wd - w.r) < 22 && wd > 1) {fx += (c.x - w.x) / wd * 85; fy += (c.y-w.y)/wd*85;fz += (c.z-w.z)/wd*85;}
    }
    // 輕微聚集與近距離排斥，保留每個個體的輪廓。
    for (const other of creatures) {
      if (other === c) continue;
      const dx=other.x-c.x,dy=other.y-c.y,dz=other.z-c.z,dd=max(1,sqrt(dx*dx+dy*dy+dz*dz));
      if (dd < 60) {fx -= dx / dd * 16; fy -= dy/dd*16;fz -= dz/dd*16;}
      else if (dd < 145 && c.water > .3) {fx += dx * .014; fy += dy*.014;fz += dz*.014;}
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
    c.vz = (c.vz + fz * dt) * exp(-dt * 1.5);
    c.x = constrain(c.x + c.vx * dt * activity, 25, width - 25);
    c.y = constrain(c.y + c.vy * dt * activity, height * .28, height - 160);
    c.z = constrain(c.z + c.vz * dt * activity,-190,190);
    // 能量充足才出芽，母體消耗資源；達上限停止增生。
    if (c.age > 18 && c.energy > .82 && c.water > .6 && creatures.length + children.length < LIMIT) {
      children.push(makeCreature(constrain(c.x+36,25,width-25),c.y,c.size*.65,constrain(c.z+random(-25,25),-190,190)));
      c.energy -= .45; c.water -= .16; c.age = 0;
    }
  }
  creatures.push(...children);
}

function drawSky() {
  // 有前後位置的微光，旋轉視角時能看出雲帶深度。
  noLights();strokeWeight(2);
  for (let i=0;i<100;i++) {
    const x=(noise(i*7)*width+time*(2+i%3))%width;
    const y=noise(i*7+2)*height,z=noise(i*7+4)*600-300;
    stroke(150,198,212,25+25*sin(time+i));point(x,y,z);
  }
  ambientLight(95,115,145);
  directionalLight(175,225,255,-.4,.2,-1);
  directionalLight(170,120,220,.8,-.4,.5);
}
function drawCreature(c) {
  push();translate(c.x,c.y,c.z);
  rotateX(.35+sin(time*.35+c.phase)*.25);rotateY(c.phase+sin(time*.22+c.phase)*.5);rotateZ(sin(time*.3+c.phase)*.24);
  const breath=1+sin(time*1.6+c.phase)*.065;
  const r=c.size*(.7+c.bloom*.6)*breath;
  const opening=.38+c.bloom*.8;
  // 五片晶瓣是有厚度的立體菱形，不是貼在空間裡的平面圖片。
  for (let i=0;i<5;i++) {
    push();rotateZ(i*TWO_PI/5);rotateY(.45-c.bloom*.7+sin(time+c.phase+i)*.06);
    fill(115,190+c.bloom*20,221,165);specularMaterial(110,150,180);shininess(45);noStroke();
    drawCrystal(r*(1.05+opening),r*.44*opening,r*.22);
    noLights();stroke(207,231,251,150);strokeWeight(.8);line(0,0,r*.23,r*(1.05+opening),0,0);pop();
  }
  // 核心先畫，外層半透明橢球殼再包住它。
  noStroke();push();fill(140,205,240);emissiveMaterial(65,100,145);sphere(5+c.energy*5,10,6);pop();
  fill(75,155,185,100);specularMaterial(100,150,180);shininess(70);ellipsoid(r*.84,r,r*.72,18,12);
  // 緩慢擺動的兩條感絲，讓側面也有生命感。
  noLights();noFill();stroke(183,215,245,110);strokeWeight(.9);
  for (let j=0;j<2;j++) {
    beginShape();for(let k=0;k<=12;k++) {const t=k/12;vertex((j-.5)*r*.45+sin(t*4+time+c.phase)*r*.15*t,r*(.6+t*1.2),cos(t*3+time+c.phase)*r*.35*t);}endShape();
  }
  pop();
}
function drawCrystal(length,wide,thick) {
  const v=[[0,0,0],[length*.57,-wide,0],[length,0,0],[length*.57,wide,0],[length*.5,0,thick],[length*.5,0,-thick]];
  // 上下各四個三角面，法向量讓燈光照出晶體切面。
  const faces=[[0,1,4],[1,2,4],[2,3,4],[3,0,4],[1,0,5],[2,1,5],[3,2,5],[0,3,5]];
  beginShape(TRIANGLES);
  for(const [a,b,c] of faces) {
    const u=v[b].map((n,i)=>n-v[a][i]),w=v[c].map((n,i)=>n-v[a][i]);
    const n=[u[1]*w[2]-u[2]*w[1],u[2]*w[0]-u[0]*w[2],u[0]*w[1]-u[1]*w[0]],d=Math.hypot(...n);
    normal(n[0]/d,n[1]/d,n[2]/d);for(const idx of [a,b,c])vertex(...v[idx]);
  }
  endShape();
}
function mousePressed(event) {
  if (!paused && !viewMode && event.target.tagName==='CANVAS' && event.button===0) {
    const p=pointerInWorld();waves.push({x:p.x,y:p.y,z:p.z,r:1});
  }
}

function keyPressed(event) {
  if (['BUTTON','INPUT'].includes(event.target.tagName)) return;
  if (key === ' ') {togglePause();return false;}
  if (key === 'r' || key === 'R') resetGarden();
}
function windowResized() {
  const oldW=width,oldH=height;resizeCanvas(windowWidth,max(windowHeight,540));
  for (const c of creatures) {c.x *= width/oldW;c.y *= height/oldH;}
}
