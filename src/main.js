const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

window.addEventListener("DOMContentLoaded", async() => {
  console.log("load");
  let setting= document.getElementById("setting");
  let reading_textarea = document.getElementById("reading-text");
  setting.addEventListener("submit",()=>{
    invoke("setting",{readText:reading_textarea.value})
  });
  await listen("1hour", () => {
    console.log("1hour event received");

  });
});
