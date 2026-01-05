const { app, BrowserWindow } = require('electron');
const path = require('path');
const { spawn } = require('child_process');
const http = require('http');

let mainWindow;
let serverProcess;

// Function to wait for the server to be ready
function waitForServer(url, callback) {
  const interval = setInterval(() => {
    http.get(url, () => {
      clearInterval(interval);
      callback();
    }).on('error', () => {
      // Server not ready yet
    });
  }, 500); // check every 500ms
}

// Function to create the Electron window
function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js') // optional, if you have one
    }
  });

  // Wait for server to be ready before loading
  waitForServer('http://localhost:3000', () => {
    mainWindow.loadURL('http://localhost:3000');
  });

  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

// Function to start the backend server
function startServer() {
  serverProcess = spawn('node', ['server/server.js'], {
    stdio: 'inherit'
  });
}

// Event: App ready
app.on('ready', () => {
  startServer();
  createWindow();
});

// Event: All windows closed
app.on('window-all-closed', () => {
  if (serverProcess) serverProcess.kill();
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

// Event: App activated (macOS)
app.on('activate', () => {
  if (mainWindow === null) {
    createWindow();
  }
});

// Ensure server is killed on process exit (Ctrl+C)
process.on('SIGINT', () => {
  if (serverProcess) serverProcess.kill();
  process.exit();
});
