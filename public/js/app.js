const API_URL = 'http://localhost:3000/api';

// Helper for authenticated fetch
async function authFetch(url, options = {}) {
    const headers = {
        'Content-Type': 'application/json',
        ...options.headers
    };
    if (currentToken) {
        headers['Authorization'] = `Bearer ${currentToken}`;
    }
    const response = await fetch(url, { ...options, headers });
    if (response.status === 401) {
        // Token expired or invalid - logout
        if (currentToken) handleLogout();
    }
    return response;
}
const modal = document.getElementById('modal');
const modalBody = document.getElementById('modal-body');
const modalContent = document.querySelector('.modal-content');

let currentUser = JSON.parse(localStorage.getItem('user')) || null;
let currentToken = localStorage.getItem('token') || null;

// Wait for DOM to be ready
document.addEventListener('DOMContentLoaded', () => {
    // Auth form
    const loginForm = document.getElementById('login-form');
    if (loginForm) loginForm.addEventListener('submit', handleLogin);

    // Initial Auth Check
    checkAuth();

    // Modal close button
    const closeBtn = document.querySelector('.close');
    if (closeBtn) closeBtn.onclick = () => modal.classList.remove('active');

    window.onclick = (e) => {
        if (e.target === modal) modal.classList.remove('active');
    };

    // Dashboard Buttons
    document.getElementById('add-product-btn')?.addEventListener('click', () => showProductForm());
    document.getElementById('add-customer-btn')?.addEventListener('click', () => showCustomerForm());
    document.getElementById('new-sale-btn')?.addEventListener('click', () => showSaleForm());
    document.getElementById('generate-report-btn')?.addEventListener('click', generateReport);

    if (currentUser) {
        loadDashboard();
        loadSales();
        if (currentUser.role === 'admin') {
            loadProducts();
            loadCustomers();
            loadInventory();
        }
    }
});

// ------------------- Auth -------------------
async function handleLogin(e) {
    e.preventDefault();
    const username = document.getElementById('login-username').value;
    const password = document.getElementById('login-password').value;
    const errorEl = document.getElementById('login-error');

    try {
        const response = await authFetch(`${API_URL}/auth/login`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, password })
        });

        if (!response.ok) {
            const data = await response.json();
            throw new Error(data.error || 'Login failed');
        }

        const data = await response.json();
        currentToken = data.token;
        currentUser = data.user;

        localStorage.setItem('token', currentToken);
        localStorage.setItem('user', JSON.stringify(currentUser));

        checkAuth();

        // Initial data load
        loadDashboard();
        loadSales();
        if (currentUser.role === 'admin') {
            loadProducts();
            loadCustomers();
            loadInventory();
        }

    } catch (err) {
        errorEl.textContent = err.message;
        errorEl.style.display = 'block';
    }
}

function handleLogout() {
    currentToken = null;
    currentUser = null;
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    checkAuth();
}

function checkAuth() {
    const authContainer = document.getElementById('auth-container');
    const appContainer = document.getElementById('app-container');

    if (currentToken && currentUser) {
        authContainer.style.display = 'none';
        appContainer.classList.add('active');
        const userLabel = document.getElementById('topbar-user-label');
        if (userLabel) userLabel.textContent = currentUser.username;
        applyRoleRestrictions();
    } else {
        authContainer.style.display = 'flex';
        appContainer.classList.remove('active');
    }
}

function applyRoleRestrictions() {
    if (!currentUser) return;

    const role = currentUser.role;
    const adminNavItems = ['nav-dashboard', 'nav-products', 'nav-inventory', 'nav-customers', 'nav-reports'];

    if (role === 'admin') {
        adminNavItems.forEach(id => {
            const el = document.getElementById(id);
            if (el) el.style.display = 'block';
        });
    } else {
        // Normal user only sees Sales
        adminNavItems.forEach(id => {
            const el = document.getElementById(id);
            if (el) el.style.display = 'none';
        });
        showSection('sales');
        document.getElementById('nav-sales').classList.add('active');
    }
}

