const http = require('http');

const endpoints = [
    '/api/products',
    '/api/customers',
    '/api/sales',
    '/api/inventory'
];

async function test(endpoint) {
    return new Promise((resolve) => {
        http.get(`http://localhost:3000${endpoint}`, (res) => {
            let data = '';
            res.on('data', (chunk) => data += chunk);
            res.on('end', () => {
                resolve({ endpoint, status: res.statusCode, response: data });
            });
        }).on('error', (err) => {
            resolve({ endpoint, status: 'Error', response: err.message });
        });
    });
}

async function run() {
    for (const endpoint of endpoints) {
        const result = await test(endpoint);
        console.log(`Endpoint: ${result.endpoint}`);
        console.log(`Status: ${result.status}`);
        console.log(`Response: ${result.response.substring(0, 100)}${result.response.length > 100 ? '...' : ''}`);
        console.log('---');
    }
}

run();
