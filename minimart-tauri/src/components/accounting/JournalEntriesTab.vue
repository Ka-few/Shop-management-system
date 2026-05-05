<template>
  <section class="accounting-panel">
    <header class="toolbar">
      <div>
        <h2>Journal Entries</h2>
        <span>Posted ledger and manual controls</span>
      </div>
      <button type="button" @click="showManualForm = !showManualForm">{{ showManualForm ? 'Close' : 'Manual Entry' }}</button>
    </header>

    <div class="filters">
      <input v-model="filters.dateFrom" type="date" />
      <input v-model="filters.dateTo" type="date" />
      <select v-model="filters.accountId">
        <option value="">All accounts</option>
        <option v-for="account in accounts" :key="account.id" :value="String(account.id)">
          {{ account.code }} - {{ account.name }}
        </option>
      </select>
      <input v-model="filters.reference" placeholder="Reference" />
      <input v-model="filters.search" placeholder="Description search" />
      <button type="button" @click="loadEntries">Filter</button>
    </div>

    <form v-if="showManualForm" class="manual-form" @submit.prevent="createManualEntry">
      <div class="manual-head">
        <input v-model="manual.date" type="date" required />
        <input v-model="manual.reference" placeholder="Reference optional" />
        <input v-model="manual.description" placeholder="Description" required />
      </div>

      <div class="line-editor">
        <div v-for="(line, index) in manual.lines" :key="index" class="entry-line">
          <select v-model.number="line.account_id" required>
            <option :value="0">Select account</option>
            <option v-for="account in activeAccounts" :key="account.id" :value="account.id">
              {{ account.code }} - {{ account.name }}
            </option>
          </select>
          <input v-model.number="line.debit" type="number" min="0" step="0.01" placeholder="Debit" />
          <input v-model.number="line.credit" type="number" min="0" step="0.01" placeholder="Credit" />
          <input v-model="line.memo" placeholder="Memo" />
          <button type="button" class="secondary-btn" @click="removeLine(index)">Remove</button>
        </div>
      </div>

      <div class="balance-row" :class="{ error: !isBalanced }">
        <span>Debit: KES {{ money(totalDebit) }}</span>
        <span>Credit: KES {{ money(totalCredit) }}</span>
        <strong>{{ isBalanced ? 'Balanced' : 'Out of balance' }}</strong>
      </div>
      <div class="form-actions">
        <button type="button" class="secondary-btn" @click="addLine">Add Line</button>
        <button :disabled="!canSubmitManual">Post Entry</button>
      </div>
    </form>

    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Date</th>
            <th>Reference</th>
            <th>Description</th>
            <th>Transaction</th>
            <th>VAT</th>
            <th>Profit</th>
            <th>Status</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          <template v-for="entry in paginatedEntries" :key="entry.id">
            <tr>
              <td>{{ entry.date }}</td>
              <td>{{ entry.reference || '-' }}</td>
              <td>
                {{ entry.description }}
                <span v-if="entry.is_system_generated" class="pill">System</span>
              </td>
              <td>KES {{ money(entrySummary(entry).transactionAmount) }}</td>
              <td>KES {{ money(entrySummary(entry).vatAmount) }}</td>
              <td :class="{ loss: entrySummary(entry).profit < 0 }">KES {{ money(entrySummary(entry).profit) }}</td>
              <td><span class="status" :class="entry.status">{{ entry.status }}</span></td>
              <td class="actions">
                <button type="button" @click="toggle(entry.id)">{{ expanded[entry.id] ? 'Hide' : 'View' }}</button>
                <button type="button" class="secondary-btn" :disabled="entry.status === 'reversed'" @click="reverseEntry(entry)">Reverse</button>
              </td>
            </tr>
            <tr v-if="expanded[entry.id]" class="detail-row">
              <td colspan="8">
                <div class="summary-grid">
                  <div><span>Transaction Amount</span><strong>KES {{ money(entrySummary(entry).transactionAmount) }}</strong></div>
                  <div><span>VAT (16%)</span><strong>KES {{ money(entrySummary(entry).vatAmount) }}</strong></div>
                  <div><span>Net Sales</span><strong>KES {{ money(entrySummary(entry).netSales) }}</strong></div>
                  <div><span>COGS</span><strong>KES {{ money(entrySummary(entry).cogs) }}</strong></div>
                  <div :class="{ loss: entrySummary(entry).profit < 0 }"><span>Profit / Loss</span><strong>KES {{ money(entrySummary(entry).profit) }}</strong></div>
                  <div :class="{ loss: entrySummary(entry).profit < 0 }"><span>Margin</span><strong>{{ money(entrySummary(entry).marginPercent) }}%</strong></div>
                </div>
                <p v-if="entrySummary(entry).profit < 0" class="loss-note">Loss-making sale</p>
                <table>
                  <thead><tr><th>Account</th><th>Debit</th><th>Credit</th><th>Memo</th></tr></thead>
                  <tbody>
                    <tr v-for="line in entry.lines" :key="line.id" :class="lineClass(line)">
                      <td>
                        <span class="type-pill">{{ line.account_type }}</span>
                        {{ line.account_code }} - {{ line.account_name }}
                      </td>
                      <td>KES {{ money(line.debit) }}</td>
                      <td>KES {{ money(line.credit) }}</td>
                      <td>{{ line.memo || '-' }}</td>
                    </tr>
                  </tbody>
                </table>
              </td>
            </tr>
          </template>
        </tbody>
      </table>
    </div>
    <PaginationControls v-model:page="page" v-model:page-size="pageSize" :total-items="entries.length" />
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useNotifications } from '../../composables/useNotifications'
import PaginationControls from '../PaginationControls.vue'
import { summarizeJournalEntry } from '../../composables/useJournalSummary'

