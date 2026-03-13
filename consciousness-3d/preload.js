const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('api', {
  onPythonMessage: (cb) => ipcRenderer.on('python-message', (event, data) => cb(data)),
  sendToPython: (obj) => ipcRenderer.send('send-to-python', obj),
});
