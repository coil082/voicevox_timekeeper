const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

window.addEventListener("DOMContentLoaded", async() => {
  console.log("load");
  await listen("1hour", () => {
    console.log("1hour event received");

  });
});
