<template>
  <div class="app-shell">
    <aside class="sidebar">
      <div class="brand">
        <strong>Minimart POS</strong>
        <span>{{ currentUser?.username }}</span>
      </div>
      <nav class="nav-tabs">
        <button v-for="tab in tabs" :key="tab.id" :class="{ active: activeView === tab.id }" @click="activeView = tab.id">
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
                <tr v-for="sale in recentSales" :key="sale.id">
                  <td>#{{ sale.id }}</td>
                  <td>{{ sale.customer_name }}</td>
                  <td>KES {{ money(sale.total_amount) }}</td>
                </tr>
                <tr v-if="recentSales.length === 0"><td colspan="3">No completed sales yet.</td></tr>
              </tbody>
            </table>
          </section>
          <section class="panel">
            <h2>Top Products</h2>
            <table>
              <tbody>
                <tr v-for="product in topProducts" :key="product.name">
                  <td>{{ product.name }}</td>
                  <td>{{ product.total_quantity }} sold</td>
                  <td>KES {{ money(product.total_revenue) }}</td>
                </tr>
                <tr v-if="topProducts.length === 0"><td colspan="3">No product sales yet.</td></tr>
              </tbody>
            </table>
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
              v-for="product in filteredProducts"
              :key="product.id"
              class="product-tile"
              :disabled="product.quantity_in_stock <= 0"
              @click="addToCart(product, product.barcode || undefined)"
            >
              <strong>{{ product.name }}</strong>
              <span>{{ product.sku }}</span>
              <em>KES {{ money(product.unit_price) }}</em>
              <small :class="{ low: product.quantity_in_stock <= product.reorder_level }">
                Stock {{ product.quantity_in_stock }}
              </small>
            </button>
          </div>
        </div>

        <aside class="cart-panel">
          <h2>Cart</h2>
          <div class="cart-list">
            <p v-if="cart.length === 0" class="empty-state">No items in cart.</p>
            <div v-for="(item, index) in cart" :key="item.product.id" class="cart-item">
              <div>
                <strong>{{ item.product.name }}</strong>
                <span>KES {{ money(item.product.unit_price) }} x {{ item.quantity }}</span>
              </div>
              <div class="qty-controls">
                <button @click="updateQuantity(index, item.quantity - 1)">-</button>
                <b>{{ item.quantity }}</b>
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
        <section v-if="lastReceipt" class="receipt-panel">
          <header>
            <h3>Receipt #{{ lastReceipt.saleId }}</h3>
            <span>{{ lastReceipt.createdAt }}</span>
          </header>
          <div v-for="item in lastReceipt.items" :key="item.name" class="receipt-line">
            <span>{{ item.name }} x {{ item.quantity }}</span>
            <strong>KES {{ money(item.lineTotal) }}</strong>
          </div>
          <div class="receipt-totals">
            <div><span>Subtotal</span><strong>KES {{ money(lastReceipt.subtotal) }}</strong></div>
            <div><span>VAT 16% included</span><strong>KES {{ money(lastReceipt.vat) }}</strong></div>
            <div class="grand"><span>Total</span><strong>KES {{ money(lastReceipt.total) }}</strong></div>
            <div><span>Paid</span><strong>KES {{ money(lastReceipt.paid) }}</strong></div>
            <div><span>Change</span><strong>KES {{ money(lastReceipt.change) }}</strong></div>
          </div>
        </section>
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
          <input v-model.number="newProduct.quantity_in_stock" type="number" min="0" placeholder="Initial stock" />
          <input v-model.number="newProduct.reorder_level" type="number" min="0" placeholder="Reorder" />
          <input v-model="newProduct.barcode" placeholder="Barcode optional" />
          <button>Create Product</button>
        </form>
        <form v-if="editingProductId" class="product-form edit-form" @submit.prevent="saveProductEdit">
          <input v-model="editProduct.name" placeholder="Name" required />
          <input v-model="editProduct.sku" placeholder="SKU" required />
          <select v-model.number="editProduct.category_id" required>
            <option v-for="category in categories" :key="category.id" :value="category.id">{{ category.name }}</option>
          </select>
          <input v-model.number="editProduct.unit_price" type="number" min="0" step="0.01" placeholder="Price" required />
          <input v-model.number="editProduct.cost_price" type="number" min="0" step="0.01" placeholder="Cost" />
          <input v-model.number="editProduct.quantity_in_stock" type="number" min="0" placeholder="Stock" />
          <input v-model.number="editProduct.reorder_level" type="number" min="0" placeholder="Reorder" />
          <input v-model="editProduct.barcode" placeholder="Barcode optional" />
          <button>Save</button>
          <button type="button" class="secondary-btn" @click="cancelEdit">Cancel</button>
        </form>
        <p v-if="productFormMessage" class="form-message" :class="{ error: productFormError }">{{ productFormMessage }}</p>
        <section class="panel">
          <table>
            <thead><tr><th>Name</th><th>SKU</th><th>Barcode</th><th>Category</th><th>Price</th><th>Stock</th><th>Actions</th></tr></thead>
            <tbody>
              <tr v-for="product in products" :key="product.id">
                <td>{{ product.name }}</td>
                <td>{{ product.sku }}</td>
                <td>{{ product.barcode || '-' }}</td>
                <td>{{ categoryName(product.category_id) }}</td>
                <td>KES {{ money(product.unit_price) }}</td>
                <td>{{ product.quantity_in_stock }}</td>
                <td class="action-cell">
                  <button @click="startEdit(product)">Edit</button>
                  <button class="danger-btn" @click="deleteProduct(product)">Delete</button>
                </td>
              </tr>
            </tbody>
          </table>
        </section>
      </section>

      <section v-if="activeView === 'inventory'" class="view">
        <header class="view-header"><h1>Inventory</h1><button @click="loadInventory">Reload</button></header>
        <section class="panel">
          <table>
            <thead><tr><th>Product</th><th>Stock</th><th>Reorder</th><th>Status</th><th>Adjust</th></tr></thead>
            <tbody>
              <tr v-for="item in inventory" :key="item.product_id">
                <td>{{ item.product_name }}</td>
                <td>{{ item.current_stock }}</td>
                <td>{{ item.reorder_level }}</td>
                <td><span class="status" :class="item.status">{{ item.status.replace('_', ' ') }}</span></td>
                <td>
                  <input v-model.number="adjustments[item.product_id]" type="number" />
                  <button @click="adjustStock(item.product_id)">Apply</button>
                </td>
              </tr>
            </tbody>
          </table>
        </section>
      </section>

      <section v-if="activeView === 'reports'" class="view">
        <header class="view-header"><h1>Reports</h1><button @click="loadReports">Refresh</button></header>
        <div class="metrics-grid">
          <div class="metric"><span>Sales Today</span><strong>{{ dailySummary.sale_count ?? 0 }}</strong></div>
          <div class="metric"><span>Items Sold</span><strong>{{ dailySummary.items_sold ?? 0 }}</strong></div>
          <div class="metric"><span>Revenue</span><strong>KES {{ money(dailySummary.total_sales) }}</strong></div>
        </div>
        <section class="panel">
          <h2>Sales by Category</h2>
          <table>
            <tbody>
              <tr v-for="row in salesByCategory" :key="row.name">
                <td>{{ row.icon }} {{ row.name }}</td>
                <td>{{ row.product_count }} products</td>
                <td>KES {{ money(row.total_revenue) }}</td>
              </tr>
            </tbody>
          </table>
        </section>
      </section>

      <section v-if="activeView === 'settings'" class="view">
        <header class="view-header"><h1>Settings</h1><button @click="validateDb">Validate DB</button></header>
        <section class="panel">
          <table>
            <tbody>
              <tr v-for="setting in settings" :key="setting.key">
                <td>{{ setting.key }}</td>
                <td>{{ setting.value }}</td>
                <td>{{ setting.description }}</td>
              </tr>
            </tbody>
          </table>
        </section>
        <pre v-if="dbValidation">{{ dbValidation }}</pre>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, reactive, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface User { id: number; username: string; email: string; role: string }
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
interface ReceiptItem { name: string; quantity: number; lineTotal: number }
interface Receipt {
  saleId: number
  createdAt: string
  items: ReceiptItem[]
  subtotal: number
  vat: number
  total: number
  paid: number
  change: number
}

