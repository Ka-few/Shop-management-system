<template>
  <div class="app-shell">
    <aside class="sidebar">
      <div class="brand">
        <strong>{{ receiptStoreName }}</strong>
        <span>{{ currentUser?.username }}</span>
      </div>
      <nav class="nav-tabs">
        <button v-for="tab in visibleTabs" :key="tab.id" :class="{ active: activeView === tab.id }" @click="activeView = tab.id">
          <span>{{ tab.icon }}</span>{{ tab.label }}
        </button>
      </nav>
      <button class="logout-btn" @click="logout">Logout</button>
    </aside>

    <main class="workspace">
      <section v-if="activeView === 'dashboard'" class="view">
        <header class="view-header">
          <h1>Dashboard</h1>
          <button @click="refreshAll">Refresh</button>
        </header>
        <div class="metrics-grid">
          <div class="metric"><span>Today</span><strong>KES {{ money(dashboard?.todays_sales) }}</strong></div>
          <div class="metric"><span>This Week</span><strong>KES {{ money(dashboard?.weekly_sales) }}</strong></div>
          <div class="metric"><span>Products</span><strong>{{ dashboard?.total_products ?? 0 }}</strong></div>
          <div class="metric warning"><span>Low Stock</span><strong>{{ dashboard?.low_stock_items ?? 0 }}</strong></div>
        </div>
        <div class="split-grid">
          <section class="panel">
            <h2>Recent Transactions</h2>
            <table>
              <tbody>
                <tr v-for="sale in paginatedRecentSales" :key="sale.id">
                  <td>#{{ sale.id }}</td>
                  <td>{{ sale.customer_name }}</td>
                  <td>KES {{ money(sale.total_amount) }}</td>
                </tr>
                <tr v-if="recentSales.length === 0"><td colspan="3">No completed sales yet.</td></tr>
              </tbody>
            </table>
            <PaginationControls v-model:page="pages.recentSales" v-model:page-size="pageSizes.recentSales" :total-items="recentSales.length" />
          </section>
          <section class="panel">
            <h2>Top Products</h2>
            <table>
              <tbody>
                <tr v-for="product in paginatedTopProducts" :key="product.name">
                  <td>{{ product.name }}</td>
                  <td>{{ quantityLabel(product.total_quantity) }} sold</td>
                  <td>KES {{ money(product.total_revenue) }}</td>
                </tr>
                <tr v-if="topProducts.length === 0"><td colspan="3">No product sales yet.</td></tr>
              </tbody>
            </table>
            <PaginationControls v-model:page="pages.topProducts" v-model:page-size="pageSizes.topProducts" :total-items="topProducts.length" />
          </section>
        </div>
      </section>

      <section v-if="activeView === 'pos'" class="view pos-view">
        <div class="sale-surface">
          <header class="view-header compact">
            <h1>Point of Sale</h1>
            <span>{{ filteredProducts.length }} products</span>
          </header>

          <div class="scan-row">
            <input ref="barcodeInput" v-model="barcodeQuery" @keyup.enter="addBarcodeToCart" placeholder="Scan barcode or type SKU/name" />
            <button @click="addBarcodeToCart">Add Scan</button>
          </div>

          <div class="search-controls">
            <input v-model="searchQuery" placeholder="Search products" />
            <select v-model="selectedCategory">
              <option value="">All categories</option>
              <option v-for="category in categories" :key="category.id" :value="String(category.id)">
                {{ category.icon }} {{ category.name }}
              </option>
            </select>
          </div>

          <div class="products-grid">
            <button
              v-for="product in paginatedFilteredProducts"
              :key="product.id"
              class="product-tile"
              :disabled="product.quantity_in_stock <= 0"
              @click="addToCart(product, product.barcode || undefined)"
            >
              <strong>{{ product.name }}</strong>
              <span>{{ product.sku }}</span>
              <em>KES {{ money(product.unit_price) }}</em>
              <small :class="{ low: product.quantity_in_stock <= product.reorder_level }">
                Stock {{ quantityLabel(product.quantity_in_stock) }}
              </small>
            </button>
          </div>
          <PaginationControls v-model:page="pages.posProducts" v-model:page-size="pageSizes.posProducts" :total-items="filteredProducts.length" />
        </div>

        <aside class="cart-panel">
          <h2>Cart</h2>
          <div class="cart-list">
            <p v-if="cart.length === 0" class="empty-state">No items in cart.</p>
            <div v-for="(item, index) in cart" :key="item.product.id" class="cart-item">
              <div class="cart-item-summary">
                <strong>{{ item.product.name }}</strong>
                <span>KES {{ money(item.product.unit_price) }} x {{ quantityLabel(item.quantity) }} = KES {{ money(lineTotal(item)) }}</span>
              </div>
              <div class="cart-entry-controls">
                <label>
                  <span>Qty</span>
                  <input :value="quantityLabel(item.quantity)" type="number" min="0.01" step="0.01" @input="setQuantityFromInput(index, $event)" />
                </label>
                <label>
                  <span>KES</span>
                  <input :value="money(lineTotal(item))" type="text" inputmode="decimal" @input="setLineTotalFromInput(index, $event)" />
                </label>
              </div>
              <div class="qty-controls">
                <button @click="updateQuantity(index, item.quantity - 1)">-</button>
                <b>{{ quantityLabel(item.quantity) }}</b>
                <button @click="updateQuantity(index, item.quantity + 1)">+</button>
                <button class="danger" @click="removeFromCart(index)">x</button>
              </div>
            </div>
          </div>
          <div class="totals">
            <div><span>Subtotal</span><strong>KES {{ money(subtotal) }}</strong></div>
            <div><span>VAT 16% included</span><strong>KES {{ money(vat) }}</strong></div>
            <div class="grand"><span>Total</span><strong>KES {{ money(total) }}</strong></div>
          </div>
          <form class="checkout" @submit.prevent="processPayment">
            <select v-model="paymentMethod">
              <option value="cash">Cash</option>
              <option value="card">Card</option>
              <option value="mpesa">M-Pesa</option>
              <option value="partial">Partial</option>
            </select>
          <input v-model.number="amountReceived" type="number" min="0" step="0.01" placeholder="Amount received" />
          <span class="change">Change: KES {{ money(change) }}</span>
          <button type="button" class="secondary-btn" @click="amountReceived = Number(total.toFixed(2))">Exact Amount</button>
          <button :disabled="cart.length === 0 || processingPayment">{{ processingPayment ? 'Processing' : 'Complete Sale' }}</button>
          <p v-if="saleMessage" class="form-message" :class="{ error: saleError }">{{ saleMessage }}</p>
        </form>
        </aside>
      </section>

      <section v-if="activeView === 'products'" class="view">
        <header class="view-header"><h1>Products</h1><button @click="loadProducts">Reload</button></header>
        <form class="product-form" @submit.prevent="createProduct">
          <input v-model="newProduct.name" placeholder="Name" required />
          <input v-model="newProduct.sku" placeholder="SKU" required />
          <select v-model.number="newProduct.category_id" required>
            <option v-for="category in categories" :key="category.id" :value="category.id">{{ category.name }}</option>
          </select>
          <input v-model.number="newProduct.unit_price" type="number" min="0" step="0.01" placeholder="Price" required />
          <input v-model.number="newProduct.cost_price" type="number" min="0" step="0.01" placeholder="Cost" />
          <input v-model.number="newProduct.quantity_in_stock" type="number" min="0" step="0.01" placeholder="Initial stock" />
          <input v-model.number="newProduct.reorder_level" type="number" min="0" step="0.01" placeholder="Reorder" />
          <input v-model="newProduct.barcode" placeholder="Barcode optional" />
          <button>Create Product</button>
        </form>
        <form v-if="editingProductId" ref="productEditForm" class="product-form edit-form" @submit.prevent="saveProductEdit">
          <input v-model="editProduct.name" placeholder="Name" required />
          <input v-model="editProduct.sku" placeholder="SKU" required />
          <select v-model.number="editProduct.category_id" required>
            <option v-for="category in categories" :key="category.id" :value="category.id">{{ category.name }}</option>
          </select>
          <input v-model.number="editProduct.unit_price" type="number" min="0" step="0.01" placeholder="Price" required />
          <input v-model.number="editProduct.cost_price" type="number" min="0" step="0.01" placeholder="Cost" />
          <input v-model.number="editProduct.quantity_in_stock" type="number" min="0" step="0.01" placeholder="Stock" />
          <input v-model.number="editProduct.reorder_level" type="number" min="0" step="0.01" placeholder="Reorder" />
          <input v-model="editProduct.barcode" placeholder="Barcode optional" />
          <button>Save</button>
          <button type="button" class="secondary-btn" @click="cancelEdit">Cancel</button>
        </form>
        <p v-if="productFormMessage" class="form-message" :class="{ error: productFormError }">{{ productFormMessage }}</p>
        <section class="panel">
          <table>
            <thead><tr><th>Name</th><th>SKU</th><th>Barcode</th><th>Category</th><th>Price</th><th>Stock</th><th>Actions</th></tr></thead>
            <tbody>
              <tr v-for="product in paginatedProducts" :key="product.id">
                <td>{{ product.name }}</td>
                <td>{{ product.sku }}</td>
                <td>{{ product.barcode || '-' }}</td>
                <td>{{ categoryName(product.category_id) }}</td>
                <td>KES {{ money(product.unit_price) }}</td>
                <td>{{ quantityLabel(product.quantity_in_stock) }}</td>
                <td class="action-cell">
                  <button @click="startEdit(product)">Edit</button>
                  <button class="danger-btn" @click="deleteProduct(product)">Delete</button>
                </td>
              </tr>
            </tbody>
          </table>
          <PaginationControls v-model:page="pages.products" v-model:page-size="pageSizes.products" :total-items="products.length" />
        </section>
      </section>

      <section v-if="activeView === 'inventory'" class="view">
        <header class="view-header"><h1>Inventory</h1><button @click="loadInventory">Reload</button></header>
        <section class="panel">
          <table>
            <thead><tr><th>Product</th><th>Stock</th><th>Reorder</th><th>Status</th><th>Adjust</th></tr></thead>
            <tbody>
              <tr v-for="item in paginatedInventory" :key="item.product_id">
                <td>{{ item.product_name }}</td>
                <td>{{ quantityLabel(item.current_stock) }}</td>
                <td>{{ quantityLabel(item.reorder_level) }}</td>
                <td><span class="status" :class="item.status">{{ item.status.replace('_', ' ') }}</span></td>
                <td>
                  <input v-model.number="adjustments[item.product_id]" type="number" step="0.01" />
                  <button @click="adjustStock(item.product_id)">Apply</button>
                </td>
              </tr>
            </tbody>
          </table>
          <PaginationControls v-model:page="pages.inventory" v-model:page-size="pageSizes.inventory" :total-items="inventory.length" />
        </section>
      </section>

      <section v-if="activeView === 'reports'" class="view">
        <header class="view-header"><h1>Reports</h1><button @click="loadReports">Refresh</button></header>
        <div class="metrics-grid">
          <div class="metric"><span>Sales Today</span><strong>{{ dailySummary.sale_count ?? 0 }}</strong></div>
          <div class="metric"><span>Items Sold</span><strong>{{ quantityLabel(dailySummary.items_sold ?? 0) }}</strong></div>
          <div class="metric"><span>Revenue</span><strong>KES {{ money(dailySummary.total_sales) }}</strong></div>
        </div>
        <section class="panel">
          <h2>Sales by Category</h2>
          <table>
            <tbody>
              <tr v-for="row in paginatedSalesByCategory" :key="row.name">
                <td>{{ row.icon }} {{ row.name }}</td>
                <td>{{ row.product_count }} products</td>
                <td>KES {{ money(row.total_revenue) }}</td>
              </tr>
            </tbody>
          </table>
          <PaginationControls v-model:page="pages.salesByCategory" v-model:page-size="pageSizes.salesByCategory" :total-items="salesByCategory.length" />
        </section>
      </section>

      <section v-if="activeView === 'accounting'" class="view">
        <header class="view-header">
          <h1>Accounting</h1>
          <span>Double-entry ledger controls</span>
        </header>
        <nav class="accounting-tabs">
          <button :class="{ active: activeAccountingTab === 'accounts' }" @click="activeAccountingTab = 'accounts'">Chart of Accounts</button>
          <button :class="{ active: activeAccountingTab === 'journal' }" @click="activeAccountingTab = 'journal'">Journal Entries</button>
          <button :class="{ active: activeAccountingTab === 'pl' }" @click="activeAccountingTab = 'pl'">Profit & Loss</button>
        </nav>
        <section class="panel">
          <ChartOfAccountsTab v-if="activeAccountingTab === 'accounts'" />
          <JournalEntriesTab v-if="activeAccountingTab === 'journal'" />
          <ProfitAndLossTab v-if="activeAccountingTab === 'pl'" />
        </section>
      </section>

      <section v-if="activeView === 'settings'" class="view">
        <header class="view-header">
          <h1>Settings</h1>
          <div class="settings-actions">
            <button v-if="activeSettingsTab === 'app'" @click="saveAllSettings" :disabled="savingAllSettings">{{ savingAllSettings ? 'Saving...' : 'Save All' }}</button>
            <button v-if="activeSettingsTab === 'app'" @click="validateDb">Validate DB</button>
          </div>
        </header>
        <nav class="accounting-tabs">
          <button :class="{ active: activeSettingsTab === 'app' }" @click="activeSettingsTab = 'app'">App Settings</button>
          <button :class="{ active: activeSettingsTab === 'users' }" @click="activeSettingsTab = 'users'">Users</button>
        </nav>
        <section v-if="activeSettingsTab === 'app'" class="panel">
          <table>
            <thead>
              <tr><th>Key</th><th>Value</th><th>Description</th><th>Action</th></tr>
            </thead>
            <tbody>
              <tr v-for="setting in settings" :key="setting.key">
                <td>{{ setting.key }}</td>
                <td>
                  <input
                    v-model="settingEdits[setting.key]"
                    :placeholder="setting.value || ''"
                  />
                </td>
                <td>{{ setting.description }}</td>
                <td>
                  <button
                    type="button"
                    @click="saveSetting(setting.key)"
                    :disabled="savingSettingKey === setting.key"
                  >
                    {{ savingSettingKey === setting.key ? 'Saving...' : 'Save' }}
                  </button>
                </td>
              </tr>
            </tbody>
          </table>
        </section>
        <pre v-if="activeSettingsTab === 'app' && dbValidation">{{ dbValidation }}</pre>
        <section v-if="activeSettingsTab === 'users'" class="panel">
          <form ref="userEditForm" class="user-form" @submit.prevent="saveUser">
            <input v-model="userForm.username" placeholder="Username" required />
            <input v-model="userForm.email" type="email" placeholder="Email" required />
            <select v-model="userForm.role">
              <option value="user">User</option>
              <option value="staff">Staff</option>
              <option value="teller">Teller</option>
              <option value="admin">Admin</option>
            </select>
            <input v-model="userForm.password" type="password" :placeholder="editingUserId ? 'New password optional' : 'Password'" :required="!editingUserId" />
            <button :disabled="savingUser">{{ savingUser ? 'Saving...' : editingUserId ? 'Update User' : 'Create User' }}</button>
            <button v-if="editingUserId" type="button" class="secondary-btn" @click="cancelUserEdit">Cancel</button>
          </form>
          <p v-if="userFormMessage" class="form-message" :class="{ error: userFormError }">{{ userFormMessage }}</p>
          <table>
            <thead>
              <tr><th>Username</th><th>Email</th><th>Role</th><th>Created</th><th>Actions</th></tr>
            </thead>
            <tbody>
              <tr v-for="user in users" :key="user.id">
                <td>{{ user.username }}</td>
                <td>{{ user.email }}</td>
                <td><span class="role-pill">{{ user.role }}</span></td>
                <td>{{ shortDate(user.created_at) }}</td>
                <td class="action-cell">
                  <button type="button" @click="startUserEdit(user)">Edit</button>
                  <button type="button" class="danger-btn" :disabled="user.id === currentUser?.id" @click="deleteUserAccount(user)">Delete</button>
                </td>
              </tr>
              <tr v-if="users.length === 0"><td colspan="5">No users found.</td></tr>
            </tbody>
          </table>
        </section>
      </section>
    </main>

    <div v-if="printReceiptData" class="print-popup" role="dialog" aria-modal="true">
      <section class="print-dialog">
        <header class="print-actions">
          <div>
            <strong>Receipt #{{ printReceiptData.saleId }}</strong>
            <span>{{ printReceiptData.createdAt }}</span>
          </div>
          <div>
            <button type="button" class="icon-btn" @click="printReceipt" aria-label="Print receipt">
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <path d="M6 9V2h12v7" />
                <path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2" />
                <path d="M6 14h12v8H6z" />
              </svg>
            </button>
            <button type="button" class="icon-btn close-btn" @click="closePrintPopup" aria-label="Close receipt print popup">x</button>
          </div>
        </header>

        <article class="print-receipt">
          <header>
            <strong>{{ receiptStoreName }}</strong>
            <span>Receipt #{{ printReceiptData.saleId }}</span>
            <small>{{ printReceiptData.createdAt }}</small>
            <small v-if="receiptStoreAddress">{{ receiptStoreAddress }}</small>
            <small v-if="receiptStorePhone">{{ receiptStorePhone }}</small>
          </header>
          <div class="print-items">
            <div v-for="item in printReceiptData.items" :key="item.name" class="print-line">
              <span>{{ item.name }} x {{ quantityLabel(item.quantity) }}</span>
              <strong>KES {{ money(item.lineTotal) }}</strong>
            </div>
          </div>
          <div class="print-totals">
            <div><span>Subtotal</span><strong>KES {{ money(printReceiptData.subtotal) }}</strong></div>
            <div><span>VAT 16% included</span><strong>KES {{ money(printReceiptData.vat) }}</strong></div>
            <div class="grand"><span>Total</span><strong>KES {{ money(printReceiptData.total) }}</strong></div>
            <div><span>Paid</span><strong>KES {{ money(printReceiptData.paid) }}</strong></div>
            <div><span>Change</span><strong>KES {{ money(printReceiptData.change) }}</strong></div>
          </div>
          <footer v-if="receiptFooter" class="print-footer">{{ receiptFooter }}</footer>
        </article>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, reactive, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useNotifications } from '../composables/useNotifications'
