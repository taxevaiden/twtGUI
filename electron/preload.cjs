const { contextBridge, ipcRenderer, app } = require("electron");

contextBridge.exposeInMainWorld("electronAPI", {
  invoke: (channel, ...args) => ipcRenderer.invoke(channel, ...args),
  getVersion: () => ipcRenderer.invoke("getVersion"),
  getPlatform: () => ipcRenderer.invoke("getPlatform"),
  getVersions: () => ipcRenderer.invoke("getVersions"),
});
