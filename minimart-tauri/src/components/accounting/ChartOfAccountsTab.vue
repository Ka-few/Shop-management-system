<template>
  <section class="accounting-panel">
    <header class="toolbar">
      <div>
        <h2>Chart of Accounts</h2>
        <span>{{ filteredAccounts.length }} accounts</span>
      </div>
      <button type="button" @click="startCreate">Add Account</button>
    </header>

    <div class="filters">
      <input v-model="search" placeholder="Search code or name" />
      <select v-model="typeFilter">
        <option value="">All types</option>
        <option v-for="type in accountTypes" :key="type" :value="type">{{ type }}</option>
      </select>
      <select v-model="statusFilter">
        <option value="">All statuses</option>
        <option value="active">Active</option>
        <option value="inactive">Inactive</option>
      </select>
    </div>

    <form v-if="editing" class="account-form" @submit.prevent="saveAccount">
      <input v-model="form.code" placeholder="Code" required />
      <input v-model="form.name" placeholder="Account name" required />
      <select v-model="form.account_type" required>
        <option v-for="type in accountTypes" :key="type" :value="type">{{ type }}</option>
      </select>
      <select v-model="parentValue">
        <option value="">No parent</option>
        <option
          v-for="account in parentOptions"
          :key="account.id"
          :value="String(account.id)"
        >
          {{ account.code }} - {{ account.name }}
        </option>
      </select>
      <label class="check-row">
        <input v-model="form.is_active" type="checkbox" />
        Active
      </label>
      <button>{{ editing.id ? 'Save' : 'Create' }}</button>
      <button type="button" class="secondary-btn" @click="cancelEdit">Cancel</button>
    </form>

    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Code</th>
            <th>Name</th>
            <th>Type</th>
            <th>Parent</th>
            <th>Status</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="account in paginatedAccounts" :key="account.id" :class="{ inactive: !account.is_active }">
            <td>{{ account.code }}</td>
            <td :style="{ paddingLeft: `${(account.depth || 0) * 22 + 10}px` }">{{ account.name }}</td>
            <td>{{ account.account_type }}</td>
            <td>{{ parentName(account.parent_id) }}</td>
            <td><span class="status" :class="{ off: !account.is_active }">{{ account.is_active ? 'Active' : 'Inactive' }}</span></td>
            <td class="actions">
              <button type="button" @click="startEdit(account)">Edit</button>
              <button type="button" class="danger-btn" @click="deleteSelected(account)">Delete</button>
            </td>
          </tr>
          <tr v-if="filteredAccounts.length === 0">
            <td colspan="6">No accounts match the current filters.</td>
          </tr>
        </tbody>
      </table>
    </div>
    <PaginationControls v-model:page="page" v-model:page-size="pageSize" :total-items="filteredAccounts.length" />
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useNotifications } from '../../composables/useNotifications'
import PaginationControls from '../PaginationControls.vue'

interface Account {
  id: number
  code: string
  name: string
  account_type: string
  parent_id: number | null
  is_active: boolean
  depth?: number
}

const accountTypes = ['Asset', 'Liability', 'Equity', 'Revenue', 'Expense']
const accounts = ref<Account[]>([])
const search = ref('')
const typeFilter = ref('')
const statusFilter = ref('')
const page = ref(1)
const pageSize = ref(25)
const editing = ref<{ id: number | null } | null>(null)
const parentValue = ref('')
const { showToast, showPrompt } = useNotifications()

const form = reactive({
  code: '',
  name: '',
  account_type: 'Asset',
  parent_id: null as number | null,
  is_active: true,
})

const accountMap = computed(() => new Map(accounts.value.map((account) => [account.id, account])))
const parentName = (id: number | null) => id ? accountMap.value.get(id)?.name || '-' : '-'

const treeAccounts = computed(() => {
  const children = new Map<number | null, Account[]>()
  for (const account of accounts.value) {
    const key = account.parent_id || null
    children.set(key, [...(children.get(key) || []), account])
  }
  for (const group of children.values()) group.sort((a, b) => a.code.localeCompare(b.code))

  const output: Account[] = []
  const visit = (parentId: number | null, depth: number) => {
    for (const account of children.get(parentId) || []) {
      output.push({ ...account, depth })
      visit(account.id, depth + 1)
    }
  }
  visit(null, 0)
  return output
})