import PaginationControls from './PaginationControls.vue'
import ChartOfAccountsTab from './accounting/ChartOfAccountsTab.vue'
import JournalEntriesTab from './accounting/JournalEntriesTab.vue'
import ProfitAndLossTab from './accounting/ProfitAndLossTab.vue'

interface User { id: number; username: string; email: string; role: string; created_at?: string; updated_at?: string | null }
interface Category { id: number; name: string; icon: string | null; description: string | null }
interface Product {
  id: number
  name: string
  category_id: number
  sku: string
  barcode: string | null
  barcode_format: string
  description: string | null
  unit_price: number
  cost_price: number | null
  quantity_in_stock: number
  reorder_level: number
}
interface CartItem { product: Product; quantity: number; barcodeScanned?: string }
interface InventoryStatus { product_id: number; product_name: string; current_stock: number; reorder_level: number; status: string }
interface Setting { key: string; value: string | null; description: string | null }
interface ReceiptItem { name: string; quantity: number; lineTotal: number }
interface Receipt {
  saleId: number
  createdAt: string
  items: ReceiptItem[]
  subtotal: number
  vat: number
  netSales: number
  cogs: number
  profit: number
  marginPercent: number
  total: number
  paid: number
  change: number
}

const props = defineProps<{ currentUser: User | null }>()
const emit = defineEmits<{ logout: [] }>()

