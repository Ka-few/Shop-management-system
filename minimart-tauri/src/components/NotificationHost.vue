<template>
  <Teleport to="body">
    <div class="toast-stack" aria-live="polite" aria-atomic="true">
      <article
        v-for="toast in notifications.toasts"
        :key="toast.id"
        class="toast"
        :class="toast.type"
      >
        <div>
          <strong>{{ toast.title }}</strong>
          <p v-if="toast.message">{{ toast.message }}</p>
        </div>
        <button type="button" @click="dismissToast(toast.id)" aria-label="Dismiss notification">x</button>
      </article>
    </div>

    <div v-if="notifications.prompt.visible" class="prompt-backdrop" role="dialog" aria-modal="true">
      <section class="prompt-card" :class="notifications.prompt.type">
        <div>
          <strong>{{ notifications.prompt.title }}</strong>
          <p>{{ notifications.prompt.message }}</p>
        </div>
        <div class="prompt-actions">
          <button type="button" class="ghost-btn" @click="closePrompt(false)">
            {{ notifications.prompt.cancelText }}
          </button>
          <button type="button" class="confirm-btn" @click="closePrompt(true)">
            {{ notifications.prompt.confirmText }}
          </button>
        </div>
      </section>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { useNotifications } from '../composables/useNotifications'

const { notifications, dismissToast, closePrompt } = useNotifications()
</script>

<style scoped>
.toast-stack {
  position: fixed;
  right: 18px;
  top: 18px;
  z-index: 60;
  display: grid;
  gap: 10px;
  width: min(380px, calc(100vw - 36px));
}

.toast {
  display: flex;
  justify-content: space-between;
  gap: 14px;
  padding: 14px;
  background: #0a0a0a;
  color: #ffffff;
  border: 1px solid #d4af37;
  border-left-width: 5px;
  border-radius: 8px;
  box-shadow: 0 18px 45px rgba(0, 0, 0, 0.28);
}

.toast.success { border-left-color: #d4af37; }
.toast.info { border-left-color: #ffffff; }
.toast.warning { border-left-color: #f2c94c; }
.toast.error { border-left-color: #b91c1c; }

.toast strong {
  display: block;
  font-size: 0.95rem;
}

.toast p {
  margin-top: 4px;
  color: #e5e0d2;
  font-size: 0.9rem;
  line-height: 1.35;
}

.toast button {
  width: 28px;
  height: 28px;
  flex: 0 0 auto;
  border: 1px solid rgba(212, 175, 55, 0.55);
  border-radius: 6px;
  background: transparent;
  color: #d4af37;
  cursor: pointer;
  font-weight: 800;
}

.prompt-backdrop {
  position: fixed;
  inset: 0;
  z-index: 70;
  display: grid;
  place-items: center;
  padding: 20px;
  background: rgba(0, 0, 0, 0.55);
}

.prompt-card {
  width: min(420px, 100%);
  display: grid;
  gap: 18px;
  padding: 18px;
  background: #ffffff;
  color: #0a0a0a;
  border: 1px solid #d4af37;
  border-top: 5px solid #d4af37;
  border-radius: 8px;
  box-shadow: 0 24px 60px rgba(0, 0, 0, 0.35);
}

.prompt-card.danger {
  border-top-color: #b91c1c;
}

.prompt-card strong {
  display: block;
  font-size: 1.05rem;
}

.prompt-card p {
  margin-top: 6px;
  color: #3d3523;
  line-height: 1.45;
}

.prompt-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.prompt-actions button {
  min-height: 38px;
  padding: 0 14px;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 800;
}

.ghost-btn {
  border: 1px solid #d4af37;
  background: #ffffff;
  color: #0a0a0a;
}

.confirm-btn {
  border: 1px solid #0a0a0a;
  background: #0a0a0a;
  color: #d4af37;
}
</style>
