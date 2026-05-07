<template>
  <section class="splash-screen" aria-label="Minimart POS loading">
    <div class="splash-mark" aria-hidden="true">
      <svg viewBox="0 0 96 96" role="img">
        <path class="mark-roof" d="M18 31 28 17h40l10 14" />
        <path class="mark-awning" d="M18 31h60v13H18z" />
        <path class="mark-stripe" d="M30 31v13M42 31v13M54 31v13M66 31v13" />
        <path class="mark-shop" d="M24 44h48v34H24z" />
        <path class="mark-door" d="M43 55h11v23H43z" />
        <path class="mark-window" d="M31 54h8v9h-8zM58 54h8v9h-8z" />
        <path class="mark-basket" d="M34 73h29l4-15H30z" />
        <path class="mark-handle" d="M38 58c2-8 18-8 20 0" />
        <circle class="mark-wheel" cx="39" cy="80" r="3" />
        <circle class="mark-wheel" cx="61" cy="80" r="3" />
      </svg>
    </div>
    <div class="splash-copy">
      <strong>Minimart POS</strong>
      <span>Starting retail workspace</span>
    </div>
    <div class="splash-loader" aria-hidden="true">
      <span></span>
      <span></span>
      <span></span>
    </div>
  </section>
</template>

<style scoped>
.splash-screen {
  min-height: 100vh;
  display: grid;
  place-items: center;
  align-content: center;
  gap: 22px;
  background: #0a0a0a;
  color: #ffffff;
}

.splash-mark {
  width: 118px;
  height: 118px;
  display: grid;
  place-items: center;
  border: 1px solid rgba(212, 175, 55, 0.5);
  border-radius: 8px;
  background: #151515;
  box-shadow: 0 18px 50px rgba(212, 175, 55, 0.18);
}

.splash-mark svg {
  width: 82px;
  height: 82px;
  fill: none;
  stroke: #d4af37;
  stroke-width: 4;
  stroke-linejoin: round;
  stroke-linecap: round;
}

.mark-awning,
.mark-basket {
  fill: rgba(212, 175, 55, 0.18);
}

.mark-roof,
.mark-shop,
.mark-door,
.mark-window {
  stroke: #f2df9b;
}

.splash-copy {
  display: grid;
  gap: 7px;
  text-align: center;
}

.splash-copy strong {
  color: #d4af37;
  font-size: 1.8rem;
  line-height: 1;
}

.splash-copy span {
  color: #f8f5ed;
  font-size: 0.95rem;
}

.splash-loader {
  display: flex;
  gap: 8px;
}

.splash-loader span {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #d4af37;
  animation: splash-pulse 0.9s ease-in-out infinite;
}

.splash-loader span:nth-child(2) {
  animation-delay: 0.15s;
}

.splash-loader span:nth-child(3) {
  animation-delay: 0.3s;
}

@keyframes splash-pulse {
  0%, 100% {
    opacity: 0.35;
    transform: translateY(0);
  }

  50% {
    opacity: 1;
    transform: translateY(-5px);
  }
}
</style>