const tabs = [
  { id: 'dashboard', label: 'Dashboard', icon: '▦' },
  { id: 'pos', label: 'POS', icon: '▣' },
  { id: 'products', label: 'Products', icon: '□' },
  { id: 'inventory', label: 'Inventory', icon: '≡' },
  { id: 'reports', label: 'Reports', icon: '△' },
  { id: 'accounting', label: 'Accounting', icon: '$' },
  { id: 'settings', label: 'Settings', icon: '⚙' },
]

const activeView = ref('pos')
const activeAccountingTab = ref<'accounts' | 'journal' | 'pl'>('accounts')
const activeSettingsTab = ref<'app' | 'users'>('app')
const products = ref<Product[]>([])
const categories = ref<Category[]>([])
const inventory = ref<InventoryStatus[]>([])
const cart = ref<CartItem[]>([])
const dashboard = ref<any>(null)
const recentSales = ref<any[]>([])
const topProducts = ref<any[]>([])
const salesByCategory = ref<any[]>([])
const settings = ref<Setting[]>([])
const users = ref<User[]>([])
const dailySummary = ref<any>({})
const dbValidation = ref('')
const settingEdits = reactive<Record<string, string>>({})
const savingSettingKey = ref('')
const savingAllSettings = ref(false)
const savingUser = ref(false)
const adjustments = reactive<Record<number, number>>({})
const { showToast, showPrompt } = useNotifications()
const pages = reactive({
  recentSales: 1,
  topProducts: 1,
  posProducts: 1,
  products: 1,
  inventory: 1,
  salesByCategory: 1,
})
const pageSizes = reactive({
  recentSales: 10,
  topProducts: 10,
  posProducts: 24,
  products: 10,
  inventory: 10,
  salesByCategory: 10,
})