interface Account { id: number; code: string; name: string; is_active: boolean }
interface JournalLine { id: number; account_code: string; account_name: string; account_type: string; debit: number; credit: number; memo: string | null }
interface JournalEntry {
  id: number
  date: string
  reference: string | null
  description: string
  status: string
  is_system_generated: boolean
  total_debit: number
  total_credit: number
  lines: JournalLine[]
}

const today = new Date().toISOString().slice(0, 10)
const accounts = ref<Account[]>([])
const entries = ref<JournalEntry[]>([])
const page = ref(1)
const pageSize = ref(25)
const expanded = reactive<Record<number, boolean>>({})
const showManualForm = ref(false)
const { showToast, showPrompt } = useNotifications()

const filters = reactive({ dateFrom: '', dateTo: '', accountId: '', reference: '', search: '' })
const manual = reactive({
  date: today,
  reference: '',
  description: '',
  lines: [
    { account_id: 0, debit: 0, credit: 0, memo: '' },
    { account_id: 0, debit: 0, credit: 0, memo: '' },
  ],
})

const activeAccounts = computed(() => accounts.value.filter((account) => account.is_active))
const totalDebit = computed(() => manual.lines.reduce((sum, line) => sum + Number(line.debit || 0), 0))
const totalCredit = computed(() => manual.lines.reduce((sum, line) => sum + Number(line.credit || 0), 0))
const isBalanced = computed(() => Math.abs(totalDebit.value - totalCredit.value) < 0.005)
const canSubmitManual = computed(() =>
  manual.description.trim() &&
  manual.lines.length >= 2 &&
  manual.lines.every((line) => line.account_id && (Number(line.debit || 0) > 0 || Number(line.credit || 0) > 0)) &&
  isBalanced.value
)
const paginatedEntries = computed(() => entries.value.slice((page.value - 1) * pageSize.value, page.value * pageSize.value))

const money = (value: number | null | undefined) => Number(value || 0).toFixed(2)
const entrySummary = (entry: JournalEntry) => summarizeJournalEntry(entry.lines)
const lineClass = (line: JournalLine) => ({
  revenue: line.account_type === 'Revenue',
  expense: line.account_type === 'Expense',
  asset: line.account_type === 'Asset',
  vat: (line.account_code || '').trim() === '2100' || line.account_name.toLowerCase().includes('vat'),
})

const loadAccounts = async () => {
  accounts.value = await invoke<Account[]>('get_accounts')
}

const loadEntries = async () => {
  entries.value = await invoke<JournalEntry[]>('get_journal_entries', {
    dateFrom: filters.dateFrom || null,
    dateTo: filters.dateTo || null,
    accountId: filters.accountId ? Number(filters.accountId) : null,
    reference: filters.reference || null,
    search: filters.search || null,
  })
}
watch(filters, () => { page.value = 1 })

const addLine = () => manual.lines.push({ account_id: 0, debit: 0, credit: 0, memo: '' })
const removeLine = (index: number) => {
  if (manual.lines.length > 2) manual.lines.splice(index, 1)
}
const toggle = (id: number) => { expanded[id] = !expanded[id] }

const resetManual = () => {
  Object.assign(manual, { date: today, reference: '', description: '' })
  manual.lines.splice(0, manual.lines.length, { account_id: 0, debit: 0, credit: 0, memo: '' }, { account_id: 0, debit: 0, credit: 0, memo: '' })
}