// ------------------- Dashboard -------------------
async function loadDashboard() {
    if (!currentUser) return;
    try {
        const isAdmin = currentUser.role === 'admin';

        let products = [], sales = [], inventory = [], customers = [];

        if (isAdmin) {
            [products, sales, inventory, customers] = await Promise.all([
                authFetch(`${API_URL}/products`).then(r => r.ok ? r.json() : []),
                authFetch(`${API_URL}/sales`).then(r => r.ok ? r.json() : []),
                authFetch(`${API_URL}/inventory/low-stock`).then(r => r.ok ? r.json() : []),
                authFetch(`${API_URL}/customers`).then(r => r.ok ? r.json() : [])
            ]);
        } else {
            // Non-admin only gets sales
            sales = await authFetch(`${API_URL}/sales`).then(r => r.ok ? r.json() : []);
        }

        const totalProdsEl = document.getElementById('total-products');
        if (totalProdsEl) totalProdsEl.textContent = isAdmin ? products.length : 'N/A';

        const lowStockEl = document.getElementById('low-stock');
        if (lowStockEl) lowStockEl.textContent = isAdmin ? inventory.length : 'N/A';

        const totalCustEl = document.getElementById('total-customers');
        if (totalCustEl) totalCustEl.textContent = isAdmin ? customers.length : 'N/A';

        const totalSales = Array.isArray(sales) ? sales.reduce((sum, sale) => sum + parseFloat(sale.total_amount), 0) : 0;
        document.getElementById('total-sales').textContent = `KES ${totalSales.toFixed(2)}`;

        // Recent sales
        const tbody = document.querySelector('#recent-sales-table tbody');
        if (tbody && Array.isArray(sales)) {
            tbody.innerHTML = sales.slice(0, 5).map(sale => `
                <tr>
                    <td>${sale.id}</td>
                    <td>${sale.customer_name || 'Walk-in'}</td>
                    <td>KES ${parseFloat(sale.total_amount).toFixed(2)}</td>
                    <td>${new Date(sale.sale_date).toLocaleDateString()}</td>
                </tr>
            `).join('');
        }
    } catch (err) {
        console.error('Dashboard error:', err);
    }
}

// ------------------- Products -------------------
async function loadProducts() {
    try {
        const res = await authFetch(`${API_URL}/products`);
        const products = res.ok ? await res.json() : [];
        const tbody = document.querySelector('#products-table tbody');
        if (!tbody || !Array.isArray(products)) return;
        tbody.innerHTML = products.map(p => `
            <tr>
                <td>${p.id}</td>
                <td>${p.name}</td>
                <td>${p.sku || 'N/A'}</td>
                <td>${p.category || 'N/A'}</td>
                <td>KES ${parseFloat(p.price).toFixed(2)}</td>
                <td>KES ${parseFloat(p.cost || 0).toFixed(2)}</td>
                <td>
                    <button onclick="editProduct(${p.id})">Edit</button>
                    <button onclick="deleteProduct(${p.id})">Delete</button>
                </td>
            </tr>
        `).join('');
    } catch (err) {
        console.error('Error loading products:', err);
    }
}

function showProductForm(product = null) {
    const isEdit = product && product.id;
    modalBody.innerHTML = `
        <h2>${isEdit ? 'Edit' : 'Add'} Product</h2>
        <form id="product-form">
            <label>Name*</label>
            <input type="text" name="name" value="${product?.name || ''}" required>
            <label>Description</label>
            <input type="text" name="description" value="${product?.description || ''}">
            <label>SKU</label>
            <input type="text" name="sku" value="${product?.sku || ''}">
            <label>Barcode</label>
            <input type="text" name="barcode" value="${product?.barcode || ''}">
            <label>Category</label>
            <input type="text" name="category" value="${product?.category || ''}">
            <label>Price*</label>
            <input type="number" name="price" value="${product?.price || 0}" step="0.01" required>
            <label>Cost</label>
            <input type="number" name="cost" value="${product?.cost || 0}" step="0.01">
            <button type="submit">Save</button>
            <button type="button" onclick="modal.classList.remove('active')">Cancel</button>
        </form>
    `;

    const form = document.getElementById('product-form');
    form.onsubmit = async (e) => {
        e.preventDefault();
        const data = Object.fromEntries(new FormData(form));
        try {
            const isEdit = product && product.id;
            const url = isEdit ? `${API_URL}/products/${product.id}` : `${API_URL}/products`;
            const method = isEdit ? 'PUT' : 'POST';
            const response = await fetch(url, { method, headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(data) });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.error || 'Failed to save product');
            }

            modal.classList.remove('active');
            loadProducts();
            loadDashboard();
        } catch (err) {
            alert('Error saving product: ' + err.message);
        }
    };

    modal.classList.add('active');
}