defineProps<{ currentUser: User | null }>()
const emit = defineEmits<{ logout: [] }>()

const tabs = [
  { id: 'dashboard', label: 'Dashboard', icon: '▦' },
  { id: 'pos', label: 'POS', icon: '▣' },
  { id: 'products', label: 'Products', icon: '□' },
  { id: 'inventory', label: 'Inventory', icon: '≡' },
  { id: 'reports', label: 'Reports', icon: '△' },
  { id: 'settings', label: 'Settings', icon: '⚙' },
]

const activeView = ref('pos')
const products = ref<Product[]>([])
const categories = ref<Category[]>([])
const inventory = ref<InventoryStatus[]>([])
const cart = ref<CartItem[]>([])
const dashboard = ref<any>(null)
const recentSales = ref<any[]>([])
const topProducts = ref<any[]>([])
const salesByCategory = ref<any[]>([])
const settings = ref<any[]>([])
const dailySummary = ref<any>({})
const dbValidation = ref('')
const adjustments = reactive<Record<number, number>>({})

const barcodeInput = ref<HTMLInputElement | null>(null)
const barcodeQuery = ref('')
const searchQuery = ref('')
const selectedCategory = ref('')
const paymentMethod = ref('cash')
const amountReceived = ref(0)
const processingPayment = ref(false)
const saleMessage = ref('')
const saleError = ref(false)
const lastReceipt = ref<Receipt | null>(null)
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
const subtotal = computed(() => cart.value.reduce((sum, item) => sum + item.product.unit_price * item.quantity, 0))
const vat = computed(() => subtotal.value * 16 / 116)
const total = computed(() => subtotal.value)
const change = computed(() => Math.max(0, amountReceived.value - total.value))

