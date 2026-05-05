<template>
  <div class="login-container">
    <div class="login-card">
      <h2>Minimart POS Login</h2>
      <form @submit.prevent="handleLogin" class="login-form">
        <div class="form-group">
          <label for="username">Username</label>
          <input
            id="username"
            v-model="username"
            type="text"
            placeholder="Enter username"
            required
          />
        </div>
        <div class="form-group">
          <label for="password">Password</label>
          <input
            id="password"
            v-model="password"
            type="password"
            placeholder="Enter password"
            required
          />
        </div>
        <button type="submit" :disabled="loading" class="login-btn">
          {{ loading ? 'Logging in...' : 'Login' }}
        </button>
      </form>
      <p v-if="error" class="error-message">{{ error }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useNotifications } from '../composables/useNotifications'

const username = ref('')
const password = ref('')
const loading = ref(false)
const error = ref('')
const { showToast } = useNotifications()

const emit = defineEmits<{
  loginSuccess: [user: any]
}>()

const handleLogin = async () => {
  loading.value = true
  error.value = ''

  try {
    const result = await invoke('login', {
      request: {
        username: username.value,
        password: password.value
      }
    })

    emit('loginSuccess', (result as any).user)
  } catch (err) {
    error.value = err as string
    showToast('Login failed', String(err), 'error')
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.login-container {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #0a0a0a;
  padding: 20px;
}

.login-card {
  background: #ffffff;
  padding: 2rem;
  border: 1px solid #d4af37;
  border-radius: 8px;
  box-shadow: 0 18px 45px rgba(212, 175, 55, 0.16);
  width: 100%;
  max-width: 400px;
}

.login-card h2 {
  text-align: center;
  margin-bottom: 2rem;
  color: #0a0a0a;
  font-size: 1.8rem;
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.form-group {
  display: flex;
  flex-direction: column;
}

.form-group label {
  margin-bottom: 0.5rem;
  font-weight: 700;
  color: #3d3523;
}

.form-group input {
  padding: 0.75rem;
  border: 1px solid #d7c58b;
  border-radius: 6px;
  font-size: 1rem;
  transition: border-color 0.2s, box-shadow 0.2s;
}

.form-group input:focus {
  outline: none;
  border-color: #d4af37;
  box-shadow: 0 0 0 3px rgba(212, 175, 55, 0.18);
}

.login-btn {
  padding: 0.75rem;
  background: #0a0a0a;
  color: #d4af37;
  border: 1px solid #0a0a0a;
  border-radius: 6px;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: transform 0.2s;
}

.login-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 10px 24px rgba(212, 175, 55, 0.2);
}

.login-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.error-message {
  color: #b91c1c;
  text-align: center;
  margin-top: 1rem;
  font-weight: 500;
}
</style>
