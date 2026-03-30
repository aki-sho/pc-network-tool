const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('networkApi', {
  getAll: () => ipcRenderer.invoke('network:get-all')
});