const money = (value: number | null | undefined) => Number(value || 0).toFixed(2)
const categoryName = (id: number) => categories.value.find((category) => category.id === id)?.name || 'Unassigned'
const logout = () => emit('logout')

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
  settings.value = await invoke<any[]>('get_settings')
}
const refreshAll = async () => {
  await Promise.all([loadProducts(), loadCategories(), loadInventory(), loadDashboard(), loadReports(), loadSettings()])
}

const addToCart = (product: Product, barcodeScanned?: string) => {
  if (product.quantity_in_stock <= 0) return
  const existingItem = cart.value.find((item) => item.product.id === product.id)
  if (existingItem) {
    if (existingItem.quantity < product.quantity_in_stock) existingItem.quantity += 1
    return
  }
  cart.value.push({ product, quantity: 1, barcodeScanned })
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
  }
  barcodeQuery.value = ''
  await nextTick()
  barcodeInput.value?.focus()
}

const updateQuantity = (index: number, quantity: number) => {
  if (quantity <= 0) cart.value.splice(index, 1)
  else if (quantity <= cart.value[index].product.quantity_in_stock) cart.value[index].quantity = quantity
}
const removeFromCart = (index: number) => cart.value.splice(index, 1)

const processPayment = async () => {
  saleMessage.value = ''
  saleError.value = false

  if (cart.value.length === 0) {
    saleError.value = true
    saleMessage.value = 'Add at least one product to the cart.'
    return
  }
  if (amountReceived.value < total.value) {
    saleError.value = true
    saleMessage.value = 'Insufficient payment amount.'
    return
  }

  processingPayment.value = true
  try {
    const receiptItems = cart.value.map((item) => ({
      name: item.product.name,
      quantity: item.quantity,
      lineTotal: item.product.unit_price * item.quantity,
    }))
    const receiptSubtotal = subtotal.value
    const receiptVat = vat.value
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
      total: receiptTotal,
      paid: receiptPaid,
      change: completedSale.change_amount ?? Math.max(0, receiptPaid - receiptTotal),
    }
    saleMessage.value = `Sale completed. Change: KES ${money(lastReceipt.value.change)}`
    cart.value = []
    amountReceived.value = 0
    await refreshAll()
  } catch (error) {
    saleError.value = true
    saleMessage.value = String(error)
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
  } catch (error) {
    productFormError.value = true
    productFormMessage.value = String(error)
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
  } catch (error) {
    productFormError.value = true
    productFormMessage.value = String(error)
  }
}

