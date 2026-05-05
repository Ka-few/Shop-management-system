<template>
  <section class="accounting-panel">
    <header class="toolbar">
      <div>
        <h2>Profit & Loss</h2>
        <span>Generated from posted journal entries</span>
      </div>
      <div class="actions">
        <button type="button" @click="exportCsv">CSV</button>
        <button type="button" @click="printReport">PDF</button>
      </div>
    </header>

    <div class="filters">
      <input v-model="dateFrom" type="date" />
      <input v-model="dateTo" type="date" />
      <button type="button" @click="loadReport">Run Report</button>
    </div>

    <article id="pl-report" class="report-card">
      <header>
        <strong>Minimart POS Profit & Loss</strong>
        <span>{{ dateFrom || 'Beginning' }} to {{ dateTo || 'Today' }}</span>
      </header>

      <section>
        <h3>Revenue</h3>
        <div v-for="row in paginatedRevenue" :key="row.account_id" class="line">
          <span>{{ row.code }} - {{ row.name }}</span><strong>KES {{ money(row.amount) }}</strong>
        </div>
        <PaginationControls v-model:page="pages.revenue" v-model:page-size="pageSizes.revenue" :total-items="report.revenue.length" />
        <div class="total"><span>Total Revenue</span><strong>KES {{ money(report.total_revenue) }}</strong></div>
      </section>

      <section>
        <h3>Less: Cost of Goods Sold</h3>
        <div v-for="row in paginatedCogs" :key="row.account_id" class="line">
          <span>{{ row.code }} - {{ row.name }}</span><strong>KES {{ money(row.amount) }}</strong>
        </div>
        <PaginationControls v-model:page="pages.cogs" v-model:page-size="pageSizes.cogs" :total-items="report.cogs.length" />
        <div class="total"><span>Total COGS</span><strong>KES {{ money(report.total_cogs) }}</strong></div>
      </section>

      <div class="result"><span>Gross Profit</span><strong>KES {{ money(report.gross_profit) }}</strong></div>

      <section>
        <h3>Less: Expenses</h3>
        <div v-for="row in paginatedExpenses" :key="row.account_id" class="line">
          <span>{{ row.code }} - {{ row.name }}</span><strong>KES {{ money(row.amount) }}</strong>
        </div>
        <PaginationControls v-model:page="pages.expenses" v-model:page-size="pageSizes.expenses" :total-items="report.expenses.length" />
        <div class="total"><span>Total Expenses</span><strong>KES {{ money(report.total_expenses) }}</strong></div>
      </section>

      <div class="result net"><span>Net Profit</span><strong>KES {{ money(report.net_profit) }}</strong></div>
    </article>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useNotifications } from '../../composables/useNotifications'
import PaginationControls from '../PaginationControls.vue'

interface PnlRow { account_id: number; code: string; name: string; amount: number }
interface PnlReport {
  revenue: PnlRow[]
  cogs: PnlRow[]
  expenses: PnlRow[]
  total_revenue: number
  total_cogs: number
  gross_profit: number
  total_expenses: number
  net_profit: number
}

const dateFrom = ref('')
const dateTo = ref(new Date().toISOString().slice(0, 10))
const { showToast } = useNotifications()
const pages = reactive({ revenue: 1, cogs: 1, expenses: 1 })
const pageSizes = reactive({ revenue: 10, cogs: 10, expenses: 10 })
const report = reactive<PnlReport>({
  revenue: [],
  cogs: [],
  expenses: [],
  total_revenue: 0,
  total_cogs: 0,
  gross_profit: 0,
  total_expenses: 0,
  net_profit: 0,
})

const money = (value: number | null | undefined) => Number(value || 0).toFixed(2)
const paginate = (items: PnlRow[], page: number, pageSize: number) => items.slice((page - 1) * pageSize, page * pageSize)
const paginatedRevenue = computed(() => paginate(report.revenue, pages.revenue, pageSizes.revenue))
const paginatedCogs = computed(() => paginate(report.cogs, pages.cogs, pageSizes.cogs))
const paginatedExpenses = computed(() => paginate(report.expenses, pages.expenses, pageSizes.expenses))

const loadReport = async () => {
  try {
    const data = await invoke<PnlReport>('get_profit_and_loss', {
      dateFrom: dateFrom.value || null,
      dateTo: dateTo.value || null,
    })
    Object.assign(report, data)
    Object.assign(pages, { revenue: 1, cogs: 1, expenses: 1 })
  } catch (error) {
    showToast('P&L failed', String(error), 'error')
  }
}

const exportCsv = () => {
  const rows = [
    ['Section', 'Code', 'Account', 'Amount'],
    ...report.revenue.map((row) => ['Revenue', row.code, row.name, money(row.amount)]),
    ...report.cogs.map((row) => ['COGS', row.code, row.name, money(row.amount)]),
    ...report.expenses.map((row) => ['Expenses', row.code, row.name, money(row.amount)]),
    ['Summary', '', 'Gross Profit', money(report.gross_profit)],
    ['Summary', '', 'Net Profit', money(report.net_profit)],
  ]
  const csv = rows.map((row) => row.map((cell) => `"${String(cell).replace(/"/g, '""')}"`).join(',')).join('\n')
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = 'profit-and-loss.csv'
  link.click()
  URL.revokeObjectURL(url)
}

const printReport = () => window.print()

onMounted(loadReport)
defineExpose({ loadReport })
</script>

<style scoped>
.accounting-panel { display: grid; gap: 14px; }
.toolbar, .filters, .actions { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.toolbar { justify-content: space-between; }
.toolbar h2 { margin: 0; }
.toolbar span { color: var(--color-muted); }
input { min-height: 40px; border: 1px solid #d7c58b; border-radius: 6px; padding: 0 10px; background: var(--color-white); color: var(--color-black); }
button { min-height: 38px; border-radius: 6px; border: 1px solid var(--color-black); background: var(--color-black); color: var(--color-gold); padding: 0 12px; cursor: pointer; font-weight: 800; }
button:hover { background: var(--color-gold); color: var(--color-black); border-color: var(--color-gold); }
.report-card { display: grid; gap: 16px; max-width: 820px; padding: 18px; border: 1px solid var(--color-border); border-radius: 8px; background: var(--color-white); }
.report-card > header { display: grid; gap: 4px; padding-bottom: 12px; border-bottom: 2px solid var(--color-gold); text-align: center; }
.report-card > header strong { font-size: 1.2rem; }
.report-card > header span { color: var(--color-muted); }
section { display: grid; gap: 8px; }
h3 { margin: 0; font-size: 1rem; color: var(--color-muted); }
.line, .total, .result { display: flex; justify-content: space-between; gap: 12px; }
.line span { color: var(--color-muted); }
.total { padding-top: 6px; border-top: 1px solid var(--color-border); font-weight: 800; }
.result { padding: 12px; border-radius: 8px; background: #fffaf0; border: 1px solid var(--color-gold); font-weight: 900; }
.net { background: var(--color-black); color: var(--color-gold); }
@media print {
  .toolbar, .filters, :global(.sidebar), :global(.toast-stack) { display: none !important; }
  .report-card { border: 0; max-width: none; }
}
</style>