const filteredAccounts = computed(() => {
  const query = search.value.trim().toLowerCase()
  return treeAccounts.value.filter((account) => {
    const matchesSearch = !query || account.code.includes(query) || account.name.toLowerCase().includes(query)
    const matchesType = !typeFilter.value || account.account_type === typeFilter.value
    const matchesStatus = !statusFilter.value ||
      (statusFilter.value === 'active' ? account.is_active : !account.is_active)
    return matchesSearch && matchesType && matchesStatus
  })
})
const paginatedAccounts = computed(() => filteredAccounts.value.slice((page.value - 1) * pageSize.value, page.value * pageSize.value))

const parentOptions = computed(() => accounts.value.filter((account) =>
  account.is_active &&
  account.account_type === form.account_type &&
  account.id !== editing.value?.id
))
watch([search, typeFilter, statusFilter], () => { page.value = 1 })

const loadAccounts = async () => {
  accounts.value = await invoke<Account[]>('get_accounts')
}

const resetForm = () => {
  Object.assign(form, { code: '', name: '', account_type: 'Asset', parent_id: null, is_active: true })
  parentValue.value = ''
}

const startCreate = () => {
  resetForm()
  editing.value = { id: null }
}

const startEdit = (account: Account) => {
  Object.assign(form, {
    code: account.code,
    name: account.name,
    account_type: account.account_type,
    parent_id: account.parent_id,
    is_active: account.is_active,
  })
  parentValue.value = account.parent_id ? String(account.parent_id) : ''
  editing.value = { id: account.id }
}

const cancelEdit = () => {
  editing.value = null
  resetForm()
}

const saveAccount = async () => {
  form.parent_id = parentValue.value ? Number(parentValue.value) : null
  try {
    if (editing.value?.id) {
      await invoke('update_account', { id: editing.value.id, account: form })
      showToast('Account updated', `${form.code} - ${form.name}`, 'success')
    } else {
      await invoke('create_account', { account: { ...form, is_active: undefined } })
      showToast('Account created', `${form.code} - ${form.name}`, 'success')
    }
    cancelEdit()
    await loadAccounts()
  } catch (error) {
    showToast('Account save failed', String(error), 'error')
  }
}

const deleteSelected = async (account: Account) => {
  const confirmed = await showPrompt({
    title: 'Delete account?',
    message: `Deactivate ${account.code} - ${account.name}? Accounts used in journals cannot be deleted.`,
    confirmText: 'Delete',
    cancelText: 'Cancel',
    type: 'danger',
  })
  if (!confirmed) return

  try {
    await invoke('delete_account', { id: account.id })
    showToast('Account deleted', `${account.code} - ${account.name}`, 'success')
    await loadAccounts()
  } catch (error) {
    showToast('Account delete failed', String(error), 'error')
  }
}

onMounted(loadAccounts)
defineExpose({ loadAccounts })
</script>

<style scoped>
.accounting-panel { display: grid; gap: 14px; }
.toolbar, .filters, .account-form { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.toolbar { justify-content: space-between; }
.toolbar h2 { margin: 0; }
.toolbar span { color: var(--color-muted); }
input, select { min-height: 40px; border: 1px solid #d7c58b; border-radius: 6px; padding: 0 10px; background: var(--color-white); color: var(--color-black); }
button { min-height: 38px; border-radius: 6px; border: 1px solid var(--color-black); background: var(--color-black); color: var(--color-gold); padding: 0 12px; cursor: pointer; font-weight: 800; }
button:hover { background: var(--color-gold); color: var(--color-black); border-color: var(--color-gold); }
.secondary-btn { background: var(--color-white); color: var(--color-black); border-color: var(--color-gold); }
.danger-btn { background: var(--color-danger); color: var(--color-white); border-color: var(--color-danger); }
.account-form { padding: 12px; border: 1px solid var(--color-border); border-radius: 8px; background: #fffaf0; }
.check-row { display: flex; align-items: center; gap: 6px; color: var(--color-muted); font-weight: 800; }
.check-row input { min-height: auto; }
.table-wrap { overflow: auto; border: 1px solid var(--color-border); border-radius: 8px; }
table { width: 100%; border-collapse: collapse; background: var(--color-white); }
th, td { padding: 10px; border-bottom: 1px solid var(--color-border); text-align: left; }
th { color: var(--color-muted); }
.inactive { opacity: 0.58; }
.status { color: var(--color-gold-dark); font-weight: 800; }
.status.off { color: var(--color-danger); }
.actions { display: flex; gap: 6px; flex-wrap: wrap; }
</style>