const barcodeInput = ref<HTMLInputElement | null>(null)
const productEditForm = ref<HTMLFormElement | null>(null)
const userEditForm = ref<HTMLFormElement | null>(null)
const barcodeQuery = ref('')
const searchQuery = ref('')
const selectedCategory = ref('')
const paymentMethod = ref('cash')
const amountReceived = ref(0)
const processingPayment = ref(false)
const saleMessage = ref('')
const saleError = ref(false)
const lastReceipt = ref<Receipt | null>(null)
const printReceiptData = ref<Receipt | null>(null)
const productFormMessage = ref('')
const productFormError = ref(false)
const editingProductId = ref<number | null>(null)
const newProduct = reactive({
  name: '',
  sku: '',
  category_id: 1,
  barcode: '',
  unit_price: 0,
  cost_price: null as number | null,
  quantity_in_stock: 0,
  reorder_level: 10,
})
const editProduct = reactive({
  name: '',
  sku: '',
  category_id: 1,
  barcode: '',
  unit_price: 0,
  cost_price: null as number | null,
  quantity_in_stock: 0,
  reorder_level: 10,
})
const editingUserId = ref<number | null>(null)
const userForm = reactive({
  username: '',
  email: '',
  role: 'user',
  password: '',
})
const userFormMessage = ref('')
const userFormError = ref(false)

const currentUser = computed(() => props.currentUser)
const isAdmin = computed(() => props.currentUser?.role === 'admin')
const visibleTabs = computed(() => isAdmin.value ? tabs : tabs.filter((tab) => tab.id === 'pos'))
const filteredProducts = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  return products.value.filter((product) => {
    const matchesQuery = !query ||
      product.name.toLowerCase().includes(query) ||
      product.sku.toLowerCase().includes(query) ||
      (product.barcode || '').toLowerCase().includes(query)
    const matchesCategory = !selectedCategory.value || product.category_id === Number(selectedCategory.value)
    return matchesQuery && matchesCategory
  })
})
const paginate = <T,>(items: T[], page: number, pageSize: number) => items.slice((page - 1) * pageSize, page * pageSize)
const paginatedRecentSales = computed(() => paginate(recentSales.value, pages.recentSales, pageSizes.recentSales))
const paginatedTopProducts = computed(() => paginate(topProducts.value, pages.topProducts, pageSizes.topProducts))
const paginatedFilteredProducts = computed(() => paginate(filteredProducts.value, pages.posProducts, pageSizes.posProducts))
const paginatedProducts = computed(() => paginate(products.value, pages.products, pageSizes.products))
const paginatedInventory = computed(() => paginate(inventory.value, pages.inventory, pageSizes.inventory))
const paginatedSalesByCategory = computed(() => paginate(salesByCategory.value, pages.salesByCategory, pageSizes.salesByCategory))
const lineTotal = (item: CartItem) => item.product.unit_price * item.quantity
const subtotal = computed(() => cart.value.reduce((sum, item) => sum + lineTotal(item), 0))
const vat = computed(() => subtotal.value * 16 / 116)
const total = computed(() => subtotal.value)
const change = computed(() => Math.max(0, amountReceived.value - total.value))

const money = (value: number | null | undefined) => Number(value || 0).toFixed(2)
const quantityLabel = (value: number | null | undefined) => {
  const numericValue = Number(value || 0)
  return numericValue.toFixed(3).replace(/\.?0+$/, '')
}
const normalizeQuantity = (value: number) => Math.round(value * 1000) / 1000
const categoryName = (id: number) => categories.value.find((category) => category.id === id)?.name || 'Unassigned'
const logout = () => emit('logout')
const settingValue = (key: string) => settings.value.find((setting) => setting.key === key)?.value || ''
const shortDate = (value?: string) => value ? new Date(value).toLocaleDateString() : '-'
const receiptStoreName = computed(() => settingValue('store_name') || 'Minimart POS')
const receiptStoreAddress = computed(() => settingValue('store_address'))
const receiptStorePhone = computed(() => settingValue('store_phone'))
const receiptFooter = computed(() => settingValue('receipt_footer'))

watch([searchQuery, selectedCategory], () => { pages.posProducts = 1 })
watch(products, () => { pages.products = 1 })
watch(inventory, () => { pages.inventory = 1 })
watch(recentSales, () => { pages.recentSales = 1 })
watch(topProducts, () => { pages.topProducts = 1 })
watch(salesByCategory, () => { pages.salesByCategory = 1 })
watch(isAdmin, (admin) => {
  if (!admin) activeView.value = 'pos'
}, { immediate: true })

const closePrintPopup = () => {
  printReceiptData.value = null
}

const printReceipt = async () => {
  await nextTick()
  window.print()
}

