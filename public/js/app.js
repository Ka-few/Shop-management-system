const API_URL = 'http://localhost:3000/api';

// Navigation
document.querySelectorAll('.nav-item').forEach(item => {
    item.addEventListener('click', (e) => {
        e.preventDefault();
        const page = item.dataset.page;
        showPage(page);
        
        document.querySelectorAll('.nav-item').forEach(i => i.classList.remove('active'));
        item.classList.add('active');
        
        document.getElementById('page-title').textContent = 
            page.charAt(0).toUpperCase() + page.slice(1);
    });
});

function showPage(pageName) {
    document.querySelectorAll('.page').forEach(p => p.classList.remove('active'));
    document.getElementById(`${pageName}-page`).classList.add('active');
    
    switch(pageName) {
        case 'dashboard':
            loadDashboard();
            break;
        case 'products':
            loadProducts();
            break;
        case 'inventory':
            loadInventory();
            break;
        case 'sales':
            loadSales();
            break;
        case 'customers':
            loadCustomers();
            break;
    }
}

// Modal handling
const modal = document.getElementById('modal');
const closeBtn = document.querySelector('.close');

closeBtn.onclick = () => modal.classList.remove('active');
window.onclick = (e) => {
    if (e.target === modal) modal.classList.remove('active');
};

// Dashboard
async function loadDashboard() {
    try {
        const [products, sales, inventory, customers] = await Promise.all([
            fetch(`${API_URL}/products`).then(r => r.json()),
            fetch(`${API_URL}/sales`).then(r => r.json()),
            fetch(`${API_URL}/inventory/low-stock`).then(r => r.json()),
            fetch(`${API_URL}/customers`).then(r => r.json())
        ]);
        
        document.getElementById('total-products').textContent = products.length;
        document.getElementById('low-stock').textContent = inventory.length;
        document.getElementById('total-customers').textContent = customers.length;
        
        const totalSales = sales.reduce((sum, sale) => sum + parseFloat(sale.total_amount), 0);
        document.getElementById('total-sales').textContent = `$${totalSales.toFixed(2)}`;
        
        // Recent sales
        const tbody = document.querySelector('#recent-sales-table tbody');
        tbody.innerHTML = sales.slice(0, 5).map(sale => `
            
                ${sale.id}
                ${sale.customer_name || 'Walk-in'}
                $${parseFloat(sale.total_amount).toFixed(2)}
                ${new Date(sale.sale_date).toLocaleDateString()}
            
        `).join('');
    } catch (err) {
        console.error('Error loading dashboard:', err);
    }
}

// Products
async function loadProducts() {
    try {
        const products = await fetch(`${API_URL}/products`).then(r => r.json());
        const tbody = document.querySelector('#products-table tbody');
        
        tbody.innerHTML = products.map(p => `
            
                ${p.id}
                ${p.name}
                ${p.sku || 'N/A'}
                ${p.category || 'N/A'}
                $${parseFloat(p.price).toFixed(2)}
                $${parseFloat(p.cost || 0).toFixed(2)}
                
                    Edit
                    Delete
                
            
        `).join('');
    } catch (err) {
        console.error('Error loading products:', err);
    }
}

document.getElementById('add-product-btn').onclick = () => showProductForm();

function showProductForm(product = null) {
    const modalBody = document.getElementById('modal-body');
    modalBody.innerHTML = `
        ${product ? 'Edit' : 'Add'} Product
        
            
                Name*
                
            
            
                Description
                ${product?.description || ''}
            
            
                SKU
                
            
            
                Barcode
                
            
            
                Category
                
            
            
                Price*
                
            
            
                Cost
                
            
            Save
            Cancel
        
    `;
    
    document.getElementById('product-form').onsubmit = async (e) => {
        e.preventDefault();
        const formData = new FormData(e.target);
        const data = Object.fromEntries(formData);
        
        try {
            const url = product ? `${API_URL}/products/${product.id}` : `${API_URL}/products`;
            const method = product ? 'PUT' : 'POST';
            
            await fetch(url, {
                method,
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify(data)
            });
            
            modal.classList.remove('active');
            loadProducts();
        } catch (err) {
            alert('Error saving product: ' + err.message);
        }
    };
    
    modal.classList.add('active');
}

async function editProduct(id) {
    const product = await fetch(`${API_URL}/products/${id}`).then(r => r.json());
    showProductForm(product);
}

async function deleteProduct(id) {
    if (confirm('Are you sure you want to delete this product?')) {
        try {
            await fetch(`${API_URL}/products/${id}`, {method: 'DELETE'});
            loadProducts();
        } catch (err) {
            alert('Error deleting product: ' + err.message);
        }
    }
}

// Inventory
async function loadInventory() {
    try {
        const inventory = await fetch(`${API_URL}/inventory`).then(r => r.json());
        const tbody = document.querySelector('#inventory-table tbody');
        
        tbody.innerHTML = inventory.map(item => {
            let status = 'OK';
            let statusClass = 'status-ok';
            
            if (item.quantity === 0) {
                status = 'Out of Stock';
                statusClass = 'status-critical';
            } else if (item.quantity <= item.min_stock_level) {
                status = 'Low Stock';
                statusClass = 'status-low';
            }
            
            return `
                
                    ${item.product_name}
                    ${item.sku || 'N/A'}
                    ${item.quantity}
                    ${item.min_stock_level}
                    ${status}
                    
                        Update Stock
                    
                
            `;
        }).join('');
    } catch (err) {
        console.error('Error loading inventory:', err);
    }
}