const createManualEntry = async () => {
  if (!canSubmitManual.value) {
    showToast('Journal is not balanced', 'Total debit must equal total credit.', 'error')
    return
  }
  try {
    await invoke('create_manual_journal_entry', {
      entry: {
        date: manual.date,
        reference: manual.reference || null,
        description: manual.description,
        lines: manual.lines.map((line) => ({
          account_id: line.account_id,
          debit: Number(line.debit || 0),
          credit: Number(line.credit || 0),
          memo: line.memo || null,
        })),
      },
    })
    showToast('Journal posted', manual.description, 'success')
    resetManual()
    showManualForm.value = false
    await loadEntries()
  } catch (error) {
    showToast('Journal posting failed', String(error), 'error')
  }
}

const reverseEntry = async (entry: JournalEntry) => {
  const confirmed = await showPrompt({
    title: 'Reverse journal entry?',
    message: `Create an opposite entry for journal #${entry.id}?`,
    confirmText: 'Reverse',
    cancelText: 'Cancel',
    type: 'danger',
  })
  if (!confirmed) return

  try {
    await invoke('reverse_journal_entry', { id: entry.id, description: `Reversal of ${entry.description}` })
    showToast('Journal reversed', `Entry #${entry.id} has been reversed.`, 'success')
    await loadEntries()
  } catch (error) {
    showToast('Reverse failed', String(error), 'error')
  }
}

onMounted(async () => {
  await Promise.all([loadAccounts(), loadEntries()])
})
defineExpose({ loadEntries, loadAccounts })
</script>

<style scoped>
.accounting-panel { display: grid; gap: 14px; }
.toolbar, .filters, .manual-head, .entry-line, .form-actions, .balance-row { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.toolbar { justify-content: space-between; }
.toolbar h2 { margin: 0; }
.toolbar span { color: var(--color-muted); }
input, select { min-height: 40px; border: 1px solid #d7c58b; border-radius: 6px; padding: 0 10px; background: var(--color-white); color: var(--color-black); }
button { min-height: 38px; border-radius: 6px; border: 1px solid var(--color-black); background: var(--color-black); color: var(--color-gold); padding: 0 12px; cursor: pointer; font-weight: 800; }
button:hover:not(:disabled) { background: var(--color-gold); color: var(--color-black); border-color: var(--color-gold); }
button:disabled { opacity: 0.5; cursor: not-allowed; }
.secondary-btn { background: var(--color-white); color: var(--color-black); border-color: var(--color-gold); }
.manual-form { display: grid; gap: 12px; padding: 12px; border: 1px solid var(--color-border); border-radius: 8px; background: #fffaf0; }
.line-editor { display: grid; gap: 8px; }
.entry-line { display: grid; grid-template-columns: minmax(220px, 1fr) 120px 120px minmax(160px, 1fr) auto; }
.balance-row { justify-content: flex-end; color: var(--color-gold-dark); font-weight: 800; }
.balance-row.error { color: var(--color-danger); }
.table-wrap { overflow: auto; border: 1px solid var(--color-border); border-radius: 8px; }
table { width: 100%; border-collapse: collapse; background: var(--color-white); }
th, td { padding: 10px; border-bottom: 1px solid var(--color-border); text-align: left; }
th { color: var(--color-muted); }
.actions { display: flex; gap: 6px; flex-wrap: wrap; }
.pill { margin-left: 6px; padding: 2px 6px; border-radius: 999px; background: var(--color-gold); color: var(--color-black); font-size: 0.75rem; font-weight: 800; }
.status { text-transform: capitalize; color: var(--color-gold-dark); font-weight: 800; }
.status.reversed { color: var(--color-danger); }
.detail-row > td { background: #fffaf0; }
.summary-grid { display: grid; gap: 8px; grid-template-columns: repeat(3, minmax(170px, 1fr)); margin-bottom: 10px; }
.summary-grid > div { display: grid; gap: 2px; padding: 8px; border: 1px solid var(--color-border); border-radius: 6px; background: var(--color-white); }
.summary-grid span { color: var(--color-muted); font-size: 0.85rem; }
.loss { color: var(--color-danger); font-weight: 900; }
.loss-note { margin: 0 0 8px; color: var(--color-danger); font-weight: 800; }
.type-pill { display: inline-block; margin-right: 6px; padding: 1px 6px; border-radius: 999px; border: 1px solid var(--color-border); font-size: 0.72rem; color: var(--color-muted); }
tr.revenue td { background: rgba(44, 121, 57, 0.08); }
tr.expense td { background: rgba(185, 28, 28, 0.08); }
tr.asset td { background: rgba(0, 0, 0, 0.03); }
tr.vat td { background: rgba(107, 114, 128, 0.08); }
@media (max-width: 980px) {
  .entry-line { grid-template-columns: 1fr; }
  .summary-grid { grid-template-columns: 1fr; }
}
</style>