const deleteProduct = async (product: Product) => {
  productFormMessage.value = ''
  productFormError.value = false

  if (!confirm(`Delete ${product.name}?`)) return

  try {
    await invoke('delete_product', { id: product.id })
    cart.value = cart.value.filter((item) => item.product.id !== product.id)
    if (editingProductId.value === product.id) editingProductId.value = null
    await Promise.all([loadProducts(), loadInventory(), loadDashboard()])
    productFormMessage.value = 'Product deleted.'
  } catch (error) {
    productFormError.value = true
    productFormMessage.value = String(error)
  }
}

const adjustStock = async (productId: number) => {
  const quantity = adjustments[productId] || 0
  if (!quantity) return
  await invoke('adjust_inventory', { productId, quantity, reason: 'Manual stock adjustment' })
  adjustments[productId] = 0
  await Promise.all([loadProducts(), loadInventory()])
}

const validateDb = async () => {
  dbValidation.value = JSON.stringify(await invoke('validate_database'), null, 2)
}

onMounted(async () => {
  await refreshAll()
  await nextTick()
  barcodeInput.value?.focus()
})
</script>

<style scoped>
.app-shell { min-height: 100vh; display: grid; grid-template-columns: 240px 1fr; background: #eef2f5; color: #17202a; }
.sidebar { background: #111827; color: white; padding: 18px; display: flex; flex-direction: column; gap: 18px; }
.brand { display: grid; gap: 4px; padding-bottom: 12px; border-bottom: 1px solid #374151; }
.brand span { color: #cbd5e1; font-size: 0.9rem; }
.nav-tabs { display: grid; gap: 8px; }
.nav-tabs button, .logout-btn, .view-header button, .scan-row button, .checkout button, .product-form button, td button { min-height: 38px; border: 0; border-radius: 6px; cursor: pointer; font-weight: 700; }
.nav-tabs button { display: flex; gap: 10px; align-items: center; padding: 10px 12px; background: transparent; color: #d1d5db; text-align: left; }
.nav-tabs button.active, .nav-tabs button:hover { background: #2563eb; color: white; }
.logout-btn { margin-top: auto; background: #b91c1c; color: white; }
.workspace { min-width: 0; overflow: auto; }
.view { padding: 20px; display: grid; gap: 18px; }
.view-header { display: flex; justify-content: space-between; align-items: center; gap: 12px; }
.view-header h1 { font-size: 1.6rem; }
.view-header button, .scan-row button, .checkout button, .product-form button, td button { background: #2563eb; color: white; padding: 0 14px; }
.secondary-btn { background: #64748b !important; color: white; }
.danger-btn { background: #b91c1c !important; color: white; }
.pos-view { grid-template-columns: minmax(0, 1fr) 380px; align-items: start; }
.sale-surface, .cart-panel, .panel { background: white; border: 1px solid #d9e2ec; border-radius: 8px; padding: 16px; }
.scan-row, .search-controls, .product-form { display: grid; grid-template-columns: 1fr auto; gap: 10px; }
.search-controls { grid-template-columns: 1fr 220px; margin: 12px 0; }
input, select { min-height: 40px; border: 1px solid #cbd5e1; border-radius: 6px; padding: 0 10px; background: white; color: #111827; }
.products-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(170px, 1fr)); gap: 10px; }
.product-tile { min-height: 132px; display: grid; gap: 6px; text-align: left; padding: 12px; border: 1px solid #d9e2ec; background: #f8fafc; border-radius: 8px; cursor: pointer; }
.product-tile:disabled { opacity: 0.5; cursor: not-allowed; }
.product-tile em { font-style: normal; font-weight: 800; color: #047857; }
.low, .warning strong, .out_of_stock { color: #b91c1c; }
.cart-panel { position: sticky; top: 0; display: grid; gap: 14px; }
.cart-list { display: grid; gap: 10px; max-height: 42vh; overflow: auto; }
.cart-item { display: flex; justify-content: space-between; gap: 12px; border-bottom: 1px solid #e5e7eb; padding-bottom: 10px; }
.cart-item div:first-child { display: grid; gap: 4px; }
.cart-item span, .empty-state { color: #64748b; }
.qty-controls { display: flex; align-items: center; gap: 6px; }
.qty-controls button { width: 30px; height: 30px; border: 1px solid #cbd5e1; background: white; border-radius: 6px; }
.qty-controls .danger { color: #b91c1c; }
.totals { display: grid; gap: 8px; border-top: 2px solid #e5e7eb; padding-top: 12px; }
.totals div { display: flex; justify-content: space-between; }
.totals .grand { font-size: 1.2rem; }
.checkout { display: grid; gap: 10px; }
.change { color: #047857; font-weight: 800; }
.receipt-panel { display: grid; gap: 10px; border-top: 2px solid #e5e7eb; padding-top: 14px; }
.receipt-panel header, .receipt-line, .receipt-totals div { display: flex; justify-content: space-between; gap: 12px; }
.receipt-panel h3 { margin: 0; font-size: 1rem; }
.receipt-panel header span, .receipt-line span, .receipt-totals span { color: #64748b; }
.receipt-line { border-bottom: 1px solid #eef2f5; padding-bottom: 8px; }
.receipt-totals { display: grid; gap: 6px; }
.receipt-totals .grand { font-size: 1.1rem; }
.metrics-grid { display: grid; grid-template-columns: repeat(4, minmax(150px, 1fr)); gap: 12px; }
.metric { background: white; border: 1px solid #d9e2ec; border-radius: 8px; padding: 16px; display: grid; gap: 8px; }
.metric span { color: #64748b; }
.metric strong { font-size: 1.6rem; }
.split-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
.product-form { grid-template-columns: repeat(4, minmax(140px, 1fr)) auto; }
.edit-form { border-top: 1px solid #e5e7eb; padding-top: 14px; }
.form-message { margin: -8px 0 0; color: #047857; font-weight: 800; }
.form-message.error { color: #b91c1c; }
table { width: 100%; border-collapse: collapse; }
th, td { padding: 10px; border-bottom: 1px solid #e5e7eb; text-align: left; vertical-align: middle; }
td input { width: 90px; margin-right: 6px; }
.action-cell { display: flex; gap: 6px; flex-wrap: wrap; }
.status { text-transform: capitalize; font-weight: 800; }
.in_stock { color: #047857; }
.low_stock { color: #b45309; }
pre { white-space: pre-wrap; background: #111827; color: #e5e7eb; padding: 14px; border-radius: 8px; }

@media (max-width: 980px) {
  .app-shell { grid-template-columns: 1fr; }
  .sidebar { position: sticky; top: 0; z-index: 2; }
  .nav-tabs { grid-template-columns: repeat(3, 1fr); }
  .pos-view, .split-grid { grid-template-columns: 1fr; }
  .cart-panel { position: static; }
  .metrics-grid, .product-form { grid-template-columns: 1fr 1fr; }
}
</style>
