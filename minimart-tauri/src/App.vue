<script setup lang="ts">
import { ref } from 'vue'
import Login from './components/Login.vue'
import POS from './components/POS.vue'

interface User {
  id: number
  username: string
  email: string
  role: string
}

const currentUser = ref<User | null>(null)
const isLoggedIn = ref(false)

const handleLoginSuccess = (user: User) => {
  currentUser.value = user
  isLoggedIn.value = true
}

const handleLogout = () => {
  currentUser.value = null
  isLoggedIn.value = false
}
</script>

<template>
  <div id="app">
    <Login v-if="!isLoggedIn" @login-success="handleLoginSuccess" />
    <POS v-else :current-user="currentUser" @logout="handleLogout" />
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
  background: #f5f7fa;
}

#app {
  height: 100vh;
  width: 100vw;
}
</style>