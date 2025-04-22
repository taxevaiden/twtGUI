const { ipcMain, app, BrowserWindow, shell } = require("electron");
const path = require("path");
const os = require("os");
const process = require("process");

let mainWindow;

async function createWindow() {
  mainWindow = new BrowserWindow({
    minWidth: 1056,
    width: 1056,
    minHeight: 600,
    height: 600,
    titleBarStyle: "hidden",
    trafficLightPosition: { x: 16, y: 7 },
    // titleBarOverlay: {
    //     color: "#020617",
    //     symbolColor: "#bbf7d0",
    //     height: 30,
    // },
    webPreferences: {
      preload: path.join(__dirname, "preload.cjs"), // Optional: preload script
      nodeIntegration: true,
      contextIsolation: true,
    },
  });

  mainWindow.setBackgroundColor("#020617");

  const isDev = process.env.NODE_ENV === "development";

  if (isDev) {
    // load dev server URL
    mainWindow.loadURL("http://localhost:8080");
  } else {
    // serve the built Astro SSR app
    try {
      const server = await import("../dist/server/entry.mjs");

      if (!server || typeof server.startServer !== "function") {
        throw new Error(
          "startServer is not a function or missing in the entry file.",
        );
      }

      const { server: serverDetails } = await server.startServer();
      const url = `http://${serverDetails.host}:${serverDetails.port}`;
      console.log(url);
      mainWindow.loadURL(url);
    } catch (error) {
      console.error("Failed to start the server or load the URL:", error);
      app.quit(); // exit the app if the server fails to start
    }
  }

  // mainWindow.removeMenu();

  mainWindow.on("closed", () => {
    mainWindow = null;
  });

  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    if (!url.startsWith("http://localhost")) {
      shell.openExternal(url);
      return { action: "deny" };
    }
    return { action: "allow" };
  });

  mainWindow.webContents.on("will-navigate", (event, url) => {
    if (!url.startsWith("http://localhost")) {
      event.preventDefault();
      shell.openExternal(url);
    }
  });

  mainWindow.webContents.on("did-fail-load", (event, code, desc) => {
    console.error("Failed to load URL:", code, desc);
  });

  mainWindow.webContents.on("did-finish-load", () => {
    console.log("Electron finished loading the page!");
  });
}

app.whenReady().then(() => {
  createWindow();

  ipcMain.handle("getVersion", () => {
    return app.getVersion();
  });

  ipcMain.handle("getVersions", () => {
    return {
      node: process.versions.node,
      chrome: process.versions.chrome,
      v8: process.versions.v8,
      electron: process.versions.electron,
    };
  });

  ipcMain.handle("getPlatform", () => {
    return os.platform();
  });

  ipcMain.handle("get-window-state", () => {
    return mainWindow.isMaximized() ? "maximized" : "restored";
  });

  ipcMain.on("reload-window", (event) => {
    try {
      if (mainWindow && !mainWindow.isDestroyed()) {
        mainWindow.webContents.reload();
      } else {
        console.error("Main window is not available or destroyed.");
      }
    } catch (error) {
      console.error("Error while reloading the window:", error);
    }
  });

  ipcMain.handle("window-minimize", async () => {
    try {
      if (mainWindow && !mainWindow.isDestroyed()) {
        mainWindow.minimize();
      } else {
        console.error("Main window is not available or destroyed.");
      }
    } catch (error) {
      console.error("Error while minimizing the window:", error);
    }
  });

  ipcMain.handle("window-maximize", async () => {
    try {
      if (mainWindow && !mainWindow.isDestroyed()) {
        if (mainWindow.isMaximized()) {
          mainWindow.restore(); // Restore the window
        } else {
          mainWindow.maximize(); // Maximize the window
        }
      } else {
        console.error("Main window is not available or destroyed.");
      }
    } catch (error) {
      console.error("Error while maximizing the window:", error);
    }
  });

  ipcMain.handle("window-close", async () => {
    try {
      if (mainWindow && !mainWindow.isDestroyed()) {
        mainWindow.close();
      } else {
        console.error("Main window is not available or destroyed.");
      }
    } catch (error) {
      console.error("Error while closing the window:", error);
    }
  });
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});