async function editProduct(id) {
    const product = await authFetch(`${API_URL}/products/${id}`).then(r => r.json());
    showProductForm(product);
}

async function deleteProduct(id) {
    if (confirm('Delete this product?')) {
        await authFetch(`${API_URL}/products/${id}`, { method: 'DELETE' });
        loadProducts();
    }
}

// ------------------- Customers -------------------
async function loadCustomers() {
    try {
        const res = await authFetch(`${API_URL}/customers`);
        const customers = res.ok ? await res.json() : [];
        const tbody = document.querySelector('#customers-table tbody');
        if (!tbody || !Array.isArray(customers)) return;
        tbody.innerHTML = customers.map(c => `
            <tr>
                <td>${c.id}</td>
                <td>${c.name}</td>
                <td>${c.email || 'N/A'}</td>
                <td>${c.phone || 'N/A'}</td>
                <td>
                    <button onclick="editCustomer(${c.id})">Edit</button>
                    <button onclick="deleteCustomer(${c.id})">Delete</button>
                </td>
            </tr>
        `).join('');
    } catch (err) {
        console.error('Error loading customers:', err);
    }
}

function showCustomerForm(customer = null) {
    const isEdit = customer && customer.id;
    modalBody.innerHTML = `
        <h2>${isEdit ? 'Edit' : 'Add'} Customer</h2>
        <form id="customer-form">
            <label>Name*</label>
            <input type="text" name="name" value="${customer?.name || ''}" required>
            <label>Email</label>
            <input type="email" name="email" value="${customer?.email || ''}">
            <label>Phone</label>
            <input type="text" name="phone" value="${customer?.phone || ''}">
            <label>Address</label>
            <input type="text" name="address" value="${customer?.address || ''}">
            <button type="submit">Save</button>
            <button type="button" onclick="modal.classList.remove('active')">Cancel</button>
        </form>
    `;

    const form = document.getElementById('customer-form');
    form.onsubmit = async (e) => {
        e.preventDefault();
        const data = Object.fromEntries(new FormData(form));
        try {
            const isEdit = customer && customer.id;
            const url = isEdit ? `${API_URL}/customers/${customer.id}` : `${API_URL}/customers`;
            const method = isEdit ? 'PUT' : 'POST';
            const response = await fetch(url, { method, headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(data) });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.error || 'Failed to save customer');
            }

            modal.classList.remove('active');
            loadCustomers();
            loadDashboard();
        } catch (err) {
            alert('Error saving customer: ' + err.message);
        }
    };

    modal.classList.add('active');
}

async function editCustomer(id) {
    const customer = await authFetch(`${API_URL}/customers/${id}`).then(r => r.json());
    showCustomerForm(customer);
}

async function deleteCustomer(id) {
    if (confirm('Delete this customer?')) {
        await authFetch(`${API_URL}/customers/${id}`, { method: 'DELETE' });
        loadCustomers();
    }
}

// ------------------- Inventory -------------------
async function loadInventory() {
    try {
        const res = await authFetch(`${API_URL}/inventory`);
        const inventory = res.ok ? await res.json() : [];
        const tbody = document.querySelector('#inventory-table tbody');
        if (!tbody || !Array.isArray(inventory)) return;
        tbody.innerHTML = inventory.map(item => {
            let status = 'OK';
            let statusClass = 'status-ok';
            if (item.quantity === 0) { status = 'Out of Stock'; statusClass = 'status-critical'; }
            else if (item.quantity <= item.min_stock_level) { status = 'Low Stock'; statusClass = 'status-low'; }
            return `
                <tr>
                    <td>${item.product_name}</td>
                    <td>${item.sku || 'N/A'}</td>
                    <td>${item.quantity}</td>
                    <td>${item.min_stock_level}</td>
                    <td class="${statusClass}">${status}</td>
                    <td>
                        <button onclick="showAdjustmentForm(${item.product_id}, '${item.product_name.replace(/'/g, "\\'")}', ${item.quantity})">Adjust</button>
                        <button onclick="viewInventoryLogs(${item.product_id}, '${item.product_name.replace(/'/g, "\\'")}')" style="background:#6b7280;">History</button>
                    </td>
                </tr>
            `;
        }).join('');
    } catch (err) {
        console.error('Error loading inventory:', err);
    }
}