const loadProducts = async () => {
  products.value = await invoke<Product[]>('get_products', { categoryId: null })
}
const loadCategories = async () => {
  categories.value = await invoke<Category[]>('get_categories')
  if (!newProduct.category_id && categories.value[0]) newProduct.category_id = categories.value[0].id
}
const loadInventory = async () => {
  inventory.value = await invoke<InventoryStatus[]>('get_inventory_status')
}
const loadDashboard = async () => {
  dashboard.value = await invoke('get_dashboard_stats')
  recentSales.value = await invoke<any[]>('get_recent_sales', { limit: 8 })
  topProducts.value = await invoke<any[]>('get_top_products', { limit: 8 })
}
const loadReports = async () => {
  dailySummary.value = await invoke('get_daily_sales_summary')
  salesByCategory.value = await invoke<any[]>('get_sales_by_category')
}
const loadSettings = async () => {
  settings.value = await invoke<Setting[]>('get_settings')
  for (const setting of settings.value) {
    settingEdits[setting.key] = setting.value || ''
  }
}
const loadUsers = async () => {
  if (!isAdmin.value || !props.currentUser) return
  users.value = await invoke<User[]>('get_users', { adminUserId: props.currentUser.id })
}
const refreshAll = async () => {
  if (!isAdmin.value) {
    await Promise.all([loadProducts(), loadCategories(), loadSettings()])
    return
  }
  await Promise.all([loadProducts(), loadCategories(), loadInventory(), loadDashboard(), loadReports(), loadSettings(), loadUsers()])
}

const addToCart = (product: Product, barcodeScanned?: string) => {
  if (product.quantity_in_stock <= 0) {
    showToast('Out of stock', `${product.name} cannot be added to the cart.`, 'warning')
    return
  }
  const existingItem = cart.value.find((item) => item.product.id === product.id)
  if (existingItem) {
    if (existingItem.quantity < product.quantity_in_stock) {
      existingItem.quantity = normalizeQuantity(Math.min(existingItem.quantity + 1, product.quantity_in_stock))
      showToast('Cart updated', `${product.name} quantity increased.`, 'success', 2200)
    } else {
      showToast('Stock limit reached', `Only ${quantityLabel(product.quantity_in_stock)} ${product.name} available.`, 'warning')
    }
    return
  }
  cart.value.push({ product, quantity: 1, barcodeScanned })
  showToast('Added to cart', product.name, 'success', 2200)
}

const addBarcodeToCart = async () => {
  const value = barcodeQuery.value.trim()
  if (!value) return
  let product = products.value.find((candidate) => candidate.barcode === value || candidate.sku === value)
  if (!product) {
    try {
      product = await invoke<Product>('get_product_by_barcode', { barcode: value })
    } catch {
      product = filteredProducts.value[0]
    }
  }
  if (product) {
    addToCart(product, value)
    await invoke('log_barcode_scan', { productId: product.id, barcode: value }).catch(() => undefined)
  } else {
    showToast('Product not found', `No product matched "${value}".`, 'warning')
  }
  barcodeQuery.value = ''
  await nextTick()
  barcodeInput.value?.focus()
}

const updateQuantity = (index: number, quantity: number) => {
  const item = cart.value[index]
  if (!item) return
  if (quantity <= 0) {
    cart.value.splice(index, 1)
    return
  }
  if (quantity <= item.product.quantity_in_stock) {
    item.quantity = normalizeQuantity(quantity)
  } else {
    showToast('Stock limit reached', `Only ${quantityLabel(item.product.quantity_in_stock)} ${item.product.name} available.`, 'warning')
  }
}
const setQuantityFromInput = (index: number, event: Event) => {
  const quantity = Number((event.target as HTMLInputElement).value)
  if (Number.isFinite(quantity)) updateQuantity(index, quantity)
}
const setLineTotalFromInput = (index: number, event: Event) => {
  const item = cart.value[index]
  if (!item || item.product.unit_price <= 0) return
  const amount = Number((event.target as HTMLInputElement).value)
  if (!Number.isFinite(amount)) return
  updateQuantity(index, amount / item.product.unit_price)
}
const removeFromCart = (index: number) => cart.value.splice(index, 1)

const processPayment = async () => {
  saleMessage.value = ''
  saleError.value = false

  if (cart.value.length === 0) {
    saleError.value = true
    saleMessage.value = 'Add at least one product to the cart.'
    showToast('Cart is empty', saleMessage.value, 'warning')
    return
  }
  if (amountReceived.value < total.value) {
    saleError.value = true
    saleMessage.value = 'Insufficient payment amount.'
    showToast('Payment shortfall', saleMessage.value, 'error')
    return
  }

  processingPayment.value = true
  try {
    const receiptItems = cart.value.map((item) => ({
      name: item.product.name,
      quantity: item.quantity,
      lineTotal: lineTotal(item),
    }))
    const receiptSubtotal = subtotal.value
    const receiptVat = vat.value
    const receiptNetSales = receiptSubtotal - receiptVat
    const receiptCogs = cart.value.reduce(
      (sum, item) => sum + (item.product.cost_price ?? item.product.unit_price) * item.quantity,
      0,
    )
    const receiptProfit = receiptNetSales - receiptCogs
    const receiptMarginPercent = receiptNetSales > 0 ? (receiptProfit / receiptNetSales) * 100 : 0
    const receiptTotal = total.value
    const receiptPaid = amountReceived.value

    const sale: any = await invoke('create_sale', {
      sale: { customer_id: null, payment_method: paymentMethod.value, payment_amount: amountReceived.value },
    })
    for (const item of cart.value) {
      await invoke('add_sale_item', {
        saleId: sale.id,
        item: { product_id: item.product.id, quantity: item.quantity, barcode_scanned: item.barcodeScanned || null },
      })
    }
    const completedSale: any = await invoke('complete_sale', { saleId: sale.id, paymentMethod: paymentMethod.value })
    lastReceipt.value = {
      saleId: completedSale.id,
      createdAt: new Date().toLocaleString(),
      items: receiptItems,
      subtotal: receiptSubtotal,
      vat: receiptVat,
      netSales: receiptNetSales,
      cogs: receiptCogs,
      profit: receiptProfit,
      marginPercent: receiptMarginPercent,
      total: receiptTotal,
      paid: receiptPaid,
      change: completedSale.change_amount ?? Math.max(0, receiptPaid - receiptTotal),
    }
    printReceiptData.value = lastReceipt.value
    saleMessage.value = `Sale completed. Change: KES ${money(lastReceipt.value.change)}`
    showToast(
      'Sale completed',
      `Change: KES ${money(lastReceipt.value.change)} | Profit: KES ${money(lastReceipt.value.profit)} | Margin: ${money(lastReceipt.value.marginPercent)}%`,
      'success',
    )
    if (lastReceipt.value.profit < 0) {
      showToast('Loss-making sale', `Profit: KES ${money(lastReceipt.value.profit)}`, 'warning')
    }
    cart.value = []
    amountReceived.value = 0
    await refreshAll()
  } catch (error) {
    saleError.value = true
    saleMessage.value = String(error)
    showToast('Sale failed', String(error), 'error')
  } finally {
    processingPayment.value = false
  }
}

