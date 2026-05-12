<script lang="ts">
  import { onMount } from "svelte";

  let canvas: HTMLCanvasElement | null = null;

  onMount(() => {
    if (!canvas) return;

    const context = canvas.getContext("2d");
    if (!context) return;
    const ctx = context;

    let width = window.innerWidth;
    let height = window.innerHeight;
    canvas.width = width;
    canvas.height = height;

    const onResize = () => {
      width = window.innerWidth;
      height = window.innerHeight;
      if (!canvas) return;
      canvas.width = width;
      canvas.height = height;
    };

    window.addEventListener("resize", onResize);

    const particles = Array.from({ length: 72 }, () => ({
      x: Math.random() * width,
      y: Math.random() * height,
      vx: (Math.random() - 0.5) * 0.28,
      vy: (Math.random() - 0.5) * 0.28
    }));

    let raf = 0;

    function tick() {
      ctx.clearRect(0, 0, width, height);
      const isLight = document.documentElement.dataset.theme === "light";
      const lineColor = isLight ? "20, 24, 34" : "255, 255, 255";

      for (const p of particles) {
        p.x += p.vx;
        p.y += p.vy;
        if (p.x < 0 || p.x > width) p.vx *= -1;
        if (p.y < 0 || p.y > height) p.vy *= -1;
      }

      for (let i = 0; i < particles.length; i++) {
        for (let j = i + 1; j < particles.length; j++) {
          const a = particles[i];
          const b = particles[j];
          const distance = Math.hypot(a.x - b.x, a.y - b.y);
          if (distance > 170) continue;
          const alpha = (1 - distance / 170) * 0.5;
          ctx.beginPath();
          ctx.moveTo(a.x, a.y);
          ctx.lineTo(b.x, b.y);
          ctx.strokeStyle = `rgba(${lineColor}, ${alpha * 0.62})`;
          ctx.lineWidth = 1;
          ctx.stroke();
        }
      }

      for (const p of particles) {
        ctx.beginPath();
        ctx.arc(p.x, p.y, 2.6, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(${lineColor}, ${isLight ? 0.2 : 0.26})`;
        ctx.fill();
      }

      raf = requestAnimationFrame(tick);
    }

    tick();

    const cubeEl = document.getElementById("landing-cube") as HTMLElement | null;
    let rx = 15;
    let ry = 0;
    let rz = 0;
    let vx = 0.18;
    let vy = 0.34;
    let vz = 0.08;
    let cubeRaf = 0;

    function tickCube() {
      rx += vx;
      ry += vy;
      rz += vz;
      if (cubeEl) cubeEl.style.transform = `rotateX(${rx}deg) rotateY(${ry}deg) rotateZ(${rz}deg)`;
      cubeRaf = requestAnimationFrame(tickCube);
    }

    if (cubeEl) tickCube();

    return () => {
      cancelAnimationFrame(raf);
      cancelAnimationFrame(cubeRaf);
      window.removeEventListener("resize", onResize);
    };
  });
</script>

<div class="pointer-events-none fixed inset-0 z-0 amethyst-bg"></div>
<canvas bind:this={canvas} class="pointer-events-none fixed inset-0 z-0"></canvas>
<div class="pointer-events-none fixed inset-0 z-[1] flex items-center justify-center">
  <div class="amethyst-cube-scene">
    <div id="landing-cube" class="amethyst-cube">
      <div class="amethyst-cube-face" style="transform: translateZ(180px);"></div>
      <div class="amethyst-cube-face" style="transform: rotateY(180deg) translateZ(180px);"></div>
      <div class="amethyst-cube-face" style="transform: rotateY(-90deg) translateZ(180px);"></div>
      <div class="amethyst-cube-face" style="transform: rotateY(90deg) translateZ(180px);"></div>
      <div class="amethyst-cube-face" style="transform: rotateX(90deg) translateZ(180px);"></div>
      <div class="amethyst-cube-face" style="transform: rotateX(-90deg) translateZ(180px);"></div>
    </div>
  </div>
</div>
<div class="pointer-events-none fixed inset-0 z-[1] amethyst-scanlines"></div>