async function showAdjustmentForm(productId, productName, currentQty) {
    modalBody.innerHTML = `
        <h2>Adjust Inventory: ${productName}</h2>
        <form id="adjustment-form">
            <p>Current Stock: <strong>${currentQty}</strong></p>
            <label>Adjustment Type</label>
            <select id="adj-type" style="width:100%; padding:8px; margin-bottom:15px;">
                <option value="add">Add Stock (+)</option>
                <option value="remove">Remove Stock (-)</option>
                <option value="set">Set Level (=)</option>
            </select>
            <label>Amount / New Level</label>
            <input type="number" id="adj-amount" required style="width:100%; padding:8px; margin-bottom:15px;">
            <label>Reason / Note</label>
            <input type="text" id="adj-reason" placeholder="e.g. Restock, Damaged, Correction" required style="width:100%; padding:8px; margin-bottom:15px;">
            <div style="display:flex; gap:10px; margin-top:10px;">
                <button type="submit" style="flex:1;">Save Adjustment</button>
                <button type="button" onclick="modal.classList.remove('active')" style="background:#6b7280; flex:1;">Cancel</button>
            </div>
        </form>
    `;

    const form = document.getElementById('adjustment-form');
    form.onsubmit = async (e) => {
        e.preventDefault();
        const type = document.getElementById('adj-type').value;
        const amount = parseInt(document.getElementById('adj-amount').value);
        const reason = document.getElementById('adj-reason').value;

        let newQty = currentQty;
        let change = amount;

        if (type === 'add') newQty += amount;
        else if (type === 'remove') { newQty -= amount; change = -amount; }
        else if (type === 'set') { newQty = amount; change = amount - currentQty; }

        try {
            const res = await authFetch(`${API_URL}/inventory/${productId}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    quantity: newQty,
                    change_amount: change,
                    reason: reason
                })
            });

            if (!res.ok) throw new Error('Failed to update inventory');

            modal.classList.remove('active');
            loadInventory();
            loadDashboard();
        } catch (err) {
            alert('Error: ' + err.message);
        }
    };

    modal.classList.add('active');
}

async function viewInventoryLogs(productId, productName) {
    try {
        const res = await authFetch(`${API_URL}/inventory/${productId}/logs`);
        const logs = res.ok ? await res.json() : [];

        modalContent.classList.add('wide');
        modalBody.innerHTML = `
            <h2>Stock History: ${productName}</h2>
            <table>
                <thead>
                    <tr><th>Date</th><th>Change</th><th>New Level</th><th>Reason</th></tr>
                </thead>
                <tbody>
                    ${(!Array.isArray(logs) || logs.length === 0) ? '<tr><td colspan="4" style="text-align:center">No adjustments yet</td></tr>' :
                logs.map(log => `
                            <tr>
                                <td>${new Date(log.created_at).toLocaleString()}</td>
                                <td style="color: ${log.change_amount > 0 ? 'green' : (log.change_amount < 0 ? 'red' : 'inherit')}">
                                    ${log.change_amount > 0 ? '+' : ''}${log.change_amount}
                                </td>
                                <td>${log.new_quantity}</td>
                                <td>${log.reason || 'N/A'}</td>
                            </tr>
                        `).join('')
            }
                </tbody>
            </table>
            <button onclick="modal.classList.remove('active'); modalContent.classList.remove('wide');" style="margin-top:20px; width:100%; background:#6b7280;">Close</button>
        `;
        modal.classList.add('active');
    } catch (err) {
        alert('Error loading history: ' + err.message);
    }
}

// ------------------- Sales -------------------
async function loadSales() {
    try {
        const res = await authFetch(`${API_URL}/sales`);
        const sales = res.ok ? await res.json() : [];
        const tbody = document.querySelector('#sales-table tbody');
        if (!tbody || !Array.isArray(sales)) return;
        tbody.innerHTML = sales.map(sale => `
            <tr>
                <td>${sale.id}</td>
                <td>${sale.customer_name || 'Walk-in'}</td>
                <td>KES ${parseFloat(sale.total_amount).toFixed(2)}</td>
                <td>${sale.payment_method || 'N/A'}</td>
                <td>
                    <button onclick="viewSale(${sale.id})">View</button>
                </td>
            </tr>
        `).join('');
    } catch (err) {
        console.error('Error loading sales:', err);
    }
}

let cart = [];

async function showSaleForm() {
    const pRes = await authFetch(`${API_URL}/products`);
    const cRes = await authFetch(`${API_URL}/customers`);

    const products = pRes.ok ? await pRes.json() : [];
    const customers = cRes.ok ? await cRes.json() : [];

    cart = [];

    const modalContent = document.querySelector('.modal-content');
    modalContent.classList.add('wide');

    function renderPOS() {
        const total = cart.reduce((sum, item) => sum + (item.price * item.quantity), 0);
        const safeProducts = Array.isArray(products) ? products : [];
        const safeCustomers = Array.isArray(customers) ? customers : [];

        modalBody.innerHTML = `
            <h2>Point of Sale</h2>
            <div class="pos-container">
                <div class="product-selection">
                    <h3>Select Products</h3>
                    <input type="text" id="pos-product-search" placeholder="Search products..." style="width:100%; margin-bottom:10px; padding:10px;">
                    <div id="pos-product-list" style="height: 300px; overflow-y: auto; border: 1px solid #ddd; border-radius: 4px;">
                        ${safeProducts.map(p => `
                            <div class="product-item" style="padding: 10px; border-bottom: 1px solid #eee; display: flex; justify-content: space-between; align-items: center;">
                                <div>
                                    <strong>${p.name}</strong><br>
                                    <small>${p.sku || 'No SKU'} - KES ${parseFloat(p.price).toFixed(2)}</small>
                                </div>
                                <button onclick="addToCart(${p.id}, '${p.name.replace(/'/g, "\\'")}', ${p.price})">Add</button>
                            </div>
                        `).join('')}
                    </div>
                </div>
                <div class="cart-section">
                    <h3>Current Sale</h3>
                    <label>Customer</label>
                    <select id="pos-customer" style="width:100%; padding:10px; margin-bottom:15px;">
                        <option value="">Walk-in Customer</option>
                        ${safeCustomers.map(c => `<option value="${c.id}">${c.name}</option>`).join('')}
                    </select>
                    
                    <table class="cart-table">
                        <thead>
                            <tr><th>Item</th><th>Price</th><th>Qty</th><th>Subtotal</th><th></th></tr>
                        </thead>
                        <tbody id="cart-items">
                            ${cart.length === 0 ? '<tr><td colspan="5" style="text-align:center">Cart is empty</td></tr>' :
                cart.map(item => `
                                    <tr>
                                        <td>${item.name}</td>
                                        <td>KES ${item.price.toFixed(2)}</td>
                                        <td><input type="number" class="qty-input" value="${item.quantity}" min="1" onchange="updateCartQty(${item.productId}, this.value)"></td>
                                        <td>KES ${(item.price * item.quantity).toFixed(2)}</td>
                                        <td><button onclick="removeFromCart(${item.productId})" style="background:#ef4444;">&times;</button></td>
                                    </tr>
                                `).join('')
            }
                        </tbody>
                    </table>

                    <div class="cart-summary">
                        <h3>Total: KES ${total.toFixed(2)}</h3>
                        <div style="display:flex; gap:10px; justify-content: flex-end; margin-top:15px;">
                            <button onclick="saveSale()" style="background:#10b981;">Complete Sale</button>
                            <button onclick="modal.classList.remove('active'); document.querySelector('.modal-content').classList.remove('wide');" style="background:#6b7280;">Cancel</button>
                        </div>
                    </div>
                </div>
            </div>
        `;

        // Add search functionality
        document.getElementById('pos-product-search').oninput = (e) => {
            const term = e.target.value.toLowerCase();
            const items = document.querySelectorAll('.product-item');
            items.forEach(item => {
                const text = item.textContent.toLowerCase();
                item.style.display = text.includes(term) ? 'flex' : 'none';
            });
        };
    }

    // Define globally for onclick handlers
    window.addToCart = (productId, name, price) => {
        const existing = cart.find(i => i.productId === productId);
        if (existing) {
            existing.quantity++;
        } else {
            cart.push({ productId, name, price, quantity: 1 });
        }
        renderPOS();
    };

    window.updateCartQty = (productId, qty) => {
        const item = cart.find(i => i.productId === productId);
        if (item) {
            item.quantity = parseInt(qty) || 1;
            renderPOS();
        }
    };

    window.removeFromCart = (productId) => {
        cart = cart.filter(i => i.productId !== productId);
        renderPOS();
    };

    window.saveSale = async () => {
        if (cart.length === 0) return alert('Cart is empty');

        const saleData = {
            customer_id: document.getElementById('pos-customer').value || null,
            payment_method: 'Cash', // Default for now
            total_amount: cart.reduce((sum, item) => sum + (item.price * item.quantity), 0),
            items: cart.map(item => ({
                product_id: item.productId,
                quantity: item.quantity,
                unit_price: item.price,
                subtotal: item.price * item.quantity
            }))
        };

        try {
            const res = await authFetch(`${API_URL}/sales`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(saleData)
            });

            if (!res.ok) throw new Error('Failed to save sale');

            const savedSale = await res.json();
            alert('Sale completed successfully!');

            // Show print option
            if (confirm('Do you want to print the receipt?')) {
                printReceipt(savedSale, cart);
            }

            modal.classList.remove('active');
            modalContent.classList.remove('wide');
            loadSales();
            loadDashboard();
        } catch (err) {
            alert('Error creating sale: ' + err.message);
        }
    };

    renderPOS();
    modal.classList.add('active');
}

function printReceipt(sale, cartItems) {
    const receiptDiv = document.getElementById('receipt-print');
    const total = cartItems.reduce((sum, item) => sum + (item.price * item.quantity), 0);

    receiptDiv.innerHTML = `
        <div style="text-align:center; margin-bottom: 20px;">
            <h1>Shop Management System</h1>
            <p>123 Business Street, City Name</p>
            <p>Tel: +123 456 789</p>
            <hr>
            <h3>RECEIPT</h3>
            <p>Sale ID: #${sale.id}</p>
            <p>Date: ${new Date().toLocaleString()}</p>
        </div>
        <table style="width:100%; border-collapse: collapse;">
            <thead>
                <tr style="border-bottom: 1px solid #000;">
                    <th style="text-align:left;">Item</th>
                    <th style="text-align:center;">Qty</th>
                    <th style="text-align:right;">Subtotal</th>
                </tr>
            </thead>
            <tbody>
                ${cartItems.map(item => `
                    <tr>
                        <td>${item.name}</td>
                        <td style="text-align:center;">${item.quantity}</td>
                        <td style="text-align:right;">KES ${(item.price * item.quantity).toFixed(2)}</td>
                    </tr>
                `).join('')}
            </tbody>
        </table>
        <hr>
        <div style="text-align:right; font-size: 1.2em; font-weight: bold;">
            TOTAL: KES ${total.toFixed(2)}
        </div>
        <div style="text-align:center; margin-top: 30px;">
            <p>Thank you for shopping with us!</p>
        </div>
    `;

    setTimeout(() => {
        window.print();
    }, 500);
}

async function viewSale(id) {
    const sale = await authFetch(`${API_URL}/sales/${id}`).then(r => r.json());
    const items = await authFetch(`${API_URL}/sales/${id}/items`).then(r => r.json());
    modalBody.innerHTML = `
        <h2>Sale #${sale.id}</h2>
        <p>Customer: ${sale.customer_name || 'Walk-in'}</p>
        <p>Date: ${new Date(sale.sale_date).toLocaleString()}</p>
        <p>Payment: ${sale.payment_method}</p>
        <table>
            <thead><tr><th>Product</th><th>Qty</th><th>Price</th><th>Subtotal</th></tr></thead>
            <tbody>
                ${items.map(i => `<tr>
                    <td>${i.product_name}</td>
                    <td>${i.quantity}</td>
                    <td>KES ${parseFloat(i.unit_price).toFixed(2)}</td>
                    <td>KES ${parseFloat(i.subtotal).toFixed(2)}</td>
                </tr>`).join('')}
            </tbody>
        </table>
        <p>Total: KES ${parseFloat(sale.total_amount).toFixed(2)}</p>
    `;
    modal.classList.add('active');
}

// ------------------- Reports -------------------
let trendChart = null;
let categoryChart = null;

async function generateReport() {
    const from = document.getElementById('report-from').value;
    const to = document.getElementById('report-to').value;
    if (!from || !to) { alert('Select date range'); return; }

    try {
        const [report, trends, categories] = await Promise.all([
            authFetch(`${API_URL}/reports/sales?from=${from}&to=${to}`).then(r => r.json()),
            authFetch(`${API_URL}/reports/trends?from=${from}&to=${to}`).then(r => r.json()),
            authFetch(`${API_URL}/reports/categories?from=${from}&to=${to}`).then(r => r.json())
        ]);

        // Show report sections
        document.getElementById('report-summary').style.display = 'grid';
        document.getElementById('report-charts').style.display = 'grid';
        document.getElementById('report-table-container').style.display = 'block';
        document.getElementById('export-csv-btn').style.display = 'block';

        // Update Summary Cards
        document.getElementById('rep-total-sales').textContent = `KES ${report.totalSales.toFixed(2)}`;
        document.getElementById('rep-total-profit').textContent = `KES ${report.totalProfit.toFixed(2)}`;
        document.getElementById('rep-total-trans').textContent = report.totalTransactions;
        document.getElementById('rep-avg-sale').textContent = `KES ${report.avgSale.toFixed(2)}`;

        // Update Table
        const resultsDiv = document.getElementById('report-results');
        resultsDiv.innerHTML = report.topProducts.map(p => `
            <tr>
                <td>${p.product_name}</td>
                <td>${p.total_quantity}</td>
                <td>KES ${parseFloat(p.total_revenue).toFixed(2)}</td>
            </tr>
        `).join('');

        renderCharts(trends, categories);

        // Store current report for export
        window.currentReportData = { report, trends, categories, from, to };

    } catch (err) {
        console.error('Report error:', err);
        alert('Failed to generate report');
    }
}

function renderCharts(trends, categories) {
    const trendCtx = document.getElementById('trendChart').getContext('2d');
    const catCtx = document.getElementById('categoryChart').getContext('2d');

    // Destroy existing charts if they exist
    if (trendChart) trendChart.destroy();
    if (categoryChart) categoryChart.destroy();

    // Trend Chart (Line)
    trendChart = new Chart(trendCtx, {
        type: 'line',
        data: {
            labels: trends.map(t => new Date(t.date).toLocaleDateString()),
            datasets: [
                {
                    label: 'Sales',
                    data: trends.map(t => t.total_sales),
                    borderColor: '#3b82f6',
                    backgroundColor: 'rgba(59, 130, 246, 0.1)',
                    fill: true,
                    tension: 0.4
                },
                {
                    label: 'Profit',
                    data: trends.map(t => t.total_profit),
                    borderColor: '#10b981',
                    backgroundColor: 'rgba(16, 185, 129, 0.1)',
                    fill: true,
                    tension: 0.4
                }
            ]
        },
        options: {
            responsive: true,
            plugins: {
                legend: { position: 'top' }
            },
            scales: {
                y: { beginAtZero: true }
            }
        }
    });

    // Category Chart (Pie/Doughnut)
    categoryChart = new Chart(catCtx, {
        type: 'doughnut',
        data: {
            labels: categories.map(c => c.category),
            datasets: [{
                data: categories.map(c => c.total_revenue),
                backgroundColor: [
                    '#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6', '#ec4899'
                ]
            }]
        },
        options: {
            responsive: true,
            plugins: {
                legend: { position: 'bottom' }
            }
        }
    });
}

function exportToCSV() {
    const data = window.currentReportData;
    if (!data) return;

    let csv = `Report from ${data.from} to ${data.to}\n\n`;
    csv += `Total Sales,KES ${data.report.totalSales.toFixed(2)}\n`;
    csv += `Total Profit,KES ${data.report.totalProfit.toFixed(2)}\n`;
    csv += `Transactions,${data.report.totalTransactions}\n`;
    csv += `Avg Sale,KES ${data.report.avgSale.toFixed(2)}\n\n`;

    csv += `Top Products\nProduct,Units Sold,Revenue\n`;
    data.report.topProducts.forEach(p => {
        csv += `"${p.product_name}",${p.total_quantity},${p.total_revenue}\n`;
    });

    const blob = new Blob([csv], { type: 'text/csv' });
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.setAttribute('hidden', '');
    a.setAttribute('href', url);
    a.setAttribute('download', `report_${data.from}_to_${data.to}.csv`);
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
}