async function updateStock(productId) {
    const newQty = prompt('Enter new stock quantity:');
    if (newQty !== null) {
        try {
            await fetch(`${API_URL}/inventory/${productId}`, {
                method: 'PUT',
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({quantity: parseInt(newQty)})
            });
            loadInventory();
        } catch (err) {
            alert('Error updating stock: ' + err.message);
        }
    }
}

// Sales
async function loadSales() {
    try {
        const sales = await fetch(`${API_URL}/sales`).then(r => r.json());
        const tbody = document.querySelector('#sales-table tbody');
        
        tbody.innerHTML = sales.map(sale => `
            
                ${sale.id}
                ${sale.customer_name || 'Walk-in'}
                $${parseFloat(sale.total_amount).toFixed(2)}
                ${sale.payment_method || 'N/A'}
                ${new Date(sale.sale_date).toLocaleDateString()}
                
                    View
                
            
        `).join('');
    } catch (err) {
        console.error('Error loading sales:', err);
    }
}

document.getElementById('new-sale-btn').onclick = () => showSaleForm();

function showSaleForm() {
    // Implementation for new sale form
    alert('New Sale form - implement POS interface here');
}

async function viewSale(id) {
    const sale = await fetch(`${API_URL}/sales/${id}`).then(r => r.json());
    const items = await fetch(`${API_URL}/sales/${id}/items`).then(r => r.json());
    
    const modalBody = document.getElementById('modal-body');
    modalBody.innerHTML = `
        Sale #${sale.id}
        Customer: ${sale.customer_name || 'Walk-in'}
        Date: ${new Date(sale.sale_date).toLocaleString()}
        Payment: ${sale.payment_method}
        Items
        
            
                ProductQtyPriceSubtotal
            
            
                ${items.map(item => `
                    
                        ${item.product_name}
                        ${item.quantity}
                        $${parseFloat(item.unit_price).toFixed(2)}
                        $${parseFloat(item.subtotal).toFixed(2)}
                    
                `).join('')}
            
        
        Total: $${parseFloat(sale.total_amount).toFixed(2)}
    `;
    modal.classList.add('active');
}

// Customers
async function loadCustomers() {
    try {
        const customers = await fetch(`${API_URL}/customers`).then(r => r.json());
        const tbody = document.querySelector('#customers-table tbody');
        
        tbody.innerHTML = customers.map(c => `
            
                ${c.id}
                ${c.name}
                ${c.email || 'N/A'}
                ${c.phone || 'N/A'}
                
                    Edit
                    Delete
                
            
        `).join('');
    } catch (err) {
        console.error('Error loading customers:', err);
    }
}

document.getElementById('add-customer-btn').onclick = () => showCustomerForm();

function showCustomerForm(customer = null) {
    const modalBody = document.getElementById('modal-body');
    modalBody.innerHTML = `
        ${customer ? 'Edit' : 'Add'} Customer
        
            
                Name*
                
            
            
                Email
                
            
            
                Phone
                
            
            
                Address
                ${customer?.address || ''}
            
            Save
            Cancel
        
    `;
    
    document.getElementById('customer-form').onsubmit = async (e) => {
        e.preventDefault();
        const formData = new FormData(e.target);
        const data = Object.fromEntries(formData);
        
        try {
            const url = customer ? `${API_URL}/customers/${customer.id}` : `${API_URL}/customers`;
            const method = customer ? 'PUT' : 'POST';
            
            await fetch(url, {
                method,
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify(data)
            });
            
            modal.classList.remove('active');
            loadCustomers();
        } catch (err) {
            alert('Error saving customer: ' + err.message);
        }
    };
    
    modal.classList.add('active');
}

async function editCustomer(id) {
    const customer = await fetch(`${API_URL}/customers/${id}`).then(r => r.json());
    showCustomerForm(customer);
}

async function deleteCustomer(id) {
    if (confirm('Are you sure you want to delete this customer?')) {
        try {
            await fetch(`${API_URL}/customers/${id}`, {method: 'DELETE'});
            loadCustomers();
        } catch (err) {
            alert('Error deleting customer: ' + err.message);
        }
    }
}

// Reports
document.getElementById('generate-report-btn').onclick = async () => {
    const from = document.getElementById('report-from').value;
    const to = document.getElementById('report-to').value;
    
    if (!from || !to) {
        alert('Please select date range');
        return;
    }
    
    try {
        const report = await fetch(`${API_URL}/reports/sales?from=${from}&to=${to}`).then(r => r.json());
        
        const resultsDiv = document.getElementById('report-results');
        resultsDiv.innerHTML = `
            
                
                    Total Sales
                    $${report.totalSales.toFixed(2)}
                
                
                    Total Transactions
                    ${report.totalTransactions}
                
                
                    Average Sale
                    $${report.avgSale.toFixed(2)}
                
                
                    Total Profit
                    $${report.totalProfit.toFixed(2)}
                
            
            Top Selling Products
            
                
                    ProductUnits SoldRevenue
                
                
                    ${report.topProducts.map(p => `
                        
                            ${p.product_name}
                            ${p.total_quantity}
                            ${parseFloat(p.total_revenue).toFixed(2)}
                        
                    `).join('')}
                
            
        `;
    } catch (err) {
        alert('Error generating report: ' + err.message);
    }
};

// Initialize dashboard on load
loadDashboard();