const createProduct = async () => {
  productFormMessage.value = ''
  productFormError.value = false

  try {
    await invoke('create_product', {
      product: {
        ...newProduct,
        barcode: newProduct.barcode || null,
        barcode_format: 'EAN13',
        description: null,
        cost_price: newProduct.cost_price || null,
        quantity_in_stock: newProduct.quantity_in_stock || 0,
        reorder_level: newProduct.reorder_level,
        expiry_date: null,
      },
    })
    Object.assign(newProduct, { name: '', sku: '', barcode: '', unit_price: 0, cost_price: null, quantity_in_stock: 0, reorder_level: 10 })
    await Promise.all([loadProducts(), loadInventory(), loadDashboard()])
    productFormMessage.value = 'Product created.'
    showToast('Product created', 'The product is now available in POS.', 'success')
  } catch (error) {
    productFormError.value = true
    productFormMessage.value = String(error)
    showToast('Create product failed', String(error), 'error')
  }
}

const startEdit = (product: Product) => {
  editingProductId.value = product.id
  Object.assign(editProduct, {
    name: product.name,
    sku: product.sku,
    category_id: product.category_id,
    barcode: product.barcode || '',
    unit_price: product.unit_price,
    cost_price: product.cost_price,
    quantity_in_stock: product.quantity_in_stock,
    reorder_level: product.reorder_level,
  })
  productFormMessage.value = ''
  productFormError.value = false
  nextTick(() => productEditForm.value?.scrollIntoView({ behavior: 'smooth', block: 'start' }))
}

const cancelEdit = () => {
  editingProductId.value = null
  productFormMessage.value = ''
  productFormError.value = false
}

const saveProductEdit = async () => {
  if (!editingProductId.value) return
  productFormMessage.value = ''
  productFormError.value = false

  try {
    await invoke('update_product', {
      id: editingProductId.value,
      updates: {
        name: editProduct.name,
        category_id: editProduct.category_id,
        sku: editProduct.sku,
        barcode: editProduct.barcode || null,
        barcode_format: 'EAN13',
        description: null,
        unit_price: editProduct.unit_price,
        cost_price: editProduct.cost_price || null,
        quantity_in_stock: editProduct.quantity_in_stock || 0,
        reorder_level: editProduct.reorder_level,
        expiry_date: null,
      },
    })
    editingProductId.value = null
    await Promise.all([loadProducts(), loadInventory(), loadDashboard()])
    productFormMessage.value = 'Product updated.'
    showToast('Product updated', 'Changes have been saved.', 'success')
  } catch (error) {
    productFormError.value = true
    productFormMessage.value = String(error)
    showToast('Update product failed', String(error), 'error')
  }
}

const deleteProduct = async (product: Product) => {
  productFormMessage.value = ''
  productFormError.value = false

  const confirmed = await showPrompt({
    title: 'Delete product?',
    message: `Delete ${product.name}? This action cannot be undone.`,
    confirmText: 'Delete',
    cancelText: 'Keep',
    type: 'danger',
  })
  if (!confirmed) return

  try {
    await invoke('delete_product', { id: product.id })
    cart.value = cart.value.filter((item) => item.product.id !== product.id)
    if (editingProductId.value === product.id) editingProductId.value = null
    await Promise.all([loadProducts(), loadInventory(), loadDashboard()])
    productFormMessage.value = 'Product deleted.'
    showToast('Product deleted', product.name, 'success')
  } catch (error) {
    productFormError.value = true
    productFormMessage.value = String(error)
    showToast('Delete product failed', String(error), 'error')
  }
}

const adjustStock = async (productId: number) => {
  const quantity = adjustments[productId] || 0
  if (!quantity) {
    showToast('No stock change', 'Enter a quantity before applying an adjustment.', 'warning')
    return
  }
  try {
    await invoke('adjust_inventory', { productId, quantity, reason: 'Manual stock adjustment' })
    adjustments[productId] = 0
    await Promise.all([loadProducts(), loadInventory()])
    showToast('Stock adjusted', `Inventory changed by ${quantity}.`, 'success')
  } catch (error) {
    showToast('Stock adjustment failed', String(error), 'error')
  }
}

const validateDb = async () => {
  try {
    dbValidation.value = JSON.stringify(await invoke('validate_database'), null, 2)
    showToast('Database validated', 'Validation details are shown in Settings.', 'success')
  } catch (error) {
    showToast('Database validation failed', String(error), 'error')
  }
}

const saveSetting = async (key: string) => {
  savingSettingKey.value = key
  try {
    await invoke('update_setting', {
      key,
      update: { value: settingEdits[key] ?? '' },
    })
    showToast('Setting saved', `${key} updated successfully.`, 'success')
    await loadSettings()
  } catch (error) {
    showToast('Setting save failed', String(error), 'error')
  } finally {
    savingSettingKey.value = ''
  }
}

const saveAllSettings = async () => {
  if (settings.value.length === 0) return
  savingAllSettings.value = true
  try {
    for (const setting of settings.value) {
      await invoke('update_setting', {
        key: setting.key,
        update: { value: settingEdits[setting.key] ?? '' },
      })
    }
    showToast('Settings saved', 'All settings have been updated.', 'success')
    await loadSettings()
  } catch (error) {
    showToast('Save all failed', String(error), 'error')
  } finally {
    savingAllSettings.value = false
  }
}

const resetUserForm = () => {
  editingUserId.value = null
  Object.assign(userForm, {
    username: '',
    email: '',
    role: 'user',
    password: '',
  })
}

const startUserEdit = (user: User) => {
  editingUserId.value = user.id
  Object.assign(userForm, {
    username: user.username,
    email: user.email,
    role: user.role,
    password: '',
  })
  userFormMessage.value = ''
  userFormError.value = false
  nextTick(() => userEditForm.value?.scrollIntoView({ behavior: 'smooth', block: 'start' }))
}

const cancelUserEdit = () => {
  resetUserForm()
  userFormMessage.value = ''
  userFormError.value = false
}

const saveUser = async () => {
  if (!props.currentUser) return
  userFormMessage.value = ''
  userFormError.value = false
  savingUser.value = true

  try {
    if (editingUserId.value) {
      await invoke('update_user', {
        adminUserId: props.currentUser.id,
        userId: editingUserId.value,
        updates: {
          username: userForm.username,
          email: userForm.email,
          role: userForm.role,
          password: userForm.password.trim() ? userForm.password : null,
        },
      })
      userFormMessage.value = 'User updated.'
      showToast('User updated', `${userForm.username} has been updated.`, 'success')
    } else {
      await invoke('create_user', {
        adminUserId: props.currentUser.id,
        user: {
          username: userForm.username,
          email: userForm.email,
          role: userForm.role,
          password: userForm.password,
        },
      })
      userFormMessage.value = 'User created.'
      showToast('User created', `${userForm.username} can now log in.`, 'success')
    }
    resetUserForm()
    await loadUsers()
  } catch (error) {
    userFormError.value = true
    userFormMessage.value = String(error)
    showToast('User save failed', String(error), 'error')
  } finally {
    savingUser.value = false
  }
}

