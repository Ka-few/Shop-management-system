<script setup lang="ts">
import { ref } from 'vue'
import Login from './components/Login.vue'
import POS from './components/POS.vue'
import NotificationHost from './components/NotificationHost.vue'
import { useNotifications } from './composables/useNotifications'

interface User {
  id: number
  username: string
  email: string
  role: string
  created_at?: string
  updated_at?: string | null
}

const currentUser = ref<User | null>(null)
const isLoggedIn = ref(false)
const { showToast } = useNotifications()

const handleLoginSuccess = (user: User) => {
  currentUser.value = user
  isLoggedIn.value = true
  showToast('Login successful', `Welcome back, ${user.username}.`, 'success')
}

const handleLogout = () => {
  currentUser.value = null
  isLoggedIn.value = false
  showToast('Logged out', 'Your session has ended.', 'info')
}
</script>

<template>
  <div id="app">
    <Login v-if="!isLoggedIn" @login-success="handleLoginSuccess" />
    <POS v-else :current-user="currentUser" @logout="handleLogout" />
    <NotificationHost />
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
    'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue',
    sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  background: #0a0a0a;
  color: #0a0a0a;
}

#app {
  height: 100vh;
  width: 100vw;
}

:root {
  --color-black: #0a0a0a;
  --color-black-soft: #151515;
  --color-white: #ffffff;
  --color-cream: #f8f5ed;
  --color-gold: #d4af37;
  --color-gold-dark: #9b7628;
  --color-gold-soft: #f2df9b;
  --color-border: #e5d8aa;
  --color-muted: #6d6250;
  --color-danger: #b91c1c;
}
</style>
