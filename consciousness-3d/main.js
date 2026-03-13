const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');

const engine = require('../native/consciousness.darwin-arm64.node');

let mainWindow;
let tickInterval;

function createWindow() {
    mainWindow = new BrowserWindow({
        width: 1280,
        height: 800,
        backgroundColor: '#0D1117',
        webPreferences: {
            preload: path.join(__dirname, 'preload.js'),
            contextIsolation: true,
            nodeIntegration: false,
        },
    });

    mainWindow.loadFile('index.html');

    // Create engine — randomize starting weights for unique personality
    const init = JSON.parse(engine.createEngine(false, null));
    engine.randomizeWeights(null); // random seed from system clock

    mainWindow.webContents.on('did-finish-load', () => {
        mainWindow.webContents.send('python-message', init);
        mainWindow.webContents.send('python-message', {
            type: 'traits',
            traits: JSON.parse(engine.getTraits()),
        });

        // Tick at 60Hz
        tickInterval = setInterval(() => {
            if (mainWindow && !mainWindow.isDestroyed()) {
                const state = JSON.parse(engine.tick());
                mainWindow.webContents.send('python-message', state);
            }
        }, 16);
    });

    ipcMain.on('send-to-python', (event, msg) => {
        if (msg.type === 'input') {
            engine.injectInput(msg.direction);
        } else if (msg.type === 'config' && msg.personality !== undefined) {
            engine.setPersonality(msg.personality || null);
        } else if (msg.type === 'randomize') {
            engine.randomizeWeights(null);
            // Send updated traits to renderer
            mainWindow.webContents.send('python-message', {
                type: 'traits',
                traits: JSON.parse(engine.getTraits()),
            });
        } else if (msg.type === 'get-traits') {
            mainWindow.webContents.send('python-message', {
                type: 'traits',
                traits: JSON.parse(engine.getTraits()),
            });
        }
    });

    mainWindow.on('closed', () => {
        if (tickInterval) clearInterval(tickInterval);
        mainWindow = null;
    });
}

app.whenReady().then(createWindow);
app.on('window-all-closed', () => app.quit());