const deleteUserAccount = async (user: User) => {
  if (!props.currentUser) return

  const confirmed = await showPrompt({
    title: 'Delete user?',
    message: `Delete ${user.username}? This action cannot be undone.`,
    confirmText: 'Delete',
    cancelText: 'Keep',
    type: 'danger',
  })
  if (!confirmed) return

  try {
    await invoke('delete_user', {
      adminUserId: props.currentUser.id,
      userId: user.id,
    })
    if (editingUserId.value === user.id) resetUserForm()
    await loadUsers()
    showToast('User deleted', user.username, 'success')
  } catch (error) {
    userFormError.value = true
    userFormMessage.value = String(error)
    showToast('Delete user failed', String(error), 'error')
  }
}

onMounted(async () => {
  try {
    await refreshAll()
    await nextTick()
    barcodeInput.value?.focus()
  } catch (error) {
    showToast('App data failed to load', String(error), 'error')
  }
})
</script>

<style scoped>
.app-shell { min-height: 100vh; display: grid; grid-template-columns: 240px 1fr; background: var(--color-black); color: var(--color-black); }
.sidebar { background: var(--color-black); color: var(--color-white); padding: 18px; display: flex; flex-direction: column; gap: 18px; border-right: 1px solid rgba(212, 175, 55, 0.35); }
.brand { display: grid; gap: 4px; padding-bottom: 12px; border-bottom: 1px solid rgba(212, 175, 55, 0.35); }
.brand strong { color: var(--color-gold); }
.brand span { color: var(--color-cream); font-size: 0.9rem; }
.nav-tabs { display: grid; gap: 8px; }
.nav-tabs button, .logout-btn, .view-header button, .scan-row button, .checkout button, .product-form button, .user-form button, td button { min-height: 38px; border: 0; border-radius: 6px; cursor: pointer; font-weight: 700; }
.nav-tabs button { display: flex; gap: 10px; align-items: center; padding: 10px 12px; background: transparent; color: var(--color-cream); text-align: left; border: 1px solid transparent; }
.nav-tabs button.active, .nav-tabs button:hover { background: var(--color-gold); color: var(--color-black); border-color: var(--color-gold); }
.logout-btn { margin-top: auto; background: transparent; color: var(--color-gold); border: 1px solid var(--color-gold); }
.logout-btn:hover { background: var(--color-gold); color: var(--color-black); }
.workspace { min-width: 0; overflow: auto; background: var(--color-cream); }
.view { padding: 20px; display: grid; gap: 18px; }
.view-header { display: flex; justify-content: space-between; align-items: center; gap: 12px; }
.view-header h1 { font-size: 1.6rem; }
.view-header button, .scan-row button, .checkout button, .product-form button, .user-form button, td button { background: var(--color-black); color: var(--color-gold); padding: 0 14px; border: 1px solid var(--color-black); }
.view-header button:hover, .scan-row button:hover, .checkout button:hover, .product-form button:hover, .user-form button:hover, td button:hover { background: var(--color-gold); color: var(--color-black); border-color: var(--color-gold); }
.checkout button:disabled { opacity: 0.5; cursor: not-allowed; }
.secondary-btn { background: var(--color-white) !important; color: var(--color-black) !important; border: 1px solid var(--color-gold) !important; }
.danger-btn { background: var(--color-danger) !important; color: var(--color-white) !important; border-color: var(--color-danger) !important; }
.pos-view { grid-template-columns: minmax(0, 1fr) 380px; align-items: start; }
.sale-surface, .cart-panel, .panel { background: var(--color-white); border: 1px solid var(--color-border); border-radius: 8px; padding: 16px; box-shadow: 0 12px 30px rgba(10, 10, 10, 0.06); }
.scan-row, .search-controls, .product-form, .user-form { display: grid; grid-template-columns: 1fr auto; gap: 10px; }
.search-controls { grid-template-columns: 1fr 220px; margin: 12px 0; }
input, select { min-height: 40px; border: 1px solid #d7c58b; border-radius: 6px; padding: 0 10px; background: var(--color-white); color: var(--color-black); }
input:focus, select:focus { outline: none; border-color: var(--color-gold); box-shadow: 0 0 0 3px rgba(212, 175, 55, 0.18); }
.products-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(170px, 1fr)); gap: 10px; }
.product-tile { min-height: 132px; display: grid; gap: 6px; text-align: left; padding: 12px; border: 1px solid var(--color-border); background: #fffaf0; border-radius: 8px; cursor: pointer; color: var(--color-black); }
.product-tile:hover:not(:disabled) { border-color: var(--color-gold); box-shadow: 0 10px 22px rgba(212, 175, 55, 0.16); }
.product-tile:disabled { opacity: 0.5; cursor: not-allowed; }
.product-tile em { font-style: normal; font-weight: 800; color: var(--color-gold-dark); }
.low, .warning strong, .out_of_stock { color: var(--color-danger); }
.cart-panel { position: sticky; top: 0; display: grid; gap: 14px; }
.cart-list { display: grid; gap: 10px; max-height: 42vh; overflow: auto; }
.cart-item { display: grid; grid-template-columns: minmax(0, 1fr) auto; align-items: start; gap: 8px 12px; border-bottom: 1px solid var(--color-border); padding-bottom: 10px; }
.cart-item-summary { grid-column: 1 / -1; display: grid; gap: 4px; min-width: 0; }
.cart-item-summary strong, .cart-item-summary span { overflow-wrap: anywhere; }
.cart-item span, .empty-state { color: var(--color-muted); }
.cart-entry-controls { display: grid; gap: 6px; }
.cart-entry-controls label { display: grid; grid-template-columns: 42px 1fr; align-items: center; gap: 6px; font-size: 0.82rem; font-weight: 800; color: var(--color-muted); }
.cart-entry-controls input { min-height: 30px; width: 100%; padding: 0 8px; }
.qty-controls { display: flex; align-items: center; gap: 6px; }
.qty-controls button { width: 30px; height: 30px; border: 1px solid var(--color-border); background: var(--color-white); color: var(--color-black); border-radius: 6px; }
.qty-controls .danger { color: var(--color-danger); }
.totals { display: grid; gap: 8px; border-top: 2px solid var(--color-border); padding-top: 12px; }
.totals div { display: flex; justify-content: space-between; }
.totals .grand { font-size: 1.2rem; }
.checkout { display: grid; gap: 10px; }
.change { color: var(--color-gold-dark); font-weight: 800; }
.receipt-panel { display: grid; gap: 10px; border-top: 2px solid var(--color-border); padding-top: 14px; }
.receipt-panel header, .receipt-line, .receipt-totals div { display: flex; justify-content: space-between; gap: 12px; }
.receipt-panel header { align-items: center; }
.receipt-panel header > div { display: grid; gap: 3px; }
.receipt-panel h3 { margin: 0; font-size: 1rem; }
.receipt-panel header span, .receipt-line span, .receipt-totals span { color: var(--color-muted); }
.receipt-line { border-bottom: 1px solid var(--color-border); padding-bottom: 8px; }
.receipt-totals { display: grid; gap: 6px; }
.receipt-totals .grand { font-size: 1.1rem; }
.loss { color: var(--color-danger); font-weight: 900; }
.loss-note { margin: 0; color: var(--color-danger); font-weight: 800; }
.icon-btn { width: 38px; height: 38px; display: inline-grid; place-items: center; border: 1px solid var(--color-gold); border-radius: 6px; background: var(--color-black); color: var(--color-gold); cursor: pointer; }
.icon-btn:hover { background: var(--color-gold); color: var(--color-black); }
.icon-btn svg { width: 18px; height: 18px; fill: none; stroke: currentColor; stroke-width: 2; stroke-linecap: round; stroke-linejoin: round; }
.close-btn { font-size: 1rem; font-weight: 900; }
.print-popup { position: fixed; inset: 0; z-index: 50; display: grid; place-items: center; padding: 20px; background: rgba(10, 10, 10, 0.7); }
.print-dialog { width: min(440px, 100%); max-height: calc(100vh - 40px); overflow: auto; display: grid; gap: 14px; padding: 16px; background: var(--color-white); border: 1px solid var(--color-gold); border-radius: 8px; box-shadow: 0 24px 70px rgba(0, 0, 0, 0.35); }
.print-actions { display: flex; justify-content: space-between; align-items: center; gap: 12px; padding-bottom: 12px; border-bottom: 1px solid var(--color-border); }
.print-actions > div:first-child { display: grid; gap: 3px; }
.print-actions span { color: var(--color-muted); font-size: 0.88rem; }
.print-actions > div:last-child { display: flex; gap: 8px; }
.print-receipt { display: grid; gap: 14px; padding: 14px; background: #fffaf0; border: 1px solid var(--color-border); border-radius: 8px; color: var(--color-black); }
.print-receipt > header { display: grid; gap: 4px; text-align: center; padding-bottom: 12px; border-bottom: 1px solid var(--color-border); }
.print-receipt > header strong { font-size: 1.15rem; }
.print-receipt > header span, .print-receipt > header small { color: var(--color-muted); }
.print-receipt > header small { display: block; }
.print-items, .print-totals { display: grid; gap: 8px; }
.print-line, .print-totals div { display: flex; justify-content: space-between; gap: 12px; }
.print-line { padding-bottom: 8px; border-bottom: 1px solid var(--color-border); }
.print-line span, .print-totals span { color: var(--color-muted); }
.print-totals { padding-top: 4px; }
.print-totals .grand { padding-top: 8px; border-top: 2px solid var(--color-gold); font-size: 1.12rem; }
.print-footer { padding-top: 10px; border-top: 1px solid var(--color-border); text-align: center; color: var(--color-muted); font-size: 0.9rem; }
.metrics-grid { display: grid; grid-template-columns: repeat(4, minmax(150px, 1fr)); gap: 12px; }
.metric { background: var(--color-white); border: 1px solid var(--color-border); border-radius: 8px; padding: 16px; display: grid; gap: 8px; }
.metric span { color: var(--color-muted); }
.metric strong { font-size: 1.6rem; }
.split-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
.settings-actions { display: flex; gap: 8px; flex-wrap: wrap; }
.accounting-tabs { display: flex; gap: 8px; flex-wrap: wrap; }
.accounting-tabs button { min-height: 38px; border-radius: 6px; border: 1px solid var(--color-gold); background: var(--color-white); color: var(--color-black); padding: 0 12px; cursor: pointer; font-weight: 800; }
.accounting-tabs button.active, .accounting-tabs button:hover { background: var(--color-black); color: var(--color-gold); border-color: var(--color-black); }
.product-form { grid-template-columns: repeat(4, minmax(140px, 1fr)) auto; }
.user-form { grid-template-columns: repeat(4, minmax(140px, 1fr)) auto auto; margin-bottom: 14px; }
.edit-form { border-top: 1px solid var(--color-border); padding-top: 14px; }
.form-message { margin: -8px 0 0; color: var(--color-gold-dark); font-weight: 800; }
.form-message.error { color: var(--color-danger); }
table { width: 100%; border-collapse: collapse; }
th, td { padding: 10px; border-bottom: 1px solid var(--color-border); text-align: left; vertical-align: middle; }
th { color: var(--color-muted); }
td input { width: 90px; margin-right: 6px; }
.action-cell { display: flex; gap: 6px; flex-wrap: wrap; }
.status { text-transform: capitalize; font-weight: 800; }
.role-pill { display: inline-block; min-width: 68px; padding: 4px 8px; border-radius: 6px; background: #fffaf0; border: 1px solid var(--color-border); text-transform: capitalize; font-weight: 800; text-align: center; }
.in_stock { color: var(--color-gold-dark); }
.low_stock { color: #8a641d; }
pre { white-space: pre-wrap; background: var(--color-black); color: var(--color-gold-soft); padding: 14px; border-radius: 8px; }

@media (max-width: 980px) {
  .app-shell { grid-template-columns: 1fr; }
  .sidebar { position: sticky; top: 0; z-index: 2; }
  .nav-tabs { grid-template-columns: repeat(3, 1fr); }
  .pos-view, .split-grid { grid-template-columns: 1fr; }
  .cart-panel { position: static; }
  .cart-item { grid-template-columns: 1fr; }
  .metrics-grid, .product-form, .user-form { grid-template-columns: 1fr 1fr; }
}

@media print {
  .app-shell,
  .print-actions,
  :global(.toast-stack),
  :global(.prompt-backdrop) {
    display: none !important;
  }

  .print-popup {
    position: static;
    inset: auto;
    display: block;
    padding: 0;
    background: var(--color-white);
  }

  .print-dialog {
    width: 100%;
    max-height: none;
    overflow: visible;
    padding: 0;
    border: 0;
    border-radius: 0;
    box-shadow: none;
  }

  .print-receipt {
    border: 0;
    border-radius: 0;
    background: var(--color-white);
  }
}
</style